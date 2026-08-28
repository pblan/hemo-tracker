use crate::account_vault::{CreateLocalAccount, LocalAccountVault, VaultStatus};
use serde::Serialize;
use std::{path::PathBuf, sync::Mutex};
use tauri::{AppHandle, Manager, State};

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

fn account_directory(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map(|path| path.join("accounts").join(LOCAL_ACCOUNT_ID))
        .map_err(|_| safe_error())
}

fn safe_error() -> String {
    "the local vault operation failed".to_owned()
}
