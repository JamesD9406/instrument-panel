use serde::{Deserialize, Serialize};

/// Main sensor data structure returned to frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SensorData {
    pub status: String, // "connected" | "not_connected"
    pub last_read_at: Option<String>, // ISO-8601 timestamp
    pub diagnostics: Diagnostics,
    pub cpu: CpuData,
    pub gpu: GpuData,
    pub storage: StorageData,
    pub system: SystemData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostics {
    pub hwinfo_process_detected: bool,
    pub shared_memory_detected: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CpuData {
    pub package_temp_c: Option<f64>,
    pub package_power_w: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GpuData {
    pub hotspot_temp_c: Option<f64>,
    pub memory_junction_temp_c: Option<f64>,
    pub power_w: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageData {
    pub nvme_temp_c: Option<f64>,
    pub smart_health: String, // "good" | "warning" | "unknown"
}

impl Default for StorageData {
    fn default() -> Self {
        Self {
            nvme_temp_c: None,
            smart_health: "unknown".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemData {
    pub uptime_seconds: Option<u64>,
    pub fan_status: String, // "ok" | "warning" | "unknown"
}

impl Default for SystemData {
    fn default() -> Self {
        Self {
            uptime_seconds: None,
            fan_status: "unknown".to_string(),
        }
    }
}

// ============================================================
// HWiNFO Shared Memory Structures
// Based on HWiNFO SDK documentation
// ============================================================

/// HWiNFO shared memory header
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct HWiNFOHeader {
    pub signature: u32,           // "SiWH" = 0x48576953
    pub version: u32,             // Structure version
    pub revision: u32,            // Structure revision
    pub poll_time: i64,           // Last polling time (ms since system start)
    pub sensor_section_offset: u32,
    pub sensor_section_size: u32,
    pub sensor_count: u32,
    pub reading_section_offset: u32,
    pub reading_section_size: u32,
    pub reading_count: u32,
}

/// A single sensor entry
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct HWiNFOSensor {
    pub sensor_id: u32,
    pub sensor_instance: u32,
    pub sensor_name_original: [u8; 128],
    pub sensor_name_user: [u8; 128],
}

/// Sensor reading types
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReadingType {
    None = 0,
    Temp = 1,
    Voltage = 2,
    Fan = 3,
    Current = 4,
    Power = 5,
    Clock = 6,
    Usage = 7,
    Other = 8,
}

/// A single sensor reading
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct HWiNFOReading {
    pub reading_type: u32,
    pub sensor_index: u32,
    pub reading_id: u32,
    pub label_original: [u8; 128],
    pub label_user: [u8; 128],
    pub unit: [u8; 16],
    pub value: f64,
    pub value_min: f64,
    pub value_max: f64,
    pub value_avg: f64,
}

pub const HWINFO_SIGNATURE: u32 = 0x48576953; // "SiWH" in little-endian
pub const HWINFO_SHM_NAME: &str = "Global\\HWiNFO_SENS_SM2";