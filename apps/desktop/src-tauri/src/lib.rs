pub mod account_vault;
mod vault_commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(vault_commands::DesktopVaultState::default())
        .invoke_handler(tauri::generate_handler![
            vault_commands::get_vault_state,
            vault_commands::create_local_account,
            vault_commands::unlock_with_passphrase,
            vault_commands::unlock_with_recovery,
            vault_commands::lock_vault,
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
    }
}
