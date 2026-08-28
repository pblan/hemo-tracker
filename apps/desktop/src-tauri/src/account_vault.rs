use hemo_encrypted_vault::{
    AccountVault, AnalyteDefinition, CreateReport, MeasurementRecord, NativeVaultManager,
    PersonalTargetRange, ReportState, SourceFileRecord, VaultKey,
};
use hemo_key_lifecycle::{
    AccountKeyBundle, KeyEnvelope, Passphrase, Purpose, PurposeKey, RecoveryCode, RecoveryKey,
    UnlockedKeys,
};
use hemo_source_file_encryption::{
    OpaqueObjectId, SourceFileContext, SourceFileKey, SourceFileMetadata, decrypt_source_file,
    encrypt_source_file, generate_opaque_object_id,
};
use serde::{Deserialize, Serialize};
use std::{
    cmp::min,
    fs::{self, File, OpenOptions},
    io::{self, Cursor, Read, Write},
    path::{Path, PathBuf},
};
use thiserror::Error;
use zeroize::{Zeroize, Zeroizing};

const MANIFEST_NAME: &str = "account.json";
const VAULT_NAME: &str = "vault.db";
const FORMAT: &str = "hemo-tracker-local-account";
const VERSION: u8 = 1;
const RESET_CONFIRMATION: &str = "RESET DEMO VAULT";
/// Keep interactive imports bounded while retaining streaming encryption.
const MAX_SOURCE_FILE_BYTES: u64 = 512 * 1024 * 1024;

struct SizeLimitedReader<R> {
    inner: R,
    remaining: u64,
}

impl<R> SizeLimitedReader<R> {
    fn new(inner: R, maximum: u64) -> Self {
        Self {
            inner,
            remaining: maximum,
        }
    }
}

impl<R: Read> Read for SizeLimitedReader<R> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if buffer.is_empty() {
            return Ok(0);
        }
        if self.remaining == 0 {
            let mut probe = [0_u8; 1];
            return match self.inner.read(&mut probe)? {
                0 => Ok(0),
                _ => Err(io::Error::new(
                    io::ErrorKind::FileTooLarge,
                    "source file exceeds the V1 size limit",
                )),
            };
        }
        let readable = min(self.remaining, buffer.len() as u64) as usize;
        let count = self.inner.read(&mut buffer[..readable])?;
        self.remaining -= count as u64;
        Ok(count)
    }
}

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
    pub parsed_numeric_value: Option<String>,
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
    pub canonical_unit: Option<String>,
    pub personal_target_ranges: Vec<NewPersonalTargetRange>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewPersonalTargetRange {
    pub lower_bound: Option<String>,
    pub upper_bound: Option<String>,
    pub unit: String,
    pub valid_from: Option<String>,
    pub valid_to: Option<String>,
    pub context: Option<String>,
    pub notes: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum ReportStatus {
    Draft,
    Complete,
    Archived,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct LabReportSourceFile {
    pub id: String,
    pub original_filename: String,
    pub media_type: String,
    pub role: String,
    pub opaque_object_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct LabReportMeasurement {
    pub id: String,
    pub source_label: String,
    pub source_value: String,
    pub source_unit: String,
    pub source_reference_interval: String,
    pub source_flag: String,
    pub parsed_numeric_value: Option<String>,
    pub analyte_id: Option<String>,
    pub updated_at: String,
    pub updated_by: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
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
    pub fn backup_to(&self, destination: impl AsRef<Path>) -> Result<(), LocalAccountError> {
        let destination = destination.as_ref();
        if destination.exists() {
            return Err(LocalAccountError::AlreadyExists);
        }
        if destination.starts_with(&self.directory) {
            return Err(LocalAccountError::Operation);
        }
        let parent = destination.parent().ok_or(LocalAccountError::Operation)?;
        fs::create_dir_all(parent).map_err(|_| LocalAccountError::Operation)?;
        let staging = destination.with_extension("backup-partial");
        if staging.exists() {
            let _ = fs::remove_dir_all(&staging);
        }
        copy_directory(&self.directory, &staging)?;
        sync_directory(&staging)?;
        fs::rename(&staging, destination).map_err(|_| LocalAccountError::Operation)?;
        sync_directory(parent)
    }

    pub fn export_plaintext_zip(
        &self,
        destination: impl AsRef<Path>,
    ) -> Result<(), LocalAccountError> {
        let destination = destination.as_ref();
        if destination.exists() {
            return Err(LocalAccountError::AlreadyExists);
        }
        let parent = destination
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        fs::create_dir_all(parent).map_err(|_| LocalAccountError::Operation)?;
        let staging = destination.with_extension("zip-partial");
        if staging.exists() {
            let _ = fs::remove_file(&staging);
        }

        let result = (|| {
            let file = File::create(&staging).map_err(|_| LocalAccountError::Operation)?;
            let mut archive = zip::ZipWriter::new(file);
            let options = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated);
            let reports = self.list_lab_report_ids()?;
            let unlocked = self.unlocked_ref()?;
            let source_key = SourceFileKey::from_bytes(*unlocked.source_file_key.bytes());
            let mut csv = String::from(
                "report_id,collection_time,laboratory,source_label,source_value,source_unit,source_reference_interval,source_flag\n",
            );
            for report_id in reports {
                let report = self.get_lab_report(&report_id)?;
                for measurement in &report.measurements {
                    let fields = [
                        report.id.as_str(),
                        report.collection_time.as_str(),
                        report.laboratory.as_deref().unwrap_or(""),
                        measurement.source_label.as_str(),
                        measurement.source_value.as_str(),
                        measurement.source_unit.as_str(),
                        measurement.source_reference_interval.as_str(),
                        measurement.source_flag.as_str(),
                    ];
                    csv.push_str(
                        &fields
                            .iter()
                            .map(|field| csv_field(field))
                            .collect::<Vec<_>>()
                            .join(","),
                    );
                    csv.push('\n');
                }
                archive
                    .start_file(format!("reports/{report_id}.json"), options)
                    .map_err(|_| LocalAccountError::Operation)?;
                let json =
                    serde_json::to_vec_pretty(&report).map_err(|_| LocalAccountError::Operation)?;
                archive
                    .write_all(&json)
                    .map_err(|_| LocalAccountError::Operation)?;
                for source in report.source_files {
                    let object = self
                        .directory
                        .join("objects")
                        .join(format!("{}.hemo", source.opaque_object_id));
                    let context = SourceFileContext::new(
                        self.manifest.account_id.clone(),
                        OpaqueObjectId::parse(source.opaque_object_id.clone())
                            .map_err(|_| LocalAccountError::Operation)?,
                    );
                    let filename = source.original_filename.replace(['/', '\\'], "_");
                    archive
                        .start_file(format!("sources/{report_id}-{filename}"), options)
                        .map_err(|_| LocalAccountError::Operation)?;
                    decrypt_source_file(object, &mut archive, &context, &source_key)
                        .map_err(|_| LocalAccountError::Operation)?;
                }
            }
            archive
                .start_file("measurements.csv", options)
                .map_err(|_| LocalAccountError::Operation)?;
            archive
                .write_all(csv.as_bytes())
                .map_err(|_| LocalAccountError::Operation)?;
            let file = archive.finish().map_err(|_| LocalAccountError::Operation)?;
            file.sync_all().map_err(|_| LocalAccountError::Operation)?;
            fs::rename(&staging, destination).map_err(|_| LocalAccountError::Operation)?;
            sync_directory(parent)
        })();
        if result.is_err() {
            let _ = fs::remove_file(&staging);
        }
        result
    }

    pub fn restore_from_backup(
        &mut self,
        backup: impl AsRef<Path>,
        passphrase_text: String,
    ) -> Result<(), LocalAccountError> {
        let backup = backup.as_ref();
        if backup == self.directory || !backup.is_dir() {
            return Err(LocalAccountError::Operation);
        }
        let parent = self
            .directory
            .parent()
            .ok_or(LocalAccountError::Operation)?;
        let mut candidate = LocalAccountVault::open(backup)?;
        candidate.unlock_with_passphrase(passphrase_text.clone())?;
        candidate
            .unlocked_ref()?
            ._vault
            .integrity_check()
            .map_err(|_| LocalAccountError::Operation)?;
        drop(candidate);
        let staging = staging_directory(&self.directory)?;
        if staging.exists() {
            fs::remove_dir_all(&staging).map_err(|_| LocalAccountError::Operation)?;
        }
        copy_directory(backup, &staging)?;
        let previous = self.directory.with_extension("pre-restore");
        if previous.exists() {
            fs::remove_dir_all(&previous).map_err(|_| LocalAccountError::Operation)?;
        }
        fs::rename(&self.directory, &previous).map_err(|_| LocalAccountError::Operation)?;
        if fs::rename(&staging, &self.directory).is_err() {
            restore_previous_directory(&previous, &self.directory, parent)?;
            return Err(LocalAccountError::Operation);
        }
        let restored = (|| {
            sync_directory(parent)?;
            let mut restored = LocalAccountVault::open(&self.directory)?;
            restored.unlock_with_passphrase(passphrase_text)?;
            Ok::<_, LocalAccountError>(restored)
        })();
        let restored = match restored {
            Ok(restored) => restored,
            Err(error) => {
                let _ = fs::remove_dir_all(&self.directory);
                if restore_previous_directory(&previous, &self.directory, parent).is_err() {
                    return Err(LocalAccountError::Operation);
                }
                return Err(error);
            }
        };
        self.manifest = restored.manifest;
        self.unlocked = restored.unlocked;
        let _ = fs::remove_dir_all(previous);
        Ok(())
    }

    pub fn reset_to_demo(
        &mut self,
        passphrase_text: String,
        confirmation: &str,
    ) -> Result<String, LocalAccountError> {
        let passphrase_text = Zeroizing::new(passphrase_text);
        if confirmation != RESET_CONFIRMATION {
            return Err(LocalAccountError::Operation);
        }
        self.unlocked_ref()?;
        let passphrase = Passphrase::new(passphrase_text.as_str());
        let envelope = KeyEnvelope::from_json(&self.manifest.passphrase_envelope)
            .map_err(|_| LocalAccountError::InvalidCredentials)?;
        envelope
            .unlock_with_passphrase(&passphrase)
            .map_err(|_| LocalAccountError::InvalidCredentials)?;

        let staging = staging_directory(&self.directory)?;
        let created = LocalAccountVault::create(
            &staging,
            CreateLocalAccount {
                account_id: self.manifest.account_id.clone(),
                passphrase: passphrase_text.as_str().to_owned(),
            },
        )?;
        let validation = Self::verify_seeded_demo_vault(&created.vault);
        let recovery_code = created.recovery_code().to_owned();
        drop(created);
        if let Err(error) = validation {
            let _ = fs::remove_dir_all(&staging);
            return Err(error);
        }
        let result = self.restore_from_backup(&staging, passphrase_text.as_str().to_owned());
        let _ = fs::remove_dir_all(&staging);
        result.map(|_| recovery_code)
    }

    fn verify_seeded_demo_vault(vault: &LocalAccountVault) -> Result<(), LocalAccountError> {
        let report_ids = vault.list_lab_report_ids()?;
        if report_ids.len() != 3 {
            return Err(LocalAccountError::Operation);
        }
        for report_id in report_ids {
            let report = vault.get_lab_report(&report_id)?;
            if report.status != ReportStatus::Complete
                || report.tags != vec!["demo".to_owned()]
                || report.source_files.len() != 1
                || report.measurements.len() != 3
            {
                return Err(LocalAccountError::Operation);
            }
        }
        vault
            .unlocked_ref()?
            ._vault
            .integrity_check()
            .map_err(|_| LocalAccountError::Operation)
    }

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
        let personal_target_ranges = analyte
            .personal_target_ranges
            .into_iter()
            .map(new_personal_target_range)
            .collect::<Result<Vec<_>, _>>()?;
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
                canonical_unit: nonempty(analyte.canonical_unit),
                personal_target_ranges,
            })
            .map_err(|_| LocalAccountError::Operation)?;
        Ok(id)
    }

    pub fn add_personal_target_range(
        &mut self,
        analyte_id: &str,
        range: NewPersonalTargetRange,
    ) -> Result<String, LocalAccountError> {
        let range = new_personal_target_range(range)?;
        let id = range.id.clone();
        self.unlocked_mut()?
            ._vault
            .add_personal_target_range(analyte_id, &range)
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

        let mut vault = LocalAccountVault {
            directory,
            manifest,
            unlocked: Some(unlocked),
        };
        if let Err(error) = vault
            .seed_default_analytes()
            .and_then(|_| vault.seed_demo_data())
        {
            let _ = fs::remove_dir_all(&vault.directory);
            return Err(error);
        }
        Ok(CreatedLocalAccount {
            vault,
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
            SizeLimitedReader::new(source, MAX_SOURCE_FILE_BYTES),
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
        let parsed_numeric_value = validated_numeric(measurement.parsed_numeric_value)?;
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
                    parsed_numeric_value,
                    analyte_id: measurement.analyte_id,
                    updated_at: String::new(),
                    updated_by: "local-user".to_owned(),
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

    pub fn correct_measurement(
        &mut self,
        measurement_id: &str,
        measurement: NewMeasurement,
        updated_by: String,
    ) -> Result<(), LocalAccountError> {
        if updated_by.trim().is_empty() {
            return Err(LocalAccountError::Operation);
        }
        let parsed_numeric_value = validated_numeric(measurement.parsed_numeric_value)?;
        self.unlocked_mut()?
            ._vault
            .correct_measurement(
                measurement_id,
                &MeasurementRecord {
                    id: measurement_id.to_owned(),
                    source_label: measurement.source_label,
                    source_value: measurement.source_value,
                    source_unit: measurement.source_unit,
                    source_reference_interval: measurement.source_reference_interval,
                    source_flag: measurement.source_flag,
                    parsed_numeric_value,
                    analyte_id: measurement.analyte_id,
                    updated_at: String::new(),
                    updated_by: updated_by.clone(),
                },
                &updated_by,
            )
            .map_err(|_| LocalAccountError::Operation)
    }

    pub fn archive_lab_report(&mut self, report_id: &str) -> Result<(), LocalAccountError> {
        self.unlocked_mut()?
            ._vault
            .archive_report(report_id)
            .map_err(|_| LocalAccountError::Operation)
    }

    pub fn permanently_delete_lab_report(
        &mut self,
        report_id: &str,
        confirmed: bool,
    ) -> Result<(), LocalAccountError> {
        if !confirmed {
            return Err(LocalAccountError::Operation);
        }
        let report = self.get_lab_report(report_id)?;
        let unlocked = self.unlocked_mut()?;
        unlocked
            ._vault
            .delete_report(report_id)
            .map_err(|_| LocalAccountError::Operation)?;
        for source in report.source_files {
            let _ = fs::remove_file(
                self.directory
                    .join("objects")
                    .join(format!("{}.hemo", source.opaque_object_id)),
            );
        }
        Ok(())
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
                ReportState::Archived => ReportStatus::Archived,
            },
            source_files: report
                .source_files
                .into_iter()
                .map(|source| LabReportSourceFile {
                    id: source.id,
                    original_filename: source.original_filename,
                    media_type: source.media_type,
                    role: source.role,
                    opaque_object_id: source.opaque_object_id,
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
                    parsed_numeric_value: measurement.parsed_numeric_value,
                    analyte_id: measurement.analyte_id,
                    updated_at: measurement.updated_at,
                    updated_by: measurement.updated_by,
                })
                .collect(),
        })
    }

    pub fn read_source_file(
        &self,
        report_id: &str,
        source_file_id: &str,
    ) -> Result<(String, String, Vec<u8>), LocalAccountError> {
        let report = self.get_lab_report(report_id)?;
        let source = report
            .source_files
            .iter()
            .find(|source| source.id == source_file_id)
            .ok_or(LocalAccountError::Operation)?;
        let object = self
            .directory
            .join("objects")
            .join(format!("{}.hemo", source.opaque_object_id));
        let context = SourceFileContext::new(
            self.manifest.account_id.clone(),
            OpaqueObjectId::parse(source.opaque_object_id.clone())
                .map_err(|_| LocalAccountError::Operation)?,
        );
        let unlocked = self.unlocked_ref()?;
        let source_key = SourceFileKey::from_bytes(*unlocked.source_file_key.bytes());
        let mut bytes = Vec::new();
        decrypt_source_file(object, &mut bytes, &context, &source_key)
            .map_err(|_| LocalAccountError::Operation)?;
        Ok((
            source.original_filename.clone(),
            source.media_type.clone(),
            bytes,
        ))
    }

    pub fn list_lab_report_ids(&self) -> Result<Vec<String>, LocalAccountError> {
        self.unlocked_ref()?
            ._vault
            .list_report_ids()
            .map_err(|_| LocalAccountError::Operation)
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
        self.seed_default_analytes()?;
        Ok(())
    }

    fn seed_default_analytes(&mut self) -> Result<(), LocalAccountError> {
        if !self
            .unlocked_ref()?
            ._vault
            .list_analytes()
            .map_err(|_| LocalAccountError::Operation)?
            .is_empty()
        {
            return Ok(());
        }
        for (name, component, property, specimen, loinc, canonical_unit) in [
            (
                "Hemoglobin",
                "Hemoglobin",
                "MCnc",
                "Blood",
                Some("718-7"),
                "g/dL",
            ),
            (
                "Glucose",
                "Glucose",
                "MCnc",
                "Serum or plasma",
                Some("2345-7"),
                "mmol/L",
            ),
            (
                "Creatinine",
                "Creatinine",
                "MCnc",
                "Serum or plasma",
                Some("2160-0"),
                "umol/L",
            ),
            (
                "Platelet count",
                "Platelets",
                "N",
                "Blood",
                Some("777-3"),
                "10*3/uL",
            ),
        ] {
            self.add_analyte(NewAnalyte {
                name: name.to_owned(),
                component: component.to_owned(),
                property: property.to_owned(),
                specimen: specimen.to_owned(),
                scale: "Quantitative".to_owned(),
                method: None,
                aliases: Vec::new(),
                loinc_code: loinc.map(str::to_owned),
                canonical_unit: Some(canonical_unit.to_owned()),
                personal_target_ranges: Vec::new(),
            })?;
        }
        Ok(())
    }

    fn seed_demo_data(&mut self) -> Result<(), LocalAccountError> {
        let analytes = self.list_analytes()?;
        let analyte_id = |name: &str| {
            analytes
                .iter()
                .find(|analyte| analyte.name == name)
                .map(|analyte| analyte.id.clone())
                .ok_or(LocalAccountError::Operation)
        };
        let hemoglobin_id = analyte_id("Hemoglobin")?;
        let glucose_id = analyte_id("Glucose")?;
        let creatinine_id = analyte_id("Creatinine")?;
        let samples = [
            ("2026-01-15T08:00:00Z", "13.4", "90", "1.02"),
            ("2026-04-22T08:30:00Z", "13.8", "4.99567", "90.3"),
            ("2026-07-18T07:45:00Z", "14.1", "96", "1.08"),
        ];
        for (index, (collection_time, hemoglobin, glucose, creatinine)) in
            samples.into_iter().enumerate()
        {
            let report_id = self.create_lab_report_draft(CreateLabReportDraft {
                collection_time: collection_time.to_owned(),
                report_date: Some(collection_time[..10].to_owned()),
                laboratory: Some("Fictional Demo Laboratory".to_owned()),
                ordering_clinician: None,
                fasting_state: Some("unknown".to_owned()),
                notes: Some(
                    "Fictional demo data. Replace or archive this report before recording personal data."
                        .to_owned(),
                ),
                tags: vec!["demo".to_owned()],
            })?;
            self.add_source_file(
                &report_id,
                Cursor::new(format!(
                    "Hemo Tracker fictional demo report {}. This file is not a laboratory report.\n",
                    index + 1
                )),
                NewSourceFile {
                    original_filename: format!("demo-lab-report-{}.txt", index + 1),
                    media_type: "text/plain".to_owned(),
                    role: "primary".to_owned(),
                },
            )?;
            for (label, value, unit, interval, analyte) in [
                (
                    "Hemoglobin",
                    hemoglobin,
                    "g/dL",
                    "12-16 g/dL",
                    hemoglobin_id.as_str(),
                ),
                (
                    "Glucose",
                    glucose,
                    if index == 1 { "mmol/L" } else { "mg/dL" },
                    if index == 1 {
                        "4.0-5.6 mmol/L"
                    } else {
                        "70-100 mg/dL"
                    },
                    glucose_id.as_str(),
                ),
                (
                    "Creatinine",
                    creatinine,
                    "mg/dL",
                    "0.6-1.2 mg/dL",
                    creatinine_id.as_str(),
                ),
            ] {
                self.add_measurement(
                    &report_id,
                    NewMeasurement {
                        source_label: label.to_owned(),
                        source_value: value.to_owned(),
                        source_unit: unit.to_owned(),
                        source_reference_interval: interval.to_owned(),
                        source_flag: String::new(),
                        parsed_numeric_value: Some(value.to_owned()),
                        analyte_id: Some(analyte.to_owned()),
                    },
                )?;
            }
            self.complete_lab_report(&report_id)?;
        }
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

fn new_personal_target_range(
    range: NewPersonalTargetRange,
) -> Result<PersonalTargetRange, LocalAccountError> {
    let lower_bound = nonempty(range.lower_bound);
    let upper_bound = nonempty(range.upper_bound);
    let notes = nonempty(range.notes);
    if lower_bound.is_none() && upper_bound.is_none() && notes.is_none() {
        return Err(LocalAccountError::Operation);
    }
    if (lower_bound.is_some() || upper_bound.is_some()) && range.unit.trim().is_empty() {
        return Err(LocalAccountError::Operation);
    }
    let lower_numeric = lower_bound.as_deref().map(parse_decimal).transpose()?;
    let upper_numeric = upper_bound.as_deref().map(parse_decimal).transpose()?;
    if let (Some(lower), Some(upper)) = (lower_numeric, upper_numeric)
        && lower > upper
    {
        return Err(LocalAccountError::Operation);
    }
    if let (Some(from), Some(to)) = (&range.valid_from, &range.valid_to)
        && from > to
    {
        return Err(LocalAccountError::Operation);
    }
    Ok(PersonalTargetRange {
        id: random_identifier()?,
        lower_bound,
        upper_bound,
        unit: range.unit.trim().to_owned(),
        valid_from: nonempty(range.valid_from),
        valid_to: nonempty(range.valid_to),
        context: nonempty(range.context),
        notes,
    })
}

fn parse_decimal(value: &str) -> Result<f64, LocalAccountError> {
    value
        .replace(',', ".")
        .parse::<f64>()
        .ok()
        .filter(|value| value.is_finite())
        .ok_or(LocalAccountError::Operation)
}

fn csv_field(value: &str) -> String {
    if value
        .bytes()
        .any(|byte| matches!(byte, b',' | b'"' | b'\r' | b'\n'))
    {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_owned()
    }
}

fn validated_numeric(value: Option<String>) -> Result<Option<String>, LocalAccountError> {
    let value = nonempty(value);
    value
        .as_deref()
        .map(parse_decimal)
        .transpose()
        .map(|_| value)
}

fn nonempty(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let value = value.trim().to_owned();
        (!value.is_empty()).then_some(value)
    })
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

fn copy_directory(source: &Path, destination: &Path) -> Result<(), LocalAccountError> {
    fs::create_dir_all(destination).map_err(|_| LocalAccountError::Operation)?;
    for entry in fs::read_dir(source).map_err(|_| LocalAccountError::Operation)? {
        let entry = entry.map_err(|_| LocalAccountError::Operation)?;
        let from = entry.path();
        let to = destination.join(entry.file_name());
        if from.is_dir() {
            copy_directory(&from, &to)?;
        } else {
            fs::copy(&from, &to).map_err(|_| LocalAccountError::Operation)?;
        }
    }
    Ok(())
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

fn restore_previous_directory(
    previous: &Path,
    destination: &Path,
    parent: &Path,
) -> Result<(), LocalAccountError> {
    fs::rename(previous, destination).or_else(|_| {
        copy_directory(previous, destination)?;
        fs::remove_dir_all(previous).map_err(|_| LocalAccountError::Operation)
    })?;
    sync_directory(parent)
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

    #[test]
    fn csv_fields_escape_structural_characters() {
        assert_eq!(csv_field("plain"), "plain");
        assert_eq!(csv_field("lab, west"), "\"lab, west\"");
        assert_eq!(
            csv_field("value \"as printed\""),
            "\"value \"\"as printed\"\"\""
        );
        assert_eq!(csv_field("line\none"), "\"line\none\"");
    }

    #[test]
    fn size_limited_reader_rejects_bytes_after_the_limit() {
        let mut reader = SizeLimitedReader::new(Cursor::new(b"12345"), 4);
        let mut buffer = [0_u8; 4];
        assert_eq!(reader.read(&mut buffer).unwrap(), 4);
        assert_eq!(&buffer, b"1234");
        assert_eq!(
            reader.read(&mut [0_u8; 1]).unwrap_err().kind(),
            io::ErrorKind::FileTooLarge
        );
    }

    #[test]
    fn size_limited_reader_accepts_a_source_at_the_limit() {
        let mut reader = SizeLimitedReader::new(Cursor::new(b"1234"), 4);
        let mut buffer = [0_u8; 4];
        assert_eq!(reader.read(&mut buffer).unwrap(), 4);
        assert_eq!(reader.read(&mut [0_u8; 1]).unwrap(), 0);
    }
}
