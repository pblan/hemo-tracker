use rusqlite::{Connection, OpenFlags, OptionalExtension, params};
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

const SCHEMA_VERSION: i64 = 5;
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CreateReport {
    pub collection_time: String,
    pub report_date: Option<String>,
    pub laboratory: Option<String>,
    pub ordering_clinician: Option<String>,
    pub fasting_state: Option<String>,
    pub notes: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceFileRecord {
    pub id: String,
    pub original_filename: String,
    pub media_type: String,
    pub role: String,
    pub opaque_object_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MeasurementRecord {
    pub id: String,
    pub source_label: String,
    pub source_value: String,
    pub source_unit: String,
    pub source_reference_interval: String,
    pub source_flag: String,
    pub analyte_id: Option<String>,
    pub updated_at: String,
    pub updated_by: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnalyteDefinition {
    pub id: String,
    pub name: String,
    pub component: String,
    pub property: String,
    pub specimen: String,
    pub scale: String,
    pub method: Option<String>,
    pub aliases: Vec<String>,
    pub loinc_code: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReportState {
    Draft,
    Complete,
    Archived,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReportRecord {
    pub id: String,
    pub collection_time: String,
    pub report_date: Option<String>,
    pub laboratory: Option<String>,
    pub ordering_clinician: Option<String>,
    pub fasting_state: Option<String>,
    pub notes: Option<String>,
    pub tags: Vec<String>,
    pub state: ReportState,
    pub source_files: Vec<SourceFileRecord>,
    pub measurements: Vec<MeasurementRecord>,
}

pub struct AccountVault {
    connection: Connection,
    path: PathBuf,
    #[cfg(feature = "proofs")]
    key: VaultKey,
}

impl AccountVault {
    pub fn list_report_ids(&self) -> Result<Vec<String>, VaultError> {
        let mut statement = self.connection.prepare("SELECT id FROM reports ORDER BY collection_time DESC, rowid DESC").map_err(|_| VaultError::Operation)?;
        statement.query_map([], |row| row.get(0)).map_err(|_| VaultError::Operation)?.collect::<Result<Vec<String>, _>>().map_err(|_| VaultError::Operation)
    }
    pub fn archive_report(&self, report_id: &str) -> Result<(), VaultError> {
        self.connection.execute("INSERT OR IGNORE INTO archived_reports (report_id, archived_at) VALUES (?1, datetime('now'))", [report_id]).map_err(|_| VaultError::Operation)?;
        Ok(())
    }
    pub fn upsert_analyte(&self, analyte: &AnalyteDefinition) -> Result<(), VaultError> {
        let aliases = serde_json::to_string(&analyte.aliases).map_err(|_| VaultError::Operation)?;
        self.connection.execute("INSERT INTO analyte_definitions (id,name,component,property,specimen,scale,method,aliases_json,loinc_code) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9) ON CONFLICT(id) DO UPDATE SET name=excluded.name,component=excluded.component,property=excluded.property,specimen=excluded.specimen,scale=excluded.scale,method=excluded.method,aliases_json=excluded.aliases_json,loinc_code=excluded.loinc_code", params![analyte.id, analyte.name, analyte.component, analyte.property, analyte.specimen, analyte.scale, analyte.method, aliases, analyte.loinc_code]).map_err(|_| VaultError::Operation)?;
        Ok(())
    }

    pub fn list_analytes(&self) -> Result<Vec<AnalyteDefinition>, VaultError> {
        let mut statement = self.connection.prepare("SELECT id,name,component,property,specimen,scale,method,aliases_json,loinc_code FROM analyte_definitions ORDER BY name").map_err(|_| VaultError::Operation)?;
        statement.query_map([], |row| Ok(AnalyteDefinition { id: row.get(0)?, name: row.get(1)?, component: row.get(2)?, property: row.get(3)?, specimen: row.get(4)?, scale: row.get(5)?, method: row.get(6)?, aliases: serde_json::from_str(&row.get::<_, String>(7)?).unwrap_or_default(), loinc_code: row.get(8)? })).map_err(|_| VaultError::Operation)?.collect::<Result<Vec<_>, _>>().map_err(|_| VaultError::Operation)
    }

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

    pub fn create_report(&self, report: &CreateReport) -> Result<String, VaultError> {
        let id = random_identifier()?;
        let tags = serde_json::to_string(&report.tags).map_err(|_| VaultError::Operation)?;
        self.connection
            .execute(
                "INSERT INTO reports (
                    id, collection_time, report_date, laboratory, ordering_clinician,
                    fasting_state, notes, tags_json, state
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'draft')",
                params![
                    id,
                    report.collection_time,
                    report.report_date,
                    report.laboratory,
                    report.ordering_clinician,
                    report.fasting_state,
                    report.notes,
                    tags,
                ],
            )
            .map_err(|_| VaultError::Operation)?;
        Ok(id)
    }

    pub fn add_source_file_record(
        &self,
        report_id: &str,
        source: &SourceFileRecord,
    ) -> Result<(), VaultError> {
        self.connection
            .execute(
                "INSERT INTO source_files (
                    id, report_id, original_filename, media_type, role, opaque_object_id
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    source.id,
                    report_id,
                    source.original_filename,
                    source.media_type,
                    source.role,
                    source.opaque_object_id,
                ],
            )
            .map_err(|_| VaultError::Operation)?;
        Ok(())
    }

    pub fn add_measurement_record(
        &self,
        report_id: &str,
        measurement: &MeasurementRecord,
    ) -> Result<(), VaultError> {
        self.connection
            .execute(
                "INSERT INTO measurements (
                    id, report_id, source_label, source_value, source_unit,
                    source_reference_interval, source_flag, analyte_id, updated_at, updated_by
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, datetime('now'), 'local-user')",
                params![
                    measurement.id,
                    report_id,
                    measurement.source_label,
                    measurement.source_value,
                    measurement.source_unit,
                    measurement.source_reference_interval,
                    measurement.source_flag,
                    measurement.analyte_id,
                ],
            )
            .map_err(|_| VaultError::Operation)?;
        Ok(())
    }

    pub fn complete_report(&self, report_id: &str) -> Result<(), VaultError> {
        let changed = self
            .connection
            .execute(
                "UPDATE reports SET state = 'complete' WHERE id = ?1 AND state = 'draft'",
                [report_id],
            )
            .map_err(|_| VaultError::Operation)?;
        if changed == 1 {
            Ok(())
        } else {
            Err(VaultError::Operation)
        }
    }

    pub fn get_report(&self, report_id: &str) -> Result<ReportRecord, VaultError> {
        let report = self
            .connection
            .query_row(
                "SELECT collection_time, report_date, laboratory, ordering_clinician,
                        fasting_state, notes, tags_json, state
                 FROM reports WHERE id = ?1",
                [report_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, Option<String>>(1)?,
                        row.get::<_, Option<String>>(2)?,
                        row.get::<_, Option<String>>(3)?,
                        row.get::<_, Option<String>>(4)?,
                        row.get::<_, Option<String>>(5)?,
                        row.get::<_, String>(6)?,
                        row.get::<_, String>(7)?,
                    ))
                },
            )
            .optional()
            .map_err(|_| VaultError::Operation)?
            .ok_or(VaultError::Operation)?;
        let tags = serde_json::from_str(&report.6).map_err(|_| VaultError::Operation)?;
        let archived: bool = self.connection.query_row("SELECT EXISTS (SELECT 1 FROM archived_reports WHERE report_id = ?1)", [report_id], |row| row.get(0)).map_err(|_| VaultError::Operation)?;
        let state = if archived { ReportState::Archived } else { match report.7.as_str() {
            "draft" => ReportState::Draft,
            "complete" => ReportState::Complete,
            _ => return Err(VaultError::Operation),
        }};
        Ok(ReportRecord {
            id: report_id.to_owned(),
            collection_time: report.0,
            report_date: report.1,
            laboratory: report.2,
            ordering_clinician: report.3,
            fasting_state: report.4,
            notes: report.5,
            tags,
            state,
            source_files: self.source_files(report_id)?,
            measurements: self.measurements(report_id)?,
        })
    }

    fn source_files(&self, report_id: &str) -> Result<Vec<SourceFileRecord>, VaultError> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT id, original_filename, media_type, role, opaque_object_id
             FROM source_files WHERE report_id = ?1 ORDER BY rowid",
            )
            .map_err(|_| VaultError::Operation)?;
        statement
            .query_map([report_id], |row| {
                Ok(SourceFileRecord {
                    id: row.get(0)?,
                    original_filename: row.get(1)?,
                    media_type: row.get(2)?,
                    role: row.get(3)?,
                    opaque_object_id: row.get(4)?,
                })
            })
            .map_err(|_| VaultError::Operation)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| VaultError::Operation)
    }

    fn measurements(&self, report_id: &str) -> Result<Vec<MeasurementRecord>, VaultError> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT id, source_label, source_value, source_unit,
                    source_reference_interval, source_flag, analyte_id, updated_at, updated_by
             FROM measurements WHERE report_id = ?1 ORDER BY rowid",
            )
            .map_err(|_| VaultError::Operation)?;
        statement
            .query_map([report_id], |row| {
                Ok(MeasurementRecord {
                    id: row.get(0)?,
                    source_label: row.get(1)?,
                    source_value: row.get(2)?,
                    source_unit: row.get(3)?,
                    source_reference_interval: row.get(4)?,
                    source_flag: row.get(5)?,
                    analyte_id: row.get(6)?,
                    updated_at: row.get(7)?,
                    updated_by: row.get(8)?,
                })
            })
            .map_err(|_| VaultError::Operation)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| VaultError::Operation)
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
                 CREATE TABLE reports (
                     id TEXT PRIMARY KEY,
                     collection_time TEXT NOT NULL,
                     report_date TEXT,
                     laboratory TEXT,
                     ordering_clinician TEXT,
                     fasting_state TEXT,
                     notes TEXT,
                     tags_json TEXT NOT NULL,
                     state TEXT NOT NULL CHECK (state IN ('draft', 'complete'))
                 );
                 CREATE TABLE source_files (
                     id TEXT PRIMARY KEY,
                     report_id TEXT NOT NULL REFERENCES reports(id) ON DELETE CASCADE,
                     original_filename TEXT NOT NULL,
                     media_type TEXT NOT NULL,
                     role TEXT NOT NULL,
                     opaque_object_id TEXT NOT NULL UNIQUE
                 );
                 CREATE TABLE analyte_definitions (
                     id TEXT PRIMARY KEY, name TEXT NOT NULL, component TEXT NOT NULL,
                     property TEXT NOT NULL, specimen TEXT NOT NULL, scale TEXT NOT NULL,
                     method TEXT, aliases_json TEXT NOT NULL, loinc_code TEXT
                 );
                 CREATE TABLE measurements (
                     id TEXT PRIMARY KEY,
                     report_id TEXT NOT NULL REFERENCES reports(id) ON DELETE CASCADE,
                     source_label TEXT NOT NULL,
                     source_value TEXT NOT NULL,
                     source_unit TEXT NOT NULL,
                     source_reference_interval TEXT NOT NULL,
                     source_flag TEXT NOT NULL,
                     analyte_id TEXT, updated_at TEXT NOT NULL, updated_by TEXT NOT NULL
                 );
                 CREATE TABLE archived_reports (report_id TEXT PRIMARY KEY REFERENCES reports(id) ON DELETE CASCADE, archived_at TEXT NOT NULL);
                 PRAGMA user_version = 5;
                 COMMIT;",
            )
            .map_err(|_| VaultError::InvalidKeyOrVault)?;
    } else if version == 1 {
        connection
            .execute_batch(
                "BEGIN IMMEDIATE;
                 CREATE TABLE reports (
                     id TEXT PRIMARY KEY,
                     collection_time TEXT NOT NULL,
                     report_date TEXT,
                     laboratory TEXT,
                     ordering_clinician TEXT,
                     fasting_state TEXT,
                     notes TEXT,
                     tags_json TEXT NOT NULL,
                     state TEXT NOT NULL CHECK (state IN ('draft', 'complete'))
                 );
                 CREATE TABLE source_files (
                     id TEXT PRIMARY KEY,
                     report_id TEXT NOT NULL REFERENCES reports(id) ON DELETE CASCADE,
                     original_filename TEXT NOT NULL,
                     media_type TEXT NOT NULL,
                     role TEXT NOT NULL,
                     opaque_object_id TEXT NOT NULL UNIQUE
                 );
                 CREATE TABLE analyte_definitions (
                     id TEXT PRIMARY KEY, name TEXT NOT NULL, component TEXT NOT NULL,
                     property TEXT NOT NULL, specimen TEXT NOT NULL, scale TEXT NOT NULL,
                     method TEXT, aliases_json TEXT NOT NULL, loinc_code TEXT
                 );
                 CREATE TABLE measurements (
                     id TEXT PRIMARY KEY,
                     report_id TEXT NOT NULL REFERENCES reports(id) ON DELETE CASCADE,
                     source_label TEXT NOT NULL,
                     source_value TEXT NOT NULL,
                     source_unit TEXT NOT NULL,
                     source_reference_interval TEXT NOT NULL,
                     source_flag TEXT NOT NULL,
                     analyte_id TEXT, updated_at TEXT NOT NULL, updated_by TEXT NOT NULL
                 );
                 CREATE TABLE archived_reports (report_id TEXT PRIMARY KEY REFERENCES reports(id) ON DELETE CASCADE, archived_at TEXT NOT NULL);
                 PRAGMA user_version = 5;
                 COMMIT;",
            )
            .map_err(|_| VaultError::InvalidKeyOrVault)?;
    } else if version == 2 {
        connection
            .execute_batch("BEGIN IMMEDIATE; ALTER TABLE measurements ADD COLUMN analyte_id TEXT; CREATE TABLE analyte_definitions (id TEXT PRIMARY KEY, name TEXT NOT NULL, component TEXT NOT NULL, property TEXT NOT NULL, specimen TEXT NOT NULL, scale TEXT NOT NULL, method TEXT, aliases_json TEXT NOT NULL, loinc_code TEXT); PRAGMA user_version = 3; COMMIT;")
            .map_err(|_| VaultError::InvalidKeyOrVault)?;
    } else if version == 3 {
        connection.execute_batch("BEGIN IMMEDIATE; CREATE TABLE archived_reports (report_id TEXT PRIMARY KEY REFERENCES reports(id) ON DELETE CASCADE, archived_at TEXT NOT NULL); PRAGMA user_version = 4; COMMIT;").map_err(|_| VaultError::InvalidKeyOrVault)?;
    } else if version == 4 {
        connection.execute_batch("BEGIN IMMEDIATE; ALTER TABLE measurements ADD COLUMN updated_at TEXT NOT NULL DEFAULT ''; ALTER TABLE measurements ADD COLUMN updated_by TEXT NOT NULL DEFAULT 'local-user'; PRAGMA user_version = 5; COMMIT;").map_err(|_| VaultError::InvalidKeyOrVault)?;
    } else if version != SCHEMA_VERSION {
        return Err(VaultError::InvalidKeyOrVault);
    }
    Ok(())
}

fn encode_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn random_identifier() -> Result<String, VaultError> {
    let mut bytes = [0_u8; 16];
    getrandom::fill(&mut bytes).map_err(|_| VaultError::SecureRandom)?;
    Ok(encode_hex(&bytes))
}
