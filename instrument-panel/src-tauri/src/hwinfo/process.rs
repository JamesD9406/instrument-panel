use sysinfo::System;
use std::process::Command;

/// Check if HWiNFO64.exe or HWiNFO32.exe is running
pub fn is_running() -> bool {
    let mut sys = System::new();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    
    for process in sys.processes().values() {
        let name = process.name().to_string_lossy().to_lowercase();
        if name.contains("hwinfo64") || name.contains("hwinfo32") {
            return true;
        }
    }
    false
}

/// Launch HWiNFO executable
pub fn launch(custom_path: Option<String>) -> Result<(), String> {
    // Try paths in order of preference
    let paths_to_try: Vec<String> = if let Some(path) = custom_path {
        vec![path]
    } else {
        vec![
            r"C:\Program Files\HWiNFO64\HWiNFO64.exe".to_string(),
            r"C:\Program Files (x86)\HWiNFO64\HWiNFO64.exe".to_string(),
            r"C:\Program Files\HWiNFO32\HWiNFO32.exe".to_string(),
        ]
    };

    for path in &paths_to_try {
        if std::path::Path::new(path).exists() {
            return Command::new(path)
                .spawn()
                .map(|_| ())
                .map_err(|e| format!("Failed to launch HWiNFO: {}", e));
        }
    }

    Err("HWiNFO not found. Please install it or set a custom path.".to_string())
}