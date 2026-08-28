use hemo_encrypted_vault::{
    AccountVault, AnalyteDefinition, CreateReport, MeasurementRecord, NativeVaultManager,
    ReportState, SourceFileRecord, VaultKey,
};
use hemo_key_lifecycle::{
    AccountKeyBundle, KeyEnvelope, Passphrase, Purpose, PurposeKey, RecoveryCode, RecoveryKey,
    UnlockedKeys,
};
use hemo_source_file_encryption::{
    SourceFileContext, SourceFileKey, SourceFileMetadata, encrypt_source_file,
    generate_opaque_object_id,
};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CreateLabReportDraft {
    pub collection_time: String,
    pub report_date: Option<String>,
    pub laboratory: Option<String>,
    pub ordering_clinician: Option<String>,
    pub fasting_state: Option<String>,
    pub notes: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewSourceFile {
    pub original_filename: String,
    pub media_type: String,
    pub role: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewMeasurement {
    pub source_label: String,
    pub source_value: String,
    pub source_unit: String,
    pub source_reference_interval: String,
    pub source_flag: String,
    pub analyte_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewAnalyte {
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
pub enum ReportStatus {
    Draft,
    Complete,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LabReportSourceFile {
    pub id: String,
    pub original_filename: String,
    pub media_type: String,
    pub role: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LabReportMeasurement {
    pub id: String,
    pub source_label: String,
    pub source_value: String,
    pub source_unit: String,
    pub source_reference_interval: String,
    pub source_flag: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LabReportDetails {
    pub id: String,
    pub collection_time: String,
    pub report_date: Option<String>,
    pub laboratory: Option<String>,
    pub ordering_clinician: Option<String>,
    pub fasting_state: Option<String>,
    pub notes: Option<String>,
    pub tags: Vec<String>,
    pub status: ReportStatus,
    pub source_files: Vec<LabReportSourceFile>,
    pub measurements: Vec<LabReportMeasurement>,
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
    source_file_key: PurposeKey,
}

impl LocalAccountVault {
    pub fn add_analyte(&mut self, analyte: NewAnalyte) -> Result<String, LocalAccountError> {
        if analyte.name.trim().is_empty()
            || analyte.component.trim().is_empty()
            || analyte.property.trim().is_empty()
            || analyte.specimen.trim().is_empty()
            || analyte.scale.trim().is_empty()
        {
            return Err(LocalAccountError::Operation);
        }
        let id = random_identifier()?;
        self.unlocked_mut()?
            ._vault
            .upsert_analyte(&AnalyteDefinition {
                id: id.clone(),
                name: analyte.name,
                component: analyte.component,
                property: analyte.property,
                specimen: analyte.specimen,
                scale: analyte.scale,
                method: analyte.method,
                aliases: analyte.aliases,
                loinc_code: analyte.loinc_code,
            })
            .map_err(|_| LocalAccountError::Operation)?;
        Ok(id)
    }

    pub fn list_analytes(&self) -> Result<Vec<AnalyteDefinition>, LocalAccountError> {
        self.unlocked_ref()?
            ._vault
            .list_analytes()
            .map_err(|_| LocalAccountError::Operation)
    }
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

    pub fn create_lab_report_draft(
        &mut self,
        report: CreateLabReportDraft,
    ) -> Result<String, LocalAccountError> {
        if report.collection_time.trim().is_empty() {
            return Err(LocalAccountError::Operation);
        }
        self.unlocked_mut()?
            ._vault
            .create_report(&CreateReport {
                collection_time: report.collection_time,
                report_date: report.report_date,
                laboratory: report.laboratory,
                ordering_clinician: report.ordering_clinician,
                fasting_state: report.fasting_state,
                notes: report.notes,
                tags: report.tags,
            })
            .map_err(|_| LocalAccountError::Operation)
    }

    pub fn add_source_file(
        &mut self,
        report_id: &str,
        source: impl Read,
        metadata: NewSourceFile,
    ) -> Result<String, LocalAccountError> {
        if !matches!(
            metadata.role.as_str(),
            "primary" | "supplement" | "correction"
        ) {
            return Err(LocalAccountError::Operation);
        }
        let object_id = generate_opaque_object_id().map_err(|_| LocalAccountError::Operation)?;
        let source_file_id = random_identifier()?;
        let account_id = self.manifest.account_id.clone();
        let objects = self.directory.join("objects");
        let unlocked = self.unlocked_mut()?;
        let context = SourceFileContext::new(account_id, object_id.clone());
        let source_key = SourceFileKey::from_bytes(*unlocked.source_file_key.bytes());
        let encrypted_path = encrypt_source_file(
            source,
            &objects,
            &context,
            &source_key,
            &SourceFileMetadata {
                original_filename: metadata.original_filename.clone(),
                media_type: metadata.media_type.clone(),
            },
        )
        .map_err(|_| LocalAccountError::Operation)?;
        let record = SourceFileRecord {
            id: source_file_id.clone(),
            original_filename: metadata.original_filename,
            media_type: metadata.media_type,
            role: metadata.role,
            opaque_object_id: object_id.to_string(),
        };
        if unlocked
            ._vault
            .add_source_file_record(report_id, &record)
            .is_err()
        {
            let _ = fs::remove_file(encrypted_path);
            return Err(LocalAccountError::Operation);
        }
        Ok(source_file_id)
    }

    pub fn add_measurement(
        &mut self,
        report_id: &str,
        measurement: NewMeasurement,
    ) -> Result<String, LocalAccountError> {
        let id = random_identifier()?;
        self.unlocked_mut()?
            ._vault
            .add_measurement_record(
                report_id,
                &MeasurementRecord {
                    id: id.clone(),
                    source_label: measurement.source_label,
                    source_value: measurement.source_value,
                    source_unit: measurement.source_unit,
                    source_reference_interval: measurement.source_reference_interval,
                    source_flag: measurement.source_flag,
                    analyte_id: measurement.analyte_id,
                },
            )
            .map_err(|_| LocalAccountError::Operation)?;
        Ok(id)
    }

    pub fn complete_lab_report(&mut self, report_id: &str) -> Result<(), LocalAccountError> {
        let report = self.get_lab_report(report_id)?;
        if report.source_files.is_empty() || report.measurements.is_empty() {
            return Err(LocalAccountError::Operation);
        }
        self.unlocked_mut()?
            ._vault
            .complete_report(report_id)
            .map_err(|_| LocalAccountError::Operation)
    }

    pub fn get_lab_report(&self, report_id: &str) -> Result<LabReportDetails, LocalAccountError> {
        let report = self
            .unlocked_ref()?
            ._vault
            .get_report(report_id)
            .map_err(|_| LocalAccountError::Operation)?;
        Ok(LabReportDetails {
            id: report.id,
            collection_time: report.collection_time,
            report_date: report.report_date,
            laboratory: report.laboratory,
            ordering_clinician: report.ordering_clinician,
            fasting_state: report.fasting_state,
            notes: report.notes,
            tags: report.tags,
            status: match report.state {
                ReportState::Draft => ReportStatus::Draft,
                ReportState::Complete => ReportStatus::Complete,
            },
            source_files: report
                .source_files
                .into_iter()
                .map(|source| LabReportSourceFile {
                    id: source.id,
                    original_filename: source.original_filename,
                    media_type: source.media_type,
                    role: source.role,
                })
                .collect(),
            measurements: report
                .measurements
                .into_iter()
                .map(|measurement| LabReportMeasurement {
                    id: measurement.id,
                    source_label: measurement.source_label,
                    source_value: measurement.source_value,
                    source_unit: measurement.source_unit,
                    source_reference_interval: measurement.source_reference_interval,
                    source_flag: measurement.source_flag,
                })
                .collect(),
        })
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

    fn unlocked_ref(&self) -> Result<&UnlockedAccount, LocalAccountError> {
        self.unlocked
            .as_ref()
            .ok_or(LocalAccountError::InvalidCredentials)
    }

    fn unlocked_mut(&mut self) -> Result<&mut UnlockedAccount, LocalAccountError> {
        self.unlocked
            .as_mut()
            .ok_or(LocalAccountError::InvalidCredentials)
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
        source_file_key,
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

fn random_identifier() -> Result<String, LocalAccountError> {
    let mut bytes = [0_u8; 16];
    getrandom::fill(&mut bytes).map_err(|_| LocalAccountError::Operation)?;
    Ok(bytes.iter().map(|byte| format!("{byte:02x}")).collect())
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
