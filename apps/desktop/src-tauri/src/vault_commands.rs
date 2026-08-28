use crate::account_vault::{
    CreateLabReportDraft, CreateLocalAccount, LocalAccountVault, NewAnalyte, NewMeasurement,
    NewPersonalTargetRange, NewSourceFile, VaultStatus,
};
use serde::Serialize;
use std::fs::File;
use std::path::Path;
use std::{path::PathBuf, sync::Mutex};
use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::DialogExt;

const LOCAL_ACCOUNT_ID: &str = "default";

#[derive(Default)]
pub struct DesktopVaultState {
    vault: Mutex<Option<LocalAccountVault>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultStateResult {
    account_exists: bool,
    status: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatedVaultResult {
    recovery_code: String,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateReportRequest {
    pub collection_time: String,
    pub report_date: Option<String>,
    pub laboratory: Option<String>,
    pub ordering_clinician: Option<String>,
    pub fasting_state: Option<String>,
    pub notes: Option<String>,
    pub tags: Vec<String>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MeasurementRequest {
    pub source_label: String,
    pub source_value: String,
    pub source_unit: String,
    pub source_reference_interval: String,
    pub source_flag: String,
    pub analyte_id: Option<String>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyteRequest {
    pub name: String,
    pub component: String,
    pub property: String,
    pub specimen: String,
    pub scale: String,
    pub method: Option<String>,
    pub aliases: Vec<String>,
    pub loinc_code: Option<String>,
    pub personal_target_ranges: Vec<PersonalTargetRangeRequest>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonalTargetRangeRequest {
    pub lower_bound: Option<String>,
    pub upper_bound: Option<String>,
    pub unit: String,
    pub valid_from: Option<String>,
    pub valid_to: Option<String>,
    pub context: Option<String>,
    pub notes: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyteResult {
    pub id: String,
    pub name: String,
    pub component: String,
    pub property: String,
    pub specimen: String,
    pub scale: String,
    pub method: Option<String>,
    pub aliases: Vec<String>,
    pub loinc_code: Option<String>,
    pub personal_target_ranges: Vec<PersonalTargetRangeResult>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonalTargetRangeResult {
    pub id: String,
    pub lower_bound: Option<String>,
    pub upper_bound: Option<String>,
    pub unit: String,
    pub valid_from: Option<String>,
    pub valid_to: Option<String>,
    pub context: Option<String>,
    pub notes: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceFileResult {
    pub id: String,
    pub original_filename: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportResult {
    pub id: String,
    pub collection_time: String,
    pub laboratory: Option<String>,
    pub status: String,
    pub source_file_count: usize,
    pub measurement_count: usize,
    pub source_files: Vec<SourcePreviewResult>,
    pub measurements: Vec<MeasurementPreviewResult>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MeasurementPreviewResult {
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

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourcePreviewResult {
    pub filename: String,
    pub media_type: String,
    pub role: String,
}

#[tauri::command]
pub fn get_vault_state(
    app: AppHandle,
    state: State<'_, DesktopVaultState>,
) -> Result<VaultStateResult, String> {
    let directory = account_directory(&app)?;
    let mut vault = state.vault.lock().map_err(|_| safe_error())?;
    if vault.is_none() && directory.join("account.json").is_file() {
        *vault = Some(LocalAccountVault::open(&directory).map_err(|_| safe_error())?);
    }
    Ok(result(vault.as_ref()))
}

#[tauri::command]
pub fn create_local_account(
    app: AppHandle,
    state: State<'_, DesktopVaultState>,
    passphrase: String,
) -> Result<CreatedVaultResult, String> {
    let directory = account_directory(&app)?;
    let mut vault = state.vault.lock().map_err(|_| safe_error())?;
    if vault.is_some() || directory.join("account.json").exists() {
        return Err("the local account already exists".to_owned());
    }
    let created = LocalAccountVault::create(
        directory,
        CreateLocalAccount {
            account_id: LOCAL_ACCOUNT_ID.to_owned(),
            passphrase,
        },
    )
    .map_err(|_| safe_error())?;
    let recovery_code = created.recovery_code().to_owned();
    *vault = Some(created.into_vault());
    Ok(CreatedVaultResult { recovery_code })
}

#[tauri::command]
pub fn unlock_with_passphrase(
    app: AppHandle,
    state: State<'_, DesktopVaultState>,
    passphrase: String,
) -> Result<VaultStateResult, String> {
    with_vault(&app, &state, |vault| {
        vault.unlock_with_passphrase(passphrase)
    })
}

#[tauri::command]
pub fn unlock_with_recovery(
    app: AppHandle,
    state: State<'_, DesktopVaultState>,
    recovery_code: String,
) -> Result<VaultStateResult, String> {
    with_vault(&app, &state, |vault| {
        vault.unlock_with_recovery(recovery_code)
    })
}

#[tauri::command]
pub fn lock_vault(
    app: AppHandle,
    state: State<'_, DesktopVaultState>,
) -> Result<VaultStateResult, String> {
    with_vault(&app, &state, |vault| {
        vault.lock();
        Ok(())
    })
}

#[tauri::command]
pub fn create_lab_report(
    state: State<'_, DesktopVaultState>,
    request: CreateReportRequest,
) -> Result<String, String> {
    let mut guard = state.vault.lock().map_err(|_| safe_error())?;
    guard
        .as_mut()
        .ok_or_else(safe_error)?
        .create_lab_report_draft(CreateLabReportDraft {
            collection_time: request.collection_time,
            report_date: request.report_date,
            laboratory: request.laboratory,
            ordering_clinician: request.ordering_clinician,
            fasting_state: request.fasting_state,
            notes: request.notes,
            tags: request.tags,
        })
        .map_err(|_| safe_error())
}

#[tauri::command]
pub fn select_and_attach_source_file(
    app: AppHandle,
    state: State<'_, DesktopVaultState>,
    report_id: String,
    role: String,
) -> Result<Option<SourceFileResult>, String> {
    let Some(file_path) = app
        .dialog()
        .file()
        .add_filter("Laboratory reports", &["pdf", "png", "jpg", "jpeg", "heic"])
        .blocking_pick_file()
    else {
        return Ok(None);
    };
    let path = file_path.into_path().map_err(|_| safe_error())?;
    let original_filename = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(safe_error)?
        .to_owned();
    let media_type = media_type_for(&path);
    let file = File::open(&path).map_err(|_| safe_error())?;
    let mut guard = state.vault.lock().map_err(|_| safe_error())?;
    let id = guard
        .as_mut()
        .ok_or_else(safe_error)?
        .add_source_file(
            &report_id,
            file,
            NewSourceFile {
                original_filename: original_filename.clone(),
                media_type,
                role,
            },
        )
        .map_err(|_| safe_error())?;
    Ok(Some(SourceFileResult {
        id,
        original_filename,
    }))
}

#[tauri::command]
pub fn add_lab_measurement(
    state: State<'_, DesktopVaultState>,
    report_id: String,
    request: MeasurementRequest,
) -> Result<String, String> {
    let mut guard = state.vault.lock().map_err(|_| safe_error())?;
    guard
        .as_mut()
        .ok_or_else(safe_error)?
        .add_measurement(
            &report_id,
            NewMeasurement {
                source_label: request.source_label,
                source_value: request.source_value,
                source_unit: request.source_unit,
                source_reference_interval: request.source_reference_interval,
                source_flag: request.source_flag,
                analyte_id: request.analyte_id,
            },
        )
        .map_err(|_| safe_error())
}

#[tauri::command]
pub fn add_analyte_definition(
    state: State<'_, DesktopVaultState>,
    request: AnalyteRequest,
) -> Result<String, String> {
    let mut guard = state.vault.lock().map_err(|_| safe_error())?;
    guard
        .as_mut()
        .ok_or_else(safe_error)?
        .add_analyte(NewAnalyte {
            name: request.name,
            component: request.component,
            property: request.property,
            specimen: request.specimen,
            scale: request.scale,
            method: request.method,
            aliases: request.aliases,
            loinc_code: request.loinc_code,
            personal_target_ranges: request
                .personal_target_ranges
                .into_iter()
                .map(new_personal_target_range)
                .collect(),
        })
        .map_err(|_| safe_error())
}

#[tauri::command]
pub fn list_analyte_definitions(
    state: State<'_, DesktopVaultState>,
) -> Result<Vec<AnalyteResult>, String> {
    let guard = state.vault.lock().map_err(|_| safe_error())?;
    guard
        .as_ref()
        .ok_or_else(safe_error)?
        .list_analytes()
        .map(|items| {
            items
                .into_iter()
                .map(|item| AnalyteResult {
                    id: item.id,
                    name: item.name,
                    component: item.component,
                    property: item.property,
                    specimen: item.specimen,
                    scale: item.scale,
                    method: item.method,
                    aliases: item.aliases,
                    loinc_code: item.loinc_code,
                    personal_target_ranges: item
                        .personal_target_ranges
                        .into_iter()
                        .map(|range| PersonalTargetRangeResult {
                            id: range.id,
                            lower_bound: range.lower_bound,
                            upper_bound: range.upper_bound,
                            unit: range.unit,
                            valid_from: range.valid_from,
                            valid_to: range.valid_to,
                            context: range.context,
                            notes: range.notes,
                        })
                        .collect(),
                })
                .collect()
        })
        .map_err(|_| safe_error())
}

#[tauri::command]
pub fn add_personal_target_range(
    state: State<'_, DesktopVaultState>,
    analyte_id: String,
    request: PersonalTargetRangeRequest,
) -> Result<String, String> {
    let mut guard = state.vault.lock().map_err(|_| safe_error())?;
    guard
        .as_mut()
        .ok_or_else(safe_error)?
        .add_personal_target_range(&analyte_id, new_personal_target_range(request))
        .map_err(|_| safe_error())
}

fn new_personal_target_range(request: PersonalTargetRangeRequest) -> NewPersonalTargetRange {
    NewPersonalTargetRange {
        lower_bound: request.lower_bound,
        upper_bound: request.upper_bound,
        unit: request.unit,
        valid_from: request.valid_from,
        valid_to: request.valid_to,
        context: request.context,
        notes: request.notes,
    }
}

#[tauri::command]
pub fn get_lab_report(
    state: State<'_, DesktopVaultState>,
    report_id: String,
) -> Result<ReportResult, String> {
    let guard = state.vault.lock().map_err(|_| safe_error())?;
    let report = guard
        .as_ref()
        .ok_or_else(safe_error)?
        .get_lab_report(&report_id)
        .map_err(|_| safe_error())?;
    Ok(ReportResult {
        id: report.id,
        collection_time: report.collection_time,
        laboratory: report.laboratory,
        status: match report.status {
            crate::account_vault::ReportStatus::Draft => "draft",
            crate::account_vault::ReportStatus::Complete => "complete",
            crate::account_vault::ReportStatus::Archived => "archived",
        }
        .to_owned(),
        source_file_count: report.source_files.len(),
        measurement_count: report.measurements.len(),
        source_files: report
            .source_files
            .into_iter()
            .map(|source| SourcePreviewResult {
                filename: source.original_filename,
                media_type: source.media_type,
                role: source.role,
            })
            .collect(),
        measurements: report
            .measurements
            .into_iter()
            .map(|measurement| MeasurementPreviewResult {
                id: measurement.id,
                source_label: measurement.source_label,
                source_value: measurement.source_value,
                source_unit: measurement.source_unit,
                source_reference_interval: measurement.source_reference_interval,
                source_flag: measurement.source_flag,
                analyte_id: measurement.analyte_id,
                updated_at: measurement.updated_at,
                updated_by: measurement.updated_by,
            })
            .collect(),
    })
}

#[tauri::command]
pub fn list_lab_reports(state: State<'_, DesktopVaultState>) -> Result<Vec<String>, String> {
    let guard = state.vault.lock().map_err(|_| safe_error())?;
    guard
        .as_ref()
        .ok_or_else(safe_error)?
        .list_lab_report_ids()
        .map_err(|_| safe_error())
}

#[tauri::command]
pub fn complete_lab_report(
    state: State<'_, DesktopVaultState>,
    report_id: String,
) -> Result<(), String> {
    let mut guard = state.vault.lock().map_err(|_| safe_error())?;
    guard
        .as_mut()
        .ok_or_else(safe_error)?
        .complete_lab_report(&report_id)
        .map_err(|_| safe_error())
}

#[tauri::command]
pub fn archive_lab_report(
    state: State<'_, DesktopVaultState>,
    report_id: String,
) -> Result<(), String> {
    let mut guard = state.vault.lock().map_err(|_| safe_error())?;
    guard
        .as_mut()
        .ok_or_else(safe_error)?
        .archive_lab_report(&report_id)
        .map_err(|_| safe_error())
}

#[tauri::command]
pub fn correct_lab_measurement(
    state: State<'_, DesktopVaultState>,
    measurement_id: String,
    request: MeasurementRequest,
    updated_by: String,
) -> Result<(), String> {
    let mut guard = state.vault.lock().map_err(|_| safe_error())?;
    guard
        .as_mut()
        .ok_or_else(safe_error)?
        .correct_measurement(
            &measurement_id,
            NewMeasurement {
                source_label: request.source_label,
                source_value: request.source_value,
                source_unit: request.source_unit,
                source_reference_interval: request.source_reference_interval,
                source_flag: request.source_flag,
                analyte_id: request.analyte_id,
            },
            updated_by,
        )
        .map_err(|_| safe_error())
}

fn with_vault(
    app: &AppHandle,
    state: &State<'_, DesktopVaultState>,
    operation: impl FnOnce(
        &mut LocalAccountVault,
    ) -> Result<(), crate::account_vault::LocalAccountError>,
) -> Result<VaultStateResult, String> {
    let directory = account_directory(app)?;
    let mut guard = state.vault.lock().map_err(|_| safe_error())?;
    if guard.is_none() {
        *guard = Some(LocalAccountVault::open(directory).map_err(|_| safe_error())?);
    }
    let vault = guard.as_mut().ok_or_else(safe_error)?;
    operation(vault).map_err(|_| safe_error())?;
    Ok(result(Some(vault)))
}

#[tauri::command]
pub fn permanently_delete_lab_report(
    state: State<'_, DesktopVaultState>,
    report_id: String,
    confirmed: bool,
) -> Result<(), String> {
    let mut guard = state.vault.lock().map_err(|_| safe_error())?;
    guard
        .as_mut()
        .ok_or_else(safe_error)?
        .permanently_delete_lab_report(&report_id, confirmed)
        .map_err(|_| safe_error())
}

fn result(vault: Option<&LocalAccountVault>) -> VaultStateResult {
    match vault.map(LocalAccountVault::status) {
        None => VaultStateResult {
            account_exists: false,
            status: "missing",
        },
        Some(VaultStatus::Locked) => VaultStateResult {
            account_exists: true,
            status: "locked",
        },
        Some(VaultStatus::Unlocked) => VaultStateResult {
            account_exists: true,
            status: "unlocked",
        },
    }
}

#[tauri::command]
pub fn backup_local_vault(
    state: State<'_, DesktopVaultState>,
    destination: String,
) -> Result<(), String> {
    let guard = state.vault.lock().map_err(|_| safe_error())?;
    guard
        .as_ref()
        .ok_or_else(safe_error)?
        .backup_to(destination)
        .map_err(|_| safe_error())
}

#[tauri::command]
pub fn choose_and_backup_local_vault(
    app: AppHandle,
    state: State<'_, DesktopVaultState>,
) -> Result<bool, String> {
    let Some(path) = app
        .dialog()
        .file()
        .set_title("Save encrypted Hemo Tracker backup")
        .set_file_name("hemo-tracker-backup")
        .blocking_save_file()
    else {
        return Ok(false);
    };
    let destination = path.into_path().map_err(|_| safe_error())?;
    let guard = state.vault.lock().map_err(|_| safe_error())?;
    guard
        .as_ref()
        .ok_or_else(safe_error)?
        .backup_to(destination)
        .map_err(|_| safe_error())?;
    Ok(true)
}

#[tauri::command]
pub fn restore_local_vault(
    state: State<'_, DesktopVaultState>,
    backup: String,
    passphrase: String,
) -> Result<(), String> {
    let mut guard = state.vault.lock().map_err(|_| safe_error())?;
    guard
        .as_mut()
        .ok_or_else(safe_error)?
        .restore_from_backup(backup, passphrase)
        .map_err(|_| safe_error())
}

#[tauri::command]
pub fn choose_and_restore_local_vault(
    app: AppHandle,
    state: State<'_, DesktopVaultState>,
    passphrase: String,
) -> Result<bool, String> {
    let Some(path) = app
        .dialog()
        .file()
        .set_title("Select encrypted Hemo Tracker backup")
        .blocking_pick_folder()
    else {
        return Ok(false);
    };
    let mut guard = state.vault.lock().map_err(|_| safe_error())?;
    guard
        .as_mut()
        .ok_or_else(safe_error)?
        .restore_from_backup(path.into_path().map_err(|_| safe_error())?, passphrase)
        .map_err(|_| safe_error())?;
    Ok(true)
}

#[tauri::command]
pub fn choose_and_export_plaintext_zip(
    app: AppHandle,
    state: State<'_, DesktopVaultState>,
) -> Result<bool, String> {
    let Some(path) = app
        .dialog()
        .file()
        .set_title("Save plaintext Hemo Tracker export")
        .set_file_name("hemo-tracker-export.zip")
        .blocking_save_file()
    else {
        return Ok(false);
    };
    let destination = path.into_path().map_err(|_| safe_error())?;
    let guard = state.vault.lock().map_err(|_| safe_error())?;
    guard
        .as_ref()
        .ok_or_else(safe_error)?
        .export_plaintext_zip(destination)
        .map_err(|_| safe_error())?;
    Ok(true)
}

fn account_directory(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map(|path| path.join("accounts").join(LOCAL_ACCOUNT_ID))
        .map_err(|_| safe_error())
}

fn safe_error() -> String {
    "the local vault operation failed".to_owned()
}

fn media_type_for(path: &Path) -> String {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("pdf") => "application/pdf",
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("heic") => "image/heic",
        _ => "application/octet-stream",
    }
    .to_owned()
}
