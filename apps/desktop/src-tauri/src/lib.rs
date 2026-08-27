#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
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
