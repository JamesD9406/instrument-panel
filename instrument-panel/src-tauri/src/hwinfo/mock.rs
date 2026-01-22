use super::types::*;
use chrono::Utc;

static START_TIME: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();

pub fn read_mock_data() -> SensorData {
    let start = START_TIME.get_or_init(std::time::Instant::now);
    let elapsed_secs = start.elapsed().as_secs();
    let uptime_base: u64 = 3600 * 24 * 2; // 2 days base

    // Add some variation based on time
    let variation = ((elapsed_secs % 10) as f64 - 5.0) * 0.5;

    SensorData {
        status: "connected".to_string(),
        last_read_at: Some(Utc::now().to_rfc3339()),
        diagnostics: Diagnostics {
            hwinfo_process_detected: true,
            shared_memory_detected: true,
            message: Some("Mock mode active".to_string()),
        },
        cpu: CpuData {
            name: Some("AMD Ryzen 7 7800X3D".to_string()),
            package_temp_c: Some(45.0 + variation),
            package_power_w: Some(65.0 + variation * 2.0),
            core_clock_mhz: Some(4500.0 + variation * 100.0),
            usage_percent: Some(25.0 + variation * 5.0),
            core_temps: vec![44.0, 45.0, 43.0, 46.0, 44.0, 45.0, 43.0, 44.0],
        },
        gpu: GpuData {
            name: Some("NVIDIA GeForce RTX 5070".to_string()),
            hotspot_temp_c: Some(55.0 + variation),
            memory_junction_temp_c: Some(60.0 + variation),
            power_w: Some(120.0 + variation * 5.0),
            core_clock_mhz: Some(2500.0 + variation * 50.0),
            memory_clock_mhz: Some(10000.0),
            usage_percent: Some(15.0 + variation * 3.0),
            vram_used_mb: Some(2048.0),
            vram_total_mb: Some(12288.0),
            fan_speed_rpm: Some(1200.0 + variation * 100.0),
            fan_speed_percent: Some(35.0),
        },
        storage: StorageData {
            name: Some("S.M.A.R.T.: Lexar SSD NQ780 2TB [C:]".to_string()),
            nvme_temp_c: Some(38.0 + variation * 0.5),
            smart_health: "good".to_string(),
        },
        drives: vec![
            DriveData {
                name: Some("S.M.A.R.T.: Lexar SSD NQ780 2TB [C:]".to_string()),
                drive_letter: Some("C:".to_string()),
                temp_c: Some(38.0 + variation * 0.5),
                smart_health: "good".to_string(),
                total_gb: Some(1863.0),
                free_gb: Some(1245.0),
            },
            DriveData {
                name: Some("S.M.A.R.T.: Samsung 970 EVO 1TB [D:]".to_string()),
                drive_letter: Some("D:".to_string()),
                temp_c: Some(35.0),
                smart_health: "good".to_string(),
                total_gb: Some(931.0),
                free_gb: Some(512.0),
            },
        ],
        system: SystemData {
            name: Some("DESKTOP-PC".to_string()),
            uptime_seconds: Some(uptime_base + elapsed_secs),
            fan_status: "ok".to_string(),
            fans: vec![
                FanReading { name: "CPU Fan".to_string(), rpm: 1100.0 },
                FanReading { name: "Chassis Fan 1".to_string(), rpm: 900.0 },
            ],
        },
    }
}
