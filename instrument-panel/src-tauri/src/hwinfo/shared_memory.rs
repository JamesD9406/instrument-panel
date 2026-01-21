use super::types::*;
use crate::commands::{DebugDumpResult, HeaderDebugInfo, SensorDebugInfo, ReadingDebugInfo};
use chrono::Utc;
use std::ffi::CString;
use std::ptr;
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::Memory::{
    MapViewOfFile, OpenFileMappingA, UnmapViewOfFile, FILE_MAP_READ,
};
use windows::Win32::System::SystemInformation::GetTickCount64;

/// Get the system uptime using GetTickCount64
/// This returns milliseconds since the system was started.
/// Note: This persists through sleep but should reset on true shutdown/restart.
/// Windows Fast Startup (hybrid shutdown) may cause this to persist - disable Fast Startup
/// in Windows settings if you want accurate uptime after "shutdown".
fn get_true_uptime_seconds() -> Option<u64> {
    let tick_count_ms = unsafe { GetTickCount64() };
    Some(tick_count_ms / 1000)
}

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

    // Copy fields to local vars to avoid unaligned references from packed struct
    let sig = header.signature;
    let sensor_section_offset = header.sensor_section_offset;
    let sensor_section_size = header.sensor_section_size;
    let sensor_count = header.sensor_count;
    let reading_section_offset = header.reading_section_offset;
    let reading_section_size = header.reading_section_size;
    let reading_count = header.reading_count;

    // Verify signature
    if sig != HWINFO_SIGNATURE {
        UnmapViewOfFile(map_view).ok();
        CloseHandle(handle).ok();
        return Err(format!("Invalid HWiNFO signature: {:#X}", sig));
    }

    // Read all sensor readings
    let mut cpu_temp: Option<f64> = None;
    let mut cpu_power: Option<f64> = None;
    let mut cpu_name: Option<String> = None;
    let mut gpu_hotspot: Option<f64> = None;
    let mut gpu_mem_junction: Option<f64> = None;
    let mut gpu_power: Option<f64> = None;
    let mut gpu_name: Option<String> = None;
    let mut gpu_sensor_index: Option<u32> = None;
    let mut nvme_temp: Option<f64> = None;
    let mut storage_name: Option<String> = None;
    let mut storage_sensor_index: Option<u32> = None;
    let mut fan_readings: Vec<f64> = Vec::new();

    // Read sensor names from sensor section
    for i in 0..sensor_count {
        let sensor_ptr = base_ptr
            .add(sensor_section_offset as usize)
            .add((i as usize) * sensor_section_size as usize);

        let sensor = ptr::read_unaligned(sensor_ptr as *const HWiNFOSensor);

        let sensor_name = String::from_utf8_lossy(&sensor.sensor_name_original)
            .trim_end_matches('\0')
            .to_string();
        let sensor_name_lower = sensor_name.to_lowercase();

        // CPU name - look for AMD Ryzen or Intel Core processors
        if cpu_name.is_none() {
            if sensor_name_lower.contains("ryzen") || sensor_name_lower.contains("intel") || sensor_name_lower.contains("core i") {
                cpu_name = Some(sensor_name.clone());
            }
        }

        // GPU name - prioritize discrete GPUs (NVIDIA) over integrated (AMD Radeon)
        // Check for NVIDIA discrete GPU first
        if sensor_name_lower.contains("geforce") || sensor_name_lower.contains("rtx") || sensor_name_lower.contains("gtx") {
            gpu_name = Some(sensor_name.clone());
            gpu_sensor_index = Some(i);
        }
        // Only use AMD Radeon if no NVIDIA GPU found
        else if gpu_name.is_none() && sensor_name_lower.contains("radeon") {
            gpu_name = Some(sensor_name.clone());
            gpu_sensor_index = Some(i);
        }

        // Storage name - prefer NVMe on C: drive (OS drive), then any NVMe, then SATA
        // Look for S.M.A.R.T. sensors which have temperature readings
        if sensor_name_lower.starts_with("s.m.a.r.t.") {
            // Prefer C: drive (OS/game drive)
            if sensor_name.contains("[C:]") {
                storage_name = Some(sensor_name.clone());
                storage_sensor_index = Some(i);
            }
            // Otherwise take first NVMe drive if no C: found yet
            else if storage_name.is_none() || !storage_name.as_ref().map_or(false, |n| n.contains("[C:]")) {
                if sensor_name_lower.contains("nvme") || sensor_name_lower.contains("nq780") || sensor_name_lower.contains("980") || sensor_name_lower.contains("lexar") {
                    storage_name = Some(sensor_name.clone());
                    storage_sensor_index = Some(i);
                }
            }
            // Fallback to any drive if nothing else found
            if storage_name.is_none() {
                storage_name = Some(sensor_name.clone());
                storage_sensor_index = Some(i);
            }
        }
    }

    // Parse readings
    for i in 0..reading_count {
        let reading_ptr = base_ptr
            .add(reading_section_offset as usize)
            .add((i as usize) * reading_section_size as usize);
        
        let reading = ptr::read_unaligned(reading_ptr as *const HWiNFOReading);
        
        // Convert label to string for matching
        let label = String::from_utf8_lossy(&reading.label_original)
            .trim_end_matches('\0')
            .to_lowercase();

        // Match readings to our desired sensors
        // CPU Package Temperature - match various labels HWiNFO uses
        // "CPU Temp" (AMD Enhanced), "CPU (Tctl/Tdie)" (older AMD), "CPU Package" (Intel)
        if cpu_temp.is_none() && reading.reading_type == ReadingType::Temp as u32 {
            if label == "cpu temp"
                || label.contains("cpu (tctl/tdie)")
                || (label.contains("cpu") && label.contains("package") && label.contains("temp")) {
                cpu_temp = Some(reading.value);
            }
        }

        // CPU Package Power - "CPU Power" (AMD Enhanced), "CPU Package Power" (Intel), "CPU PPT"
        if cpu_power.is_none() && reading.reading_type == ReadingType::Power as u32 {
            if label == "cpu power"
                || label.contains("cpu package power")
                || label == "cpu ppt" {
                cpu_power = Some(reading.value);
            }
        }

        // GPU readings - only from the selected GPU sensor
        let is_target_gpu = gpu_sensor_index.map_or(false, |idx| reading.sensor_index == idx);

        // GPU Temperature (main/hotspot) - "GPU Temp", "GPU Temperature", or "GPU Hot Spot"
        if is_target_gpu && gpu_hotspot.is_none() && reading.reading_type == ReadingType::Temp as u32 {
            if label == "gpu temp" || label == "gpu temperature" || label.contains("gpu hot spot") || label.contains("hotspot") {
                gpu_hotspot = Some(reading.value);
            }
        }

        // GPU Memory Junction Temperature
        if is_target_gpu && label.contains("memory junction") {
            if reading.reading_type == ReadingType::Temp as u32 {
                gpu_mem_junction = Some(reading.value);
            }
        }

        // GPU Power - look for "GPU Power" but not limits/percentages
        if is_target_gpu && gpu_power.is_none() {
            if label == "gpu power" || (label.contains("gpu") && label.contains("power") && !label.contains("limit") && !label.contains("percent")) {
                if reading.reading_type == ReadingType::Power as u32 {
                    gpu_power = Some(reading.value);
                }
            }
        }

        // Storage/Drive Temperature - from the selected storage sensor
        let is_target_storage = storage_sensor_index.map_or(false, |idx| reading.sensor_index == idx);
        if is_target_storage && nvme_temp.is_none() && reading.reading_type == ReadingType::Temp as u32 {
            // Look for drive temperature reading - HWiNFO uses various labels
            // "Drive Temperature", "Drive Airflow Temperature", etc.
            // Prefer the main "Drive Temperature" over secondary sensors
            if label == "drive temperature" || label.contains("drive") {
                nvme_temp = Some(reading.value);
            }
        }

        // Fan readings - collect all fan RPM values to determine status
        if reading.reading_type == ReadingType::Fan as u32 {
            // Only count non-zero fan readings (0 RPM usually means stopped or not detected)
            if reading.value > 0.0 {
                fan_readings.push(reading.value);
            }
        }
    }

    // Get system uptime using Windows GetTickCount64 for more accurate boot time
    let uptime_seconds = get_true_uptime_seconds();

    // Get PC name from sysinfo
    let pc_name = sysinfo::System::host_name();

    // Determine fan status based on collected readings
    let fan_status = if fan_readings.is_empty() {
        "unknown".to_string()
    } else {
        // Check if any fans are running at concerning speeds
        // Very low RPM (< 200) might indicate stalled fan
        // Very high RPM (> 3000 for most fans) might indicate high load
        let has_stalled = fan_readings.iter().any(|&rpm| rpm < 200.0);
        let _has_high = fan_readings.iter().any(|&rpm| rpm > 3000.0);

        if has_stalled {
            "warning".to_string()
        } else {
            "ok".to_string()
        }
    };

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
            name: cpu_name,
            package_temp_c: cpu_temp,
            package_power_w: cpu_power,
        },
        gpu: GpuData {
            name: gpu_name,
            hotspot_temp_c: gpu_hotspot,
            memory_junction_temp_c: gpu_mem_junction,
            power_w: gpu_power,
        },
        storage: StorageData {
            name: storage_name,
            nvme_temp_c: nvme_temp,
            smart_health: "unknown".to_string(), // Would need deeper parsing
        },
        system: SystemData {
            name: pc_name,
            uptime_seconds,
            fan_status,
        },
    })
}

/// Debug function to dump all sensor info
pub fn debug_dump_sensors() -> Result<DebugDumpResult, String> {
    unsafe { debug_dump_sensors_inner() }
}

unsafe fn debug_dump_sensors_inner() -> Result<DebugDumpResult, String> {
    let shm_name = CString::new(HWINFO_SHM_NAME).unwrap();

    let handle: HANDLE = OpenFileMappingA(
        FILE_MAP_READ.0,
        false,
        windows::core::PCSTR(shm_name.as_ptr() as *const u8),
    ).map_err(|e| format!("Failed to open shared memory: {}", e))?;

    if handle.is_invalid() {
        return Err("Shared memory not available".to_string());
    }

    let map_view = MapViewOfFile(handle, FILE_MAP_READ, 0, 0, 0);

    if map_view.Value.is_null() {
        CloseHandle(handle).ok();
        return Err("Failed to map shared memory view".to_string());
    }

    let base_ptr = map_view.Value as *const u8;
    let header = ptr::read_unaligned(base_ptr as *const HWiNFOHeader);

    // Copy fields to local vars to avoid unaligned references from packed struct
    let sig = header.signature;
    let version = header.version;
    let revision = header.revision;
    let sensor_section_offset = header.sensor_section_offset;
    let sensor_section_size = header.sensor_section_size;
    let sensor_count = header.sensor_count;
    let reading_section_offset = header.reading_section_offset;
    let reading_section_size = header.reading_section_size;
    let reading_count = header.reading_count;

    if sig != HWINFO_SIGNATURE {
        UnmapViewOfFile(map_view).ok();
        CloseHandle(handle).ok();
        return Err(format!("Invalid signature: {:#X}", sig));
    }

    // Build header debug info
    let header_info = HeaderDebugInfo {
        signature: format!("{:#X}", sig),
        version,
        revision,
        sensor_section_offset,
        sensor_section_size,
        sensor_count,
        reading_section_offset,
        reading_section_size,
        reading_count,
    };

    let mut sensors = Vec::new();

    for i in 0..sensor_count {
        let sensor_ptr = base_ptr
            .add(sensor_section_offset as usize)
            .add((i as usize) * sensor_section_size as usize);

        let sensor = ptr::read_unaligned(sensor_ptr as *const HWiNFOSensor);

        let name_original = String::from_utf8_lossy(&sensor.sensor_name_original)
            .trim_end_matches('\0')
            .to_string();
        let name_user = String::from_utf8_lossy(&sensor.sensor_name_user)
            .trim_end_matches('\0')
            .to_string();

        sensors.push(SensorDebugInfo {
            index: i,
            sensor_id: sensor.sensor_id,
            sensor_instance: sensor.sensor_instance,
            name_original,
            name_user,
        });
    }

    UnmapViewOfFile(map_view).ok();
    CloseHandle(handle).ok();

    Ok(DebugDumpResult {
        header: header_info,
        sensors,
    })
}

/// Debug function to dump all readings from HWiNFO shared memory
/// Optional filter to search for specific labels (case-insensitive)
pub fn debug_dump_readings(filter: Option<String>) -> Result<Vec<ReadingDebugInfo>, String> {
    unsafe { debug_dump_readings_inner(filter) }
}

unsafe fn debug_dump_readings_inner(filter: Option<String>) -> Result<Vec<ReadingDebugInfo>, String> {
    let shm_name = CString::new(HWINFO_SHM_NAME).unwrap();

    let handle: HANDLE = OpenFileMappingA(
        FILE_MAP_READ.0,
        false,
        windows::core::PCSTR(shm_name.as_ptr() as *const u8),
    ).map_err(|e| format!("Failed to open shared memory: {}", e))?;

    if handle.is_invalid() {
        return Err("Shared memory not available".to_string());
    }

    let map_view = MapViewOfFile(handle, FILE_MAP_READ, 0, 0, 0);

    if map_view.Value.is_null() {
        CloseHandle(handle).ok();
        return Err("Failed to map shared memory view".to_string());
    }

    let base_ptr = map_view.Value as *const u8;
    let header = ptr::read_unaligned(base_ptr as *const HWiNFOHeader);

    // Copy fields to local vars to avoid unaligned references from packed struct
    let sig = header.signature;
    let reading_section_offset = header.reading_section_offset;
    let reading_section_size = header.reading_section_size;
    let reading_count = header.reading_count;

    if sig != HWINFO_SIGNATURE {
        UnmapViewOfFile(map_view).ok();
        CloseHandle(handle).ok();
        return Err(format!("Invalid signature: {:#X}", sig));
    }

    let filter_lower = filter.map(|f| f.to_lowercase());
    let mut readings = Vec::new();

    for i in 0..reading_count {
        let reading_ptr = base_ptr
            .add(reading_section_offset as usize)
            .add((i as usize) * reading_section_size as usize);

        let reading = ptr::read_unaligned(reading_ptr as *const HWiNFOReading);

        let label_original = String::from_utf8_lossy(&reading.label_original)
            .trim_end_matches('\0')
            .to_string();
        let label_user = String::from_utf8_lossy(&reading.label_user)
            .trim_end_matches('\0')
            .to_string();
        let unit = String::from_utf8_lossy(&reading.unit)
            .trim_end_matches('\0')
            .to_string();

        // Apply filter if provided
        if let Some(ref f) = filter_lower {
            let label_lower = label_original.to_lowercase();
            if !label_lower.contains(f) {
                continue;
            }
        }

        readings.push(ReadingDebugInfo {
            index: i,
            sensor_index: reading.sensor_index,
            reading_type: reading.reading_type,
            label_original,
            label_user,
            unit,
            value: reading.value,
        });
    }

    UnmapViewOfFile(map_view).ok();
    CloseHandle(handle).ok();

    Ok(readings)
}
