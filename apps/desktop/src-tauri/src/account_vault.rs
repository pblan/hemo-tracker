use hemo_encrypted_vault::{AccountVault, NativeVaultManager, VaultKey};
use hemo_key_lifecycle::{
    AccountKeyBundle, KeyEnvelope, Passphrase, Purpose, PurposeKey, RecoveryCode, RecoveryKey,
    UnlockedKeys,
};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};
use thiserror::Error;
use zeroize::Zeroize;

const MANIFEST_NAME: &str = "account.json";
const VAULT_NAME: &str = "vault.db";
const FORMAT: &str = "hemo-tracker-local-account";
const VERSION: u8 = 1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CreateLocalAccount {
    pub account_id: String,
    pub passphrase: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VaultStatus {
    Locked,
    Unlocked,
}

#[derive(Debug, Error)]
pub enum LocalAccountError {
    #[error("the local account already exists")]
    AlreadyExists,
    #[error("the local account is invalid or damaged")]
    InvalidAccount,
    #[error("the credentials or local account are invalid")]
    InvalidCredentials,
    #[error("the local account operation failed")]
    Operation,
}

#[derive(Serialize, Deserialize)]
struct AccountManifest {
    format: String,
    version: u8,
    account_id: String,
    passphrase_envelope: String,
    recovery_envelope: String,
}

pub struct CreatedLocalAccount {
    vault: LocalAccountVault,
    recovery_code: RecoveryCode,
}

impl CreatedLocalAccount {
    pub fn recovery_code(&self) -> &str {
        self.recovery_code.expose()
    }

    pub fn into_vault(self) -> LocalAccountVault {
        self.vault
    }
}

pub struct LocalAccountVault {
    directory: PathBuf,
    manifest: AccountManifest,
    unlocked: Option<UnlockedAccount>,
}

struct UnlockedAccount {
    _keys: UnlockedKeys,
    _vault: AccountVault,
    _source_file_key: PurposeKey,
}

impl LocalAccountVault {
    pub fn create(
        directory: impl AsRef<Path>,
        input: CreateLocalAccount,
    ) -> Result<CreatedLocalAccount, LocalAccountError> {
        Self::create_with_manifest_writer(directory.as_ref(), input, write_manifest)
    }

    fn create_with_manifest_writer(
        directory: &Path,
        input: CreateLocalAccount,
        manifest_writer: impl FnOnce(&Path, &AccountManifest) -> Result<(), LocalAccountError>,
    ) -> Result<CreatedLocalAccount, LocalAccountError> {
        if input.account_id.is_empty() || input.account_id.contains('\0') {
            return Err(LocalAccountError::InvalidAccount);
        }

        let directory = directory.to_owned();
        if directory.exists() {
            return Err(LocalAccountError::AlreadyExists);
        }
        let parent = directory.parent().ok_or(LocalAccountError::Operation)?;
        fs::create_dir_all(parent).map_err(|_| LocalAccountError::Operation)?;
        let staging = staging_directory(&directory)?;
        fs::create_dir(&staging).map_err(|_| LocalAccountError::Operation)?;

        let mut published = false;
        let result = (|| {
            let mut passphrase_text = input.passphrase;
            let passphrase = Passphrase::new(&passphrase_text);
            passphrase_text.zeroize();
            let bundle = AccountKeyBundle::create(&input.account_id, &passphrase)
                .map_err(|_| LocalAccountError::Operation)?;
            let manifest = AccountManifest {
                format: FORMAT.to_owned(),
                version: VERSION,
                account_id: input.account_id,
                passphrase_envelope: bundle.passphrase_envelope().to_json(),
                recovery_envelope: bundle.recovery_envelope().to_json(),
            };
            let staged = unlock_account(&staging, bundle.unlocked_keys())?;
            staged
                ._vault
                .integrity_check()
                .map_err(|_| LocalAccountError::Operation)?;
            drop(staged);
            manifest_writer(&staging.join(MANIFEST_NAME), &manifest)?;
            sync_directory(&staging)?;
            fs::rename(&staging, &directory).map_err(|_| LocalAccountError::Operation)?;
            published = true;
            sync_directory(parent)?;
            let unlocked = unlock_account(&directory, bundle.unlocked_keys())?;
            Ok((bundle, manifest, unlocked))
        })();

        let (bundle, manifest, unlocked) = match result {
            Ok(value) => value,
            Err(error) => {
                let _ = fs::remove_dir_all(&staging);
                if published {
                    let _ = fs::remove_dir_all(&directory);
                    let _ = sync_directory(parent);
                }
                return Err(error);
            }
        };
        let recovery_code = bundle.recovery_key().to_code();

        Ok(CreatedLocalAccount {
            vault: Self {
                directory,
                manifest,
                unlocked: Some(unlocked),
            },
            recovery_code,
        })
    }

    pub fn open(directory: impl AsRef<Path>) -> Result<Self, LocalAccountError> {
        let directory = directory.as_ref().to_owned();
        let manifest_text = fs::read_to_string(directory.join(MANIFEST_NAME))
            .map_err(|_| LocalAccountError::InvalidAccount)?;
        let manifest: AccountManifest =
            serde_json::from_str(&manifest_text).map_err(|_| LocalAccountError::InvalidAccount)?;
        validate_manifest(&manifest)?;
        Ok(Self {
            directory,
            manifest,
            unlocked: None,
        })
    }

    pub fn status(&self) -> VaultStatus {
        if self.unlocked.is_some() {
            VaultStatus::Unlocked
        } else {
            VaultStatus::Locked
        }
    }

    pub fn lock(&mut self) {
        self.unlocked = None;
    }

    pub fn unlock_with_passphrase(
        &mut self,
        mut passphrase_text: String,
    ) -> Result<(), LocalAccountError> {
        let passphrase = Passphrase::new(&passphrase_text);
        passphrase_text.zeroize();
        let envelope = KeyEnvelope::from_json(&self.manifest.passphrase_envelope)
            .map_err(|_| LocalAccountError::InvalidCredentials)?;
        let keys = envelope
            .unlock_with_passphrase(&passphrase)
            .map_err(|_| LocalAccountError::InvalidCredentials)?;
        self.finish_unlock(keys)
    }

    pub fn unlock_with_recovery(
        &mut self,
        mut recovery_code: String,
    ) -> Result<(), LocalAccountError> {
        let recovery_key = RecoveryKey::from_code(&recovery_code);
        recovery_code.zeroize();
        let recovery_key = recovery_key.map_err(|_| LocalAccountError::InvalidCredentials)?;
        let envelope = KeyEnvelope::from_json(&self.manifest.recovery_envelope)
            .map_err(|_| LocalAccountError::InvalidCredentials)?;
        let keys = envelope
            .unlock_with_recovery(&recovery_key)
            .map_err(|_| LocalAccountError::InvalidCredentials)?;
        self.finish_unlock(keys)
    }

    fn finish_unlock(&mut self, keys: UnlockedKeys) -> Result<(), LocalAccountError> {
        let unlocked = unlock_account(&self.directory, &keys)
            .map_err(|_| LocalAccountError::InvalidCredentials)?;
        self.unlocked = Some(unlocked);
        Ok(())
    }
}

fn unlock_account(
    directory: &Path,
    keys: &UnlockedKeys,
) -> Result<UnlockedAccount, LocalAccountError> {
    let database_key = keys.derive_purpose_key(Purpose::Database, 0);
    let source_file_key = keys.derive_purpose_key(Purpose::SourceFiles, 0);
    let vault_key = VaultKey::from_bytes(*database_key.bytes());
    let vault = NativeVaultManager::open(directory.join(VAULT_NAME), &vault_key)
        .map_err(|_| LocalAccountError::InvalidCredentials)?;
    Ok(UnlockedAccount {
        _keys: keys.clone(),
        _vault: vault,
        _source_file_key: source_file_key,
    })
}

fn staging_directory(directory: &Path) -> Result<PathBuf, LocalAccountError> {
    let mut random = [0_u8; 8];
    getrandom::fill(&mut random).map_err(|_| LocalAccountError::Operation)?;
    let suffix = random
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let name = directory
        .file_name()
        .ok_or(LocalAccountError::Operation)?
        .to_string_lossy();
    Ok(directory.with_file_name(format!(".{name}.creating-{suffix}")))
}

fn validate_manifest(manifest: &AccountManifest) -> Result<(), LocalAccountError> {
    if manifest.format != FORMAT
        || manifest.version != VERSION
        || manifest.account_id.is_empty()
        || manifest.account_id.contains('\0')
    {
        return Err(LocalAccountError::InvalidAccount);
    }
    Ok(())
}

fn write_manifest(path: &Path, manifest: &AccountManifest) -> Result<(), LocalAccountError> {
    let partial = path.with_extension("partial");
    let bytes = serde_json::to_vec(manifest).map_err(|_| LocalAccountError::Operation)?;
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&partial)
        .map_err(|_| LocalAccountError::Operation)?;
    file.write_all(&bytes)
        .and_then(|_| file.sync_all())
        .map_err(|_| LocalAccountError::Operation)?;
    drop(file);
    fs::rename(&partial, path).map_err(|_| LocalAccountError::Operation)?;
    sync_directory(path.parent().ok_or(LocalAccountError::Operation)?)
}

fn sync_directory(directory: &Path) -> Result<(), LocalAccountError> {
    #[cfg(unix)]
    {
        File::open(directory)
            .and_then(|file| file.sync_all())
            .map_err(|_| LocalAccountError::Operation)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failed_creation_removes_staged_files_and_allows_retry() {
        let parent = tempfile::tempdir().unwrap();
        let directory = parent.path().join("account");
        let input = || CreateLocalAccount {
            account_id: "failure-retry".to_owned(),
            passphrase: "valid passphrase".to_owned(),
        };

        let failed = LocalAccountVault::create_with_manifest_writer(&directory, input(), |_, _| {
            Err(LocalAccountError::Operation)
        });

        assert!(failed.is_err());
        assert!(!directory.exists());
        assert!(LocalAccountVault::create(&directory, input()).is_ok());
    }
}
