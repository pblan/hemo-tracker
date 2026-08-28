pub mod account_vault;
mod vault_commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(vault_commands::DesktopVaultState::default())
        .invoke_handler(tauri::generate_handler![
            vault_commands::get_vault_state,
            vault_commands::create_local_account,
            vault_commands::unlock_with_passphrase,
            vault_commands::unlock_with_recovery,
            vault_commands::lock_vault,
            vault_commands::create_lab_report,
            vault_commands::select_and_attach_source_file,
            vault_commands::add_lab_measurement,
            vault_commands::add_analyte_definition,
            vault_commands::add_personal_target_range,
            vault_commands::list_analyte_definitions,
            vault_commands::get_lab_report,
            vault_commands::list_lab_reports,
            vault_commands::complete_lab_report,
            vault_commands::archive_lab_report,
            vault_commands::correct_lab_measurement,
            vault_commands::permanently_delete_lab_report,
            vault_commands::backup_local_vault,
            vault_commands::choose_and_backup_local_vault,
            vault_commands::restore_local_vault,
            vault_commands::choose_and_restore_local_vault,
            vault_commands::choose_and_export_plaintext_zip,
        ])
        .run(tauri::generate_context!())
        .expect("Hemo Tracker failed to start");
}

#[cfg(test)]
mod tests {
    #[test]
    fn configuration_identifies_the_desktop_application() {
        let config: serde_json::Value =
            serde_json::from_str(include_str!("../tauri.conf.json")).unwrap();

        assert_eq!(config["productName"], "Hemo Tracker");
        let csp = config["app"]["security"]["csp"].as_str().unwrap();
        assert!(csp.contains("object-src 'none'"));
        assert!(csp.contains("frame-ancestors 'none'"));
    }

    #[test]
    fn default_capability_allows_native_file_open_dialog() {
        let capability: serde_json::Value =
            serde_json::from_str(include_str!("../capabilities/default.json")).unwrap();

        assert!(
            capability["permissions"]
                .as_array()
                .unwrap()
                .iter()
                .any(|permission| permission == "dialog:allow-open")
        );
        assert!(
            capability["permissions"]
                .as_array()
                .unwrap()
                .iter()
                .any(|permission| permission == "dialog:allow-save")
        );
    }
}
