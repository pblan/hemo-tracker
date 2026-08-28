use hemo_encrypted_vault_proof::{AccountVault, NativeVaultManager, VaultKey};
use hemo_key_lifecycle_proof::{
    AccountKeyBundle, KeyEnvelope, Passphrase, Purpose, RecoveryCode, RecoveryKey, UnlockedKeys,
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
}

impl LocalAccountVault {
    pub fn create(
        directory: impl AsRef<Path>,
        input: CreateLocalAccount,
    ) -> Result<CreatedLocalAccount, LocalAccountError> {
        if input.account_id.is_empty() || input.account_id.contains('\0') {
            return Err(LocalAccountError::InvalidAccount);
        }

        let directory = directory.as_ref().to_owned();
        fs::create_dir_all(&directory).map_err(|_| LocalAccountError::Operation)?;
        let manifest_path = directory.join(MANIFEST_NAME);
        if manifest_path.exists() {
            return Err(LocalAccountError::AlreadyExists);
        }

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
        let vault = open_encrypted_vault(&directory, bundle.unlocked_keys())?;
        write_manifest(&manifest_path, &manifest)?;
        let recovery_code = bundle.recovery_key().to_code();

        Ok(CreatedLocalAccount {
            vault: Self {
                directory,
                manifest,
                unlocked: Some(UnlockedAccount {
                    _keys: bundle.unlocked_keys().clone(),
                    _vault: vault,
                }),
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
        let vault = open_encrypted_vault(&self.directory, &keys)
            .map_err(|_| LocalAccountError::InvalidCredentials)?;
        self.unlocked = Some(UnlockedAccount {
            _keys: keys,
            _vault: vault,
        });
        Ok(())
    }
}

fn open_encrypted_vault(
    directory: &Path,
    keys: &UnlockedKeys,
) -> Result<AccountVault, LocalAccountError> {
    let purpose_key = keys.derive_purpose_key(Purpose::Database, 0);
    let key = VaultKey::from_bytes(*purpose_key.bytes());
    NativeVaultManager::open(directory.join(VAULT_NAME), &key)
        .map_err(|_| LocalAccountError::InvalidCredentials)
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
