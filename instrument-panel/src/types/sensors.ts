// Matches the Rust SensorData struct

export interface SensorData {
  status: "connected" | "not_connected"
  lastReadAt: string | null
  diagnostics: {
    hwinfoProcessDetected: boolean
    sharedMemoryDetected: boolean
    message?: string
  }
  cpu: {
    name: string | null
    packageTempC: number | null
    packagePowerW: number | null
  }
  gpu: {
    name: string | null
    hotspotTempC: number | null
    memoryJunctionTempC: number | null
    powerW: number | null
  }
  storage: {
    name: string | null
    nvmeTempC: number | null
    smartHealth: "good" | "warning" | "unknown"
  }
  system: {
    name: string | null
    uptimeSeconds: number | null
    fanStatus: "ok" | "warning" | "unknown"
  }
}

export interface AppSettings {
  autoLaunchHwinfo: boolean
  hwinfoPath: string | null
  mockMode: boolean
}
