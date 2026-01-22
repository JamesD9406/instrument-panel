// Matches the Rust SensorData struct

export interface SensorData {
  status: "connected" | "not_connected"
  lastReadAt: string | null
  diagnostics: {
    hwinfoProcessDetected: boolean
    sharedMemoryDetected: boolean
    message?: string
  }
  cpu: CpuData
  gpu: GpuData
  storage: StorageData
  drives: DriveData[]
  system: SystemData
}

export interface CpuData {
  name: string | null
  packageTempC: number | null
  packagePowerW: number | null
  coreClockMhz: number | null
  usagePercent: number | null
  coreTemps: number[]
}

export interface GpuData {
  name: string | null
  hotspotTempC: number | null
  memoryJunctionTempC: number | null
  powerW: number | null
  coreClockMhz: number | null
  memoryClockMhz: number | null
  usagePercent: number | null
  vramUsedMb: number | null
  vramTotalMb: number | null
  fanSpeedRpm: number | null
  fanSpeedPercent: number | null
}

export interface StorageData {
  name: string | null
  nvmeTempC: number | null
  smartHealth: "good" | "warning" | "critical" | "unknown"
}

export interface DriveData {
  name: string | null
  driveLetter: string | null
  tempC: number | null
  smartHealth: "good" | "warning" | "critical" | "unknown"
  totalGb: number | null
  freeGb: number | null
}

export interface SystemData {
  name: string | null
  uptimeSeconds: number | null
  fanStatus: "ok" | "warning" | "unknown"
  fans: FanReading[]
}

export interface FanReading {
  name: string
  rpm: number
}

export interface AppSettings {
  autoLaunchHwinfo: boolean
  hwinfoPath: string | null
  mockMode: boolean
}
