use rusqlite::{Connection, OpenFlags, params};
use std::{
    fmt, fs,
    path::{Path, PathBuf},
};
#[cfg(feature = "proofs")]
use std::{
    io::Write,
    process::{Command, ExitStatus, Stdio},
};
use thiserror::Error;
use zeroize::{Zeroize, ZeroizeOnDrop};

const SCHEMA_VERSION: i64 = 1;
const KEY_BYTES: usize = 32;

#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct VaultKey([u8; KEY_BYTES]);

impl VaultKey {
    pub fn generate() -> Result<Self, VaultError> {
        let mut key = [0_u8; KEY_BYTES];
        getrandom::fill(&mut key).map_err(|_| VaultError::SecureRandom)?;
        Ok(Self(key))
    }

    pub fn from_bytes(bytes: [u8; KEY_BYTES]) -> Self {
        Self(bytes)
    }
}

impl fmt::Debug for VaultKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("VaultKey([REDACTED])")
    }
}

#[derive(Debug, Error)]
pub enum VaultError {
    #[error("the key or account vault is invalid")]
    InvalidKeyOrVault,
    #[error("the account vault operation failed")]
    Operation,
    #[error("the operating system could not create secure random data")]
    SecureRandom,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LabReportDraft {
    pub title: String,
    pub observed_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LabReport {
    pub id: i64,
    pub title: String,
    pub observed_at: String,
}

pub struct AccountVault {
    connection: Connection,
    path: PathBuf,
    #[cfg(feature = "proofs")]
    key: VaultKey,
}

impl AccountVault {
    fn open(path: impl AsRef<Path>, key: &VaultKey) -> Result<Self, VaultError> {
        let path = path.as_ref().to_owned();
        let connection = Connection::open_with_flags(
            &path,
            OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_CREATE
                | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(|_| VaultError::InvalidKeyOrVault)?;
        apply_key(&connection, key)?;
        configure(&connection)?;
        migrate(&connection)?;
        Ok(Self {
            connection,
            path,
            #[cfg(feature = "proofs")]
            key: key.clone(),
        })
    }

    pub fn add_lab_report(&self, report: &LabReportDraft) -> Result<i64, VaultError> {
        self.connection
            .execute(
                "INSERT INTO lab_reports (title, observed_at) VALUES (?1, ?2)",
                params![report.title, report.observed_at],
            )
            .map_err(|_| VaultError::Operation)?;
        Ok(self.connection.last_insert_rowid())
    }

    pub fn list_lab_reports(&self) -> Result<Vec<LabReport>, VaultError> {
        let mut statement = self
            .connection
            .prepare("SELECT id, title, observed_at FROM lab_reports ORDER BY id")
            .map_err(|_| VaultError::InvalidKeyOrVault)?;
        let rows = statement
            .query_map([], |row| {
                Ok(LabReport {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    observed_at: row.get(2)?,
                })
            })
            .map_err(|_| VaultError::InvalidKeyOrVault)?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|_| VaultError::InvalidKeyOrVault)
    }

    pub fn schema_version(&self) -> Result<i64, VaultError> {
        self.connection
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .map_err(|_| VaultError::InvalidKeyOrVault)
    }

    pub fn temporary_store(&self) -> Result<String, VaultError> {
        let value: i64 = self
            .connection
            .pragma_query_value(None, "temp_store", |row| row.get(0))
            .map_err(|_| VaultError::Operation)?;
        match value {
            2 => Ok("memory".to_owned()),
            _ => Err(VaultError::Operation),
        }
    }

    pub fn integrity_check(&self) -> Result<(), VaultError> {
        let cipher_errors: Vec<String> = self
            .connection
            .prepare("PRAGMA cipher_integrity_check")
            .and_then(|mut statement| {
                statement
                    .query_map([], |row| row.get(0))?
                    .collect::<Result<Vec<_>, _>>()
            })
            .map_err(|_| VaultError::InvalidKeyOrVault)?;
        let sqlite_result: String = self
            .connection
            .pragma_query_value(None, "integrity_check", |row| row.get(0))
            .map_err(|_| VaultError::InvalidKeyOrVault)?;
        if cipher_errors.is_empty() && sqlite_result == "ok" {
            Ok(())
        } else {
            Err(VaultError::InvalidKeyOrVault)
        }
    }

    pub fn checkpoint(&self) -> Result<(), VaultError> {
        self.connection
            .execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
            .map_err(|_| VaultError::Operation)
    }

    #[cfg(feature = "proofs")]
    fn begin_rollback_journal_probe(&self, source_text: &str) -> Result<(), VaultError> {
        self.add_lab_report(&LabReportDraft {
            title: source_text.to_owned(),
            observed_at: "journal-probe".to_owned(),
        })?;
        self.checkpoint()?;
        self.connection
            .execute_batch("PRAGMA journal_mode = DELETE; BEGIN IMMEDIATE;")
            .map_err(|_| VaultError::Operation)?;
        self.connection
            .execute(
                "UPDATE lab_reports SET title = 'changed' WHERE title = ?1",
                [source_text],
            )
            .map_err(|_| VaultError::Operation)?;
        Ok(())
    }

    #[cfg(feature = "proofs")]
    fn finish_rollback_journal_probe(&self) -> Result<(), VaultError> {
        self.connection
            .execute_batch("ROLLBACK; PRAGMA journal_mode = WAL;")
            .map_err(|_| VaultError::Operation)
    }

    fn back_up_to(&self, destination: impl AsRef<Path>) -> Result<(), VaultError> {
        self.checkpoint()?;
        fs::copy(&self.path, destination).map_err(|_| VaultError::Operation)?;
        Ok(())
    }

    fn restore_from(
        backup: impl AsRef<Path>,
        destination: impl AsRef<Path>,
        key: &VaultKey,
    ) -> Result<(), VaultError> {
        let candidate = Self::open(backup.as_ref(), key)?;
        candidate.integrity_check()?;
        candidate.checkpoint()?;
        drop(candidate);
        fs::copy(backup, &destination).map_err(|_| VaultError::Operation)?;
        let restored = Self::open(destination, key)?;
        restored.integrity_check()
    }

    #[cfg(feature = "proofs")]
    fn run_crash_write_probe(
        &self,
        executable: impl AsRef<Path>,
        source_text: &str,
    ) -> Result<ExitStatus, VaultError> {
        let mut child = Command::new(executable.as_ref())
            .arg("crash-write")
            .arg(&self.path)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| VaultError::Operation)?;
        let stdin = child.stdin.as_mut().ok_or(VaultError::Operation)?;
        stdin
            .write_all(&self.key.0)
            .and_then(|_| stdin.write_all(source_text.as_bytes()))
            .map_err(|_| VaultError::Operation)?;
        child.wait().map_err(|_| VaultError::Operation)
    }

    #[cfg(feature = "proofs")]
    fn write_uncommitted_for_crash_probe(&self, source_text: &str) -> Result<(), VaultError> {
        self.connection
            .execute_batch("BEGIN IMMEDIATE")
            .map_err(|_| VaultError::Operation)?;
        self.add_lab_report(&LabReportDraft {
            title: source_text.to_owned(),
            observed_at: "crash-probe".to_owned(),
        })?;
        Ok(())
    }
}

pub struct NativeVaultManager;

impl NativeVaultManager {
    pub fn open(path: impl AsRef<Path>, key: &VaultKey) -> Result<AccountVault, VaultError> {
        AccountVault::open(path, key)
    }

    pub fn back_up(vault: &AccountVault, destination: impl AsRef<Path>) -> Result<(), VaultError> {
        vault.back_up_to(destination)
    }

    pub fn restore(
        backup: impl AsRef<Path>,
        destination: impl AsRef<Path>,
        key: &VaultKey,
    ) -> Result<(), VaultError> {
        AccountVault::restore_from(backup, destination, key)
    }
}

pub enum VaultCommand {
    AddLabReport(LabReportDraft),
    ListLabReports,
}

pub enum VaultCommandResult {
    LabReportAdded { id: i64 },
    LabReports(Vec<LabReport>),
}

pub struct VaultCommandFacade {
    vault: AccountVault,
}

impl VaultCommandFacade {
    pub fn new(vault: AccountVault) -> Self {
        Self { vault }
    }

    pub fn execute(&self, command: VaultCommand) -> Result<VaultCommandResult, VaultError> {
        match command {
            VaultCommand::AddLabReport(report) => Ok(VaultCommandResult::LabReportAdded {
                id: self.vault.add_lab_report(&report)?,
            }),
            VaultCommand::ListLabReports => Ok(VaultCommandResult::LabReports(
                self.vault.list_lab_reports()?,
            )),
        }
    }
}

#[cfg(feature = "proofs")]
pub struct VaultProbe;

#[cfg(feature = "proofs")]
impl VaultProbe {
    pub fn begin_rollback_journal(
        vault: &AccountVault,
        source_text: &str,
    ) -> Result<(), VaultError> {
        vault.begin_rollback_journal_probe(source_text)
    }

    pub fn finish_rollback_journal(vault: &AccountVault) -> Result<(), VaultError> {
        vault.finish_rollback_journal_probe()
    }

    pub fn run_crash_write(
        vault: &AccountVault,
        executable: impl AsRef<Path>,
        source_text: &str,
    ) -> Result<ExitStatus, VaultError> {
        vault.run_crash_write_probe(executable, source_text)
    }

    pub fn write_uncommitted(vault: &AccountVault, source_text: &str) -> Result<(), VaultError> {
        vault.write_uncommitted_for_crash_probe(source_text)
    }
}

fn apply_key(connection: &Connection, key: &VaultKey) -> Result<(), VaultError> {
    let key_literal = format!("x'{}'", encode_hex(&key.0));
    connection
        .pragma_update(None, "key", key_literal)
        .map_err(|_| VaultError::InvalidKeyOrVault)?;
    let version: String = connection
        .pragma_query_value(None, "cipher_version", |row| row.get(0))
        .map_err(|_| VaultError::InvalidKeyOrVault)?;
    if version.is_empty() {
        return Err(VaultError::InvalidKeyOrVault);
    }
    Ok(())
}

fn configure(connection: &Connection) -> Result<(), VaultError> {
    connection
        .execute_batch(
            "PRAGMA temp_store = MEMORY;
             PRAGMA secure_delete = ON;
             PRAGMA journal_mode = WAL;
             PRAGMA synchronous = FULL;
             PRAGMA foreign_keys = ON;",
        )
        .map_err(|_| VaultError::InvalidKeyOrVault)
}

fn migrate(connection: &Connection) -> Result<(), VaultError> {
    let version: i64 = connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(|_| VaultError::InvalidKeyOrVault)?;
    if version == 0 {
        connection
            .execute_batch(
                "BEGIN IMMEDIATE;
                 CREATE TABLE lab_reports (
                     id INTEGER PRIMARY KEY,
                     title TEXT NOT NULL,
                     observed_at TEXT NOT NULL
                 );
                 PRAGMA user_version = 1;
                 COMMIT;",
            )
            .map_err(|_| VaultError::InvalidKeyOrVault)?;
    } else if version != SCHEMA_VERSION {
        return Err(VaultError::InvalidKeyOrVault);
    }
    Ok(())
}

fn encode_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
