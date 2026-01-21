use super::types::*;
use chrono::Utc;

/// Generate realistic mock sensor data for UI development
pub fn generate_mock_data() -> SensorData {
    use std::time::{SystemTime, UNIX_EPOCH};
    
    // Use time-based pseudo-random variation
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    
    let variation = ((now % 1000) as f64 / 1000.0) * 10.0 - 5.0; // -5 to +5
    
    SensorData {
        status: "connected".to_string(),
        last_read_at: Some(Utc::now().to_rfc3339()),
        diagnostics: Diagnostics {
            hwinfo_process_detected: true,
            shared_memory_detected: true,
            message: Some("Mock mode enabled".to_string()),
        },
        cpu: CpuData {
            package_temp_c: Some(55.0 + variation),
            package_power_w: Some(65.0 + variation * 2.0),
        },
        gpu: GpuData {
            hotspot_temp_c: Some(72.0 + variation),
            memory_junction_temp_c: Some(68.0 + variation * 0.8),
            power_w: Some(180.0 + variation * 5.0),
        },
        storage: StorageData {
            nvme_temp_c: Some(42.0 + variation * 0.5),
            smart_health: "good".to_string(),
        },
        system: SystemData {
            uptime_seconds: Some(sysinfo::System::uptime()),
            fan_status: "ok".to_string(),
        },
    }
}
