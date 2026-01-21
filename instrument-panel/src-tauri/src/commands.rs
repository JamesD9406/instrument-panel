use crate::hwinfo::{self, types::SensorData};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::Manager;

/// Settings stored locally
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub auto_launch_hwinfo: bool,
    pub hwinfo_path: Option<String>,
    pub mock_mode: bool,
}

fn settings_path(app: &tauri::AppHandle) -> PathBuf {
    let app_dir = app.path().app_data_dir().unwrap_or_else(|_| PathBuf::from("."));
    let _ = fs::create_dir_all(&app_dir);
    app_dir.join("settings.json")
}

/// Get current sensor data from HWiNFO shared memory
#[tauri::command]
pub fn get_sensor_data(app: tauri::AppHandle) -> SensorData {
    // Check if mock mode is enabled
    let settings = get_settings(app);
    if settings.mock_mode {
        return hwinfo::mock::generate_mock_data();
    }
    
    hwinfo::read_sensor_data()
}

/// Check if HWiNFO process is running
#[tauri::command]
pub fn is_hwinfo_running() -> bool {
    hwinfo::process::is_running()
}

/// Launch HWiNFO executable
#[tauri::command]
pub fn launch_hwinfo(custom_path: Option<String>) -> Result<(), String> {
    hwinfo::process::launch(custom_path)
}

/// Get saved settings
#[tauri::command]
pub fn get_settings(app: tauri::AppHandle) -> AppSettings {
    let path = settings_path(&app);
    if path.exists() {
        fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    } else {
        AppSettings::default()
    }
}

/// Save settings
#[tauri::command]
pub fn save_settings(app: tauri::AppHandle, settings: AppSettings) -> Result<(), String> {
    let path = settings_path(&app);
    let json = serde_json::to_string_pretty(&settings)
        .map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())
}
