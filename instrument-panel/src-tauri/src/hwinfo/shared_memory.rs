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

    // CPU data
    let mut cpu_temp: Option<f64> = None;
    let mut cpu_power: Option<f64> = None;
    let mut cpu_name: Option<String> = None;
    let mut cpu_clock: Option<f64> = None;
    let mut cpu_usage: Option<f64> = None;
    let mut core_temps: Vec<f64> = Vec::new();
    let mut cpu_sensor_index: Option<u32> = None;

    // GPU data
    let mut gpu_hotspot: Option<f64> = None;
    let mut gpu_mem_junction: Option<f64> = None;
    let mut gpu_power: Option<f64> = None;
    let mut gpu_name: Option<String> = None;
    let mut gpu_sensor_index: Option<u32> = None;
    let mut gpu_core_clock: Option<f64> = None;
    let mut gpu_mem_clock: Option<f64> = None;
    let mut gpu_usage: Option<f64> = None;
    let mut gpu_vram_used: Option<f64> = None;
    let mut gpu_vram_total: Option<f64> = None;
    let mut gpu_fan_rpm: Option<f64> = None;
    let mut gpu_fan_percent: Option<f64> = None;

    // Storage data - collect all drives
    let mut drives: Vec<(u32, String, Option<String>)> = Vec::new(); // (sensor_index, name, drive_letter)
    let mut drive_temps: std::collections::HashMap<u32, f64> = std::collections::HashMap::new();
    let mut drive_health: std::collections::HashMap<u32, String> = std::collections::HashMap::new();

    // Fan data
    let mut fan_readings: Vec<FanReading> = Vec::new();

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
                cpu_sensor_index = Some(i);
            }
        }

        // GPU name - prioritize discrete GPUs (NVIDIA) over integrated (AMD Radeon)
        if sensor_name_lower.contains("geforce") || sensor_name_lower.contains("rtx") || sensor_name_lower.contains("gtx") {
            gpu_name = Some(sensor_name.clone());
            gpu_sensor_index = Some(i);
        } else if gpu_name.is_none() && sensor_name_lower.contains("radeon") {
            gpu_name = Some(sensor_name.clone());
            gpu_sensor_index = Some(i);
        }

        // Storage - collect all S.M.A.R.T. sensors
        if sensor_name_lower.starts_with("s.m.a.r.t.") {
            // Extract drive letter if present
            let drive_letter = if let Some(start) = sensor_name.find('[') {
                if let Some(end) = sensor_name.find(']') {
                    Some(sensor_name[start+1..end].to_string())
                } else {
                    None
                }
            } else {
                None
            };
            drives.push((i, sensor_name.clone(), drive_letter));
        }
    }

    // Parse readings
    for i in 0..reading_count {
        let reading_ptr = base_ptr
            .add(reading_section_offset as usize)
            .add((i as usize) * reading_section_size as usize);
        
        let reading = ptr::read_unaligned(reading_ptr as *const HWiNFOReading);
        
        let label = String::from_utf8_lossy(&reading.label_original)
            .trim_end_matches('\0')
            .to_lowercase();
        let label_original = String::from_utf8_lossy(&reading.label_original)
            .trim_end_matches('\0')
            .to_string();

        let is_cpu = cpu_sensor_index.map_or(false, |idx| reading.sensor_index == idx);
        let is_gpu = gpu_sensor_index.map_or(false, |idx| reading.sensor_index == idx);

        // CPU readings
        if is_cpu {
            // Package temperature - AMD uses "CPU (Tctl/Tdie)" or just "Tctl" or "Tdie"
            if cpu_temp.is_none() && reading.reading_type == ReadingType::Temp as u32 {
                if label == "cpu temp" || label.contains("tctl") || label.contains("tdie")
                    || (label.contains("cpu") && label.contains("package")) {
                    cpu_temp = Some(reading.value);
                }
            }
            // Per-core temperatures - AMD uses "Core X (CCD Y)" or similar
            if reading.reading_type == ReadingType::Temp as u32 {
                if (label.starts_with("core") && (label.contains("temp") || label.contains("ccd")))
                    || label.contains("ccd") {
                    core_temps.push(reading.value);
                }
            }
            // CPU power - AMD uses "CPU PPT" (Package Power Tracking)
            if cpu_power.is_none() && reading.reading_type == ReadingType::Power as u32 {
                if label == "cpu power" || label.contains("cpu package power")
                    || label == "cpu ppt" || label.contains("ppt") {
                    cpu_power = Some(reading.value);
                }
            }
            // CPU clock (average or effective)
            if cpu_clock.is_none() && reading.reading_type == ReadingType::Clock as u32 {
                if label.contains("core") && (label.contains("clock") || label.contains("effective")) {
                    cpu_clock = Some(reading.value);
                }
            }
            // CPU usage
            if cpu_usage.is_none() && reading.reading_type == ReadingType::Usage as u32 {
                if label.contains("total") || label.contains("cpu") {
                    cpu_usage = Some(reading.value);
                }
            }
        }

        // GPU readings
        if is_gpu {
            // GPU Temperature (hotspot)
            if gpu_hotspot.is_none() && reading.reading_type == ReadingType::Temp as u32 {
                if label == "gpu temp" || label == "gpu temperature" 
                    || label.contains("gpu hot spot") || label.contains("hotspot") {
                    gpu_hotspot = Some(reading.value);
                }
            }
            // Memory Junction Temperature
            if gpu_mem_junction.is_none() && label.contains("memory junction") {
                if reading.reading_type == ReadingType::Temp as u32 {
                    gpu_mem_junction = Some(reading.value);
                }
            }
            // GPU Power
            if gpu_power.is_none() && reading.reading_type == ReadingType::Power as u32 {
                if label == "gpu power" || (label.contains("gpu") && label.contains("power") 
                    && !label.contains("limit") && !label.contains("percent")) {
                    gpu_power = Some(reading.value);
                }
            }
            // GPU Core Clock
            if gpu_core_clock.is_none() && reading.reading_type == ReadingType::Clock as u32 {
                if label == "gpu clock" || label.contains("core clock") {
                    gpu_core_clock = Some(reading.value);
                }
            }
            // GPU Memory Clock
            if gpu_mem_clock.is_none() && reading.reading_type == ReadingType::Clock as u32 {
                if label.contains("memory clock") || label.contains("mem clock") {
                    gpu_mem_clock = Some(reading.value);
                }
            }
            // GPU Usage
            if gpu_usage.is_none() && reading.reading_type == ReadingType::Usage as u32 {
                if label == "gpu utilization" || label.contains("gpu core load") || label == "gpu usage" {
                    gpu_usage = Some(reading.value);
                }
            }
            // VRAM Used
            if gpu_vram_used.is_none() && reading.reading_type == ReadingType::Other as u32 {
                if label.contains("gpu memory used") || label.contains("vram used") 
                    || label.contains("d3d dedicated") {
                    gpu_vram_used = Some(reading.value);
                }
            }
            // VRAM Total (often reported as "GPU Memory Allocated" or similar)
            if gpu_vram_total.is_none() && reading.reading_type == ReadingType::Other as u32 {
                if label.contains("gpu memory total") || label.contains("vram total") {
                    gpu_vram_total = Some(reading.value);
                }
            }
            // GPU Fan RPM
            if gpu_fan_rpm.is_none() && reading.reading_type == ReadingType::Fan as u32 {
                if label.contains("gpu") || label.contains("fan") {
                    gpu_fan_rpm = Some(reading.value);
                }
            }
            // GPU Fan %
            if gpu_fan_percent.is_none() && reading.reading_type == ReadingType::Usage as u32 {
                if label.contains("fan") && (label.contains("speed") || label.contains("%")) {
                    gpu_fan_percent = Some(reading.value);
                }
            }
        }

        // Storage readings - match by sensor index
        for (drive_idx, _, _) in &drives {
            if reading.sensor_index == *drive_idx {
                // Drive temperature - include "Drive Airflow Temperature" for SATA SSDs
                if reading.reading_type == ReadingType::Temp as u32 {
                    if label == "drive temperature" || label.contains("drive temp")
                        || label.contains("airflow") {
                        // Only store if we don't have a temp yet, or prefer non-airflow over airflow
                        if !drive_temps.contains_key(drive_idx) {
                            drive_temps.insert(*drive_idx, reading.value);
                        }
                    }
                }
                // SMART Health - look for remaining life or health indicators
                // 70%+ = good, 30-70% = warning, <30% = critical
                if label.contains("remaining life") || label.contains("health")
                    || label.contains("life remaining") {
                    let health = if reading.value >= 70.0 {
                        "good".to_string()
                    } else if reading.value >= 30.0 {
                        "warning".to_string()
                    } else {
                        "critical".to_string()
                    };
                    drive_health.insert(*drive_idx, health);
                }
            }
        }

        // Fan readings (non-GPU fans)
        if reading.reading_type == ReadingType::Fan as u32 && reading.value > 0.0 {
            if !is_gpu {
                fan_readings.push(FanReading {
                    name: label_original,
                    rpm: reading.value,
                });
            }
        }
    }

    // Build drive data
    let mut drive_data: Vec<DriveData> = Vec::new();
    let mut primary_storage = StorageData::default();

    for (idx, name, letter) in &drives {
        let temp = drive_temps.get(idx).copied();
        let health = drive_health.get(idx).cloned().unwrap_or_else(|| "unknown".to_string());
        
        // Get disk space info for this drive letter
        let (total_gb, free_gb) = if let Some(ref letter) = letter {
            get_disk_space(letter)
        } else {
            (None, None)
        };

        let drive = DriveData {
            name: Some(name.clone()),
            drive_letter: letter.clone(),
            temp_c: temp,
            smart_health: health.clone(),
            total_gb,
            free_gb,
        };

        // Set primary storage (prefer C: drive)
        if letter.as_ref().map_or(false, |l| l == "C:") {
            primary_storage = StorageData {
                name: Some(name.clone()),
                nvme_temp_c: temp,
                smart_health: health,
            };
        } else if primary_storage.name.is_none() {
            primary_storage = StorageData {
                name: Some(name.clone()),
                nvme_temp_c: temp,
                smart_health: health,
            };
        }

        drive_data.push(drive);
    }

    // Sort drives by letter
    drive_data.sort_by(|a, b| {
        a.drive_letter.as_deref().unwrap_or("Z")
            .cmp(b.drive_letter.as_deref().unwrap_or("Z"))
    });

    // Get system uptime
    let uptime_seconds = get_true_uptime_seconds();
    let pc_name = sysinfo::System::host_name();

    // Determine fan status
    let fan_status = if fan_readings.is_empty() {
        "unknown".to_string()
    } else {
        let has_stalled = fan_readings.iter().any(|f| f.rpm < 200.0);
        if has_stalled { "warning".to_string() } else { "ok".to_string() }
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
            core_clock_mhz: cpu_clock,
            usage_percent: cpu_usage,
            core_temps,
        },
        gpu: GpuData {
            name: gpu_name,
            hotspot_temp_c: gpu_hotspot,
            memory_junction_temp_c: gpu_mem_junction,
            power_w: gpu_power,
            core_clock_mhz: gpu_core_clock,
            memory_clock_mhz: gpu_mem_clock,
            usage_percent: gpu_usage,
            vram_used_mb: gpu_vram_used,
            vram_total_mb: gpu_vram_total,
            fan_speed_rpm: gpu_fan_rpm,
            fan_speed_percent: gpu_fan_percent,
        },
        storage: primary_storage,
        drives: drive_data,
        system: SystemData {
            name: pc_name,
            uptime_seconds,
            fan_status,
            fans: fan_readings,
        },
    })
}

/// Get disk space for a drive letter (e.g., "C:")
fn get_disk_space(drive_letter: &str) -> (Option<f64>, Option<f64>) {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStrExt;
    use windows::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;
    use windows::core::PCWSTR;

    let path = format!("{}\\", drive_letter);
    let wide: Vec<u16> = OsString::from(&path)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let mut free_bytes: u64 = 0;
    let mut total_bytes: u64 = 0;
    let mut _total_free: u64 = 0;

    let result = unsafe {
        GetDiskFreeSpaceExW(
            PCWSTR(wide.as_ptr()),
            Some(&mut free_bytes),
            Some(&mut total_bytes),
            Some(&mut _total_free),
        )
    };

    if result.is_ok() {
        let total_gb = total_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
        let free_gb = free_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
        (Some(total_gb), Some(free_gb))
    } else {
        (None, None)
    }
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
