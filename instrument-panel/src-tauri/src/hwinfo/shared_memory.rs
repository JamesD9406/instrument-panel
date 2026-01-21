use super::types::*;
use chrono::Utc;
use std::ffi::CString;
use std::ptr;
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::Memory::{
    MapViewOfFile, OpenFileMappingA, UnmapViewOfFile, FILE_MAP_READ,
};

/// Read sensor data from HWiNFO shared memory
pub fn read() -> Result<SensorData, String> {
    unsafe { read_shared_memory() }
}

unsafe fn read_shared_memory() -> Result<SensorData, String> {
    // Open the shared memory mapping
    let shm_name = CString::new(HWINFO_SHM_NAME).unwrap();
    
    let handle: HANDLE = OpenFileMappingA(
        FILE_MAP_READ.0,
        false,
        windows::core::PCSTR(shm_name.as_ptr() as *const u8),
    ).map_err(|e| format!("Failed to open shared memory: {}. Is HWiNFO running with Shared Memory enabled?", e))?;

    if handle.is_invalid() {
        return Err("Shared memory not available. Enable it in HWiNFO settings.".to_string());
    }

    // Map view of the file
    let map_view = MapViewOfFile(handle, FILE_MAP_READ, 0, 0, 0);
    
    if map_view.Value.is_null() {
        CloseHandle(handle).ok();
        return Err("Failed to map shared memory view".to_string());
    }

    let base_ptr = map_view.Value as *const u8;

    // Read header
    let header = ptr::read_unaligned(base_ptr as *const HWiNFOHeader);

    // Verify signature
    if header.signature != HWINFO_SIGNATURE {
        UnmapViewOfFile(map_view).ok();
        CloseHandle(handle).ok();
        return Err(format!("Invalid HWiNFO signature: {:#X}", header.signature));
    }

    // Read all sensor readings
    let mut cpu_temp: Option<f64> = None;
    let mut cpu_power: Option<f64> = None;
    let mut gpu_hotspot: Option<f64> = None;
    let mut gpu_mem_junction: Option<f64> = None;
    let mut gpu_power: Option<f64> = None;
    let mut nvme_temp: Option<f64> = None;
    let mut uptime_seconds: Option<u64> = None;

    // Parse readings
    for i in 0..header.reading_count {
        let reading_ptr = base_ptr
            .add(header.reading_section_offset as usize)
            .add((i as usize) * header.reading_section_size as usize);
        
        let reading = ptr::read_unaligned(reading_ptr as *const HWiNFOReading);
        
        // Convert label to string for matching
        let label = String::from_utf8_lossy(&reading.label_original)
            .trim_end_matches('\0')
            .to_lowercase();

        // Match readings to our desired sensors
        // CPU Package Temperature
        if label.contains("cpu package") && label.contains("temp") 
            || label.contains("cpu (tctl/tdie)") {
            if reading.reading_type == ReadingType::Temp as u32 {
                cpu_temp = Some(reading.value);
            }
        }
        
        // CPU Package Power
        if (label.contains("cpu package power") || label.contains("cpu ppt"))
            && reading.reading_type == ReadingType::Power as u32 {
            cpu_power = Some(reading.value);
        }

        // GPU Hotspot Temperature
        if label.contains("gpu hot spot") || label.contains("gpu hotspot") {
            if reading.reading_type == ReadingType::Temp as u32 {
                gpu_hotspot = Some(reading.value);
            }
        }

        // GPU Memory Junction Temperature
        if label.contains("gpu memory junction") {
            if reading.reading_type == ReadingType::Temp as u32 {
                gpu_mem_junction = Some(reading.value);
            }
        }

        // GPU Power
        if label.contains("gpu power") && !label.contains("limit") {
            if reading.reading_type == ReadingType::Power as u32 {
                gpu_power = Some(reading.value);
            }
        }

        // NVMe Temperature
        if label.contains("drive temperature") || label.contains("nvme") && label.contains("temp") {
            if reading.reading_type == ReadingType::Temp as u32 && nvme_temp.is_none() {
                nvme_temp = Some(reading.value);
            }
        }
    }

    // Get system uptime from sysinfo
    uptime_seconds = Some(sysinfo::System::uptime());

    // Clean up
    UnmapViewOfFile(map_view).ok();
    CloseHandle(handle).ok();

    Ok(SensorData {
        status: "connected".to_string(),
        last_read_at: Some(Utc::now().to_rfc3339()),
        diagnostics: Diagnostics {
            hwinfo_process_detected: true,
            shared_memory_detected: true,
            message: None,
        },
        cpu: CpuData {
            package_temp_c: cpu_temp,
            package_power_w: cpu_power,
        },
        gpu: GpuData {
            hotspot_temp_c: gpu_hotspot,
            memory_junction_temp_c: gpu_mem_junction,
            power_w: gpu_power,
        },
        storage: StorageData {
            nvme_temp_c: nvme_temp,
            smart_health: "unknown".to_string(), // Would need deeper parsing
        },
        system: SystemData {
            uptime_seconds,
            fan_status: "unknown".to_string(), // Would need fan sensor parsing
        },
    })
}
