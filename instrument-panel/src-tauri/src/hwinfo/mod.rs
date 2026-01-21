pub mod process;
pub mod shared_memory;
pub mod types;
pub mod mock;

use types::{SensorData, Diagnostics, CpuData, GpuData, StorageData, SystemData};

/// Main entry point: read sensor data from HWiNFO
pub fn read_sensor_data() -> SensorData {
    let process_running = process::is_running();
    let shm_result = shared_memory::read();
    
    match shm_result {
        Ok(data) => data,
        Err(msg) => {
            // Return not_connected state with diagnostics
            SensorData {
                status: "not_connected".to_string(),
                last_read_at: None,
                diagnostics: Diagnostics {
                    hwinfo_process_detected: process_running,
                    shared_memory_detected: false,
                    message: Some(msg),
                },
                cpu: CpuData::default(),
                gpu: GpuData::default(),
                storage: StorageData::default(),
                system: SystemData::default(),
            }
        }
    }
}
