use serde::Serialize;

#[derive(Serialize)]
pub struct AppInfo {
    pub version: String,
    pub name: String,
    pub poe_version: String,
    pub league: String,
}

/// Returns the app version from Cargo.toml.
#[tauri::command]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Returns full app metadata.
#[tauri::command]
pub fn get_app_info() -> AppInfo {
    AppInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        name: "Path of AI".to_string(),
        poe_version: "PoE 1 + PoE 2".to_string(),
        league: "Mirage (3.28)".to_string(),
    }
}
