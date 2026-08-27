const PRODUCT_NAME: &str = "Hemo Tracker";

pub fn product_name() -> &'static str {
    PRODUCT_NAME
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("Hemo Tracker failed to start");
}

#[cfg(test)]
mod tests {
    use super::product_name;

    #[test]
    fn identifies_the_desktop_application() {
        assert_eq!(product_name(), "Hemo Tracker");
    }
}
