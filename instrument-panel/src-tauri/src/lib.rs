mod commands;
mod hwinfo;

use commands::{get_sensor_data, is_hwinfo_running, launch_hwinfo, get_settings, save_settings, debug_dump_sensors, debug_dump_readings};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            get_sensor_data,
            is_hwinfo_running,
            launch_hwinfo,
            get_settings,
            save_settings,
            debug_dump_sensors,
            debug_dump_readings
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

