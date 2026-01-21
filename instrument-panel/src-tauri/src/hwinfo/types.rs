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
    pub name: Option<String>,
    pub package_temp_c: Option<f64>,
    pub package_power_w: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GpuData {
    pub name: Option<String>,
    pub hotspot_temp_c: Option<f64>,
    pub memory_junction_temp_c: Option<f64>,
    pub power_w: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageData {
    pub name: Option<String>,
    pub nvme_temp_c: Option<f64>,
    pub smart_health: String, // "good" | "warning" | "unknown"
}

impl Default for StorageData {
    fn default() -> Self {
        Self {
            name: None,
            nvme_temp_c: None,
            smart_health: "unknown".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemData {
    pub name: Option<String>,
    pub uptime_seconds: Option<u64>,
    pub fan_status: String, // "ok" | "warning" | "unknown"
}

impl Default for SystemData {
    fn default() -> Self {
        Self {
            name: None,
            uptime_seconds: None,
            fan_status: "unknown".to_string(),
        }
    }
}

// ============================================================
// HWiNFO Shared Memory Structures
// Based on HWiNFO SDK documentation
// All structs use packed alignment (no padding)
// ============================================================

/// HWiNFO shared memory header
/// Total size: 44 bytes (0x2C)
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct HWiNFOHeader {
    pub signature: u32,              // 0x00: "HWiS" = 0x53695748
    pub version: u32,                // 0x04: Structure version
    pub revision: u32,               // 0x08: Structure revision
    pub poll_time: i64,              // 0x0C: Last polling time (ms since system start)
    pub sensor_section_offset: u32,  // 0x14: Byte offset to sensor array
    pub sensor_section_size: u32,    // 0x18: Size of each sensor entry
    pub sensor_count: u32,           // 0x1C: Number of sensors
    pub reading_section_offset: u32, // 0x20: Byte offset to readings array
    pub reading_section_size: u32,   // 0x24: Size of each reading entry
    pub reading_count: u32,          // 0x28: Number of readings
}

/// A single sensor entry
/// Total size: 264 bytes (0x108)
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct HWiNFOSensor {
    pub sensor_id: u32,                   // 0x00: Sensor ID
    pub sensor_instance: u32,             // 0x04: Sensor instance
    pub sensor_name_original: [u8; 128],  // 0x08: Original sensor name
    pub sensor_name_user: [u8; 128],      // 0x88: User-customized sensor name
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
/// Total size: 312 bytes (0x138)
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct HWiNFOReading {
    pub reading_type: u32,           // 0x00: Reading type (see ReadingType enum)
    pub sensor_index: u32,           // 0x04: Index into sensor array
    pub reading_id: u32,             // 0x08: Reading ID
    pub label_original: [u8; 128],   // 0x0C: Original reading label
    pub label_user: [u8; 128],       // 0x8C: User-customized label
    pub unit: [u8; 16],              // 0x10C: Unit string
    pub value: f64,                  // 0x11C: Current value
    pub value_min: f64,              // 0x124: Minimum value
    pub value_max: f64,              // 0x12C: Maximum value
    pub value_avg: f64,              // 0x134: Average value
}

pub const HWINFO_SIGNATURE: u32 = 0x53695748; // "HWiS" in ASCII (little-endian: bytes 48 57 69 53)
pub const HWINFO_SHM_NAME: &str = "Global\\HWiNFO_SENS_SM2";