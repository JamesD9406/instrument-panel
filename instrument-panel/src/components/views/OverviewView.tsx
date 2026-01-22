import { SensorData } from "../../types/sensors"

interface Props {
  data: SensorData | null
}

function formatTemp(value: number | null | undefined): string {
  if (value === null || value === undefined) return "—"
  return `${Math.round(value)}°C`
}

function formatPower(value: number | null | undefined): string {
  if (value === null || value === undefined) return "—"
  return `${Math.round(value)}W`
}

function formatUptime(seconds: number | null | undefined): string {
  if (!seconds) return "—"
  const hours = Math.floor(seconds / 3600)
  const mins = Math.floor((seconds % 3600) / 60)
  if (hours > 24) {
    const days = Math.floor(hours / 24)
    const remainingHours = hours % 24
    return `${days}d ${remainingHours}h`
  }
  return `${hours}h ${mins}m`
}

function shortenName(name: string | null | undefined, maxLen = 20): string {
  if (!name) return "—"
  if (name.length <= maxLen) return name
  return name.substring(0, maxLen - 1) + "…"
}

function cleanGpuName(name: string | null | undefined): string {
  if (!name) return ""
  // Remove "GPU [#0]: " or similar prefixes
  return name.replace(/^GPU\s*\[#\d+\]:\s*/i, "").trim()
}

function cleanCpuName(name: string | null | undefined): string {
  if (!name) return ""
  // Remove "CPU [#0]: " prefix if present
  return name.replace(/^CPU\s*\[#\d+\]:\s*/i, "").trim()
}

export function OverviewView({ data }: Props) {
  const cpu = data?.cpu
  const gpu = data?.gpu
  const storage = data?.storage
  const drives = data?.drives ?? []
  const system = data?.system

  // Get primary drive letter (prefer C:)
  const primaryDrive =
    drives.find((d) => d.driveLetter === "C:") || drives[0]
  const driveLetter = primaryDrive?.driveLetter || storage?.name || "Storage"

  const cpuTempWarning = (cpu?.packageTempC ?? 0) >= 85
  const gpuTempWarning = (gpu?.hotspotTempC ?? 0) >= 95
  const storageTempWarning = (primaryDrive?.tempC ?? storage?.nvmeTempC ?? 0) >= 70

  return (
    <div className="overview-grid">
      <div className={`overview-card ${cpuTempWarning ? "warning" : ""}`}>
        <div className="overview-card-header">CPU</div>
        <div className="overview-card-name">{shortenName(cleanCpuName(cpu?.name))}</div>
        <div className="overview-card-main">
          {formatTemp(cpu?.packageTempC)}
        </div>
        <div className="overview-card-secondary">
          {formatPower(cpu?.packagePowerW)}
        </div>
      </div>

      <div className={`overview-card ${gpuTempWarning ? "warning" : ""}`}>
        <div className="overview-card-header">GPU</div>
        <div className="overview-card-name">{shortenName(cleanGpuName(gpu?.name))}</div>
        <div className="overview-card-main">
          {formatTemp(gpu?.hotspotTempC)}
        </div>
        <div className="overview-card-secondary">
          {formatPower(gpu?.powerW)}
        </div>
      </div>

      <div className={`overview-card ${storageTempWarning ? "warning" : ""}`}>
        <div className="overview-card-header">{driveLetter}</div>
        <div className="overview-card-name">
          {shortenName(
            primaryDrive?.name?.replace(/^S\.M\.A\.R\.T\.:\s*/i, "").replace(/\s*\[[A-Z]:\].*$/, "")
          )}
        </div>
        <div className="overview-card-main">
          {formatTemp(primaryDrive?.tempC ?? storage?.nvmeTempC)}
        </div>
        <div className="overview-card-secondary">
          {primaryDrive?.smartHealth || storage?.smartHealth || "—"}
        </div>
      </div>

      <div className="overview-card">
        <div className="overview-card-header">System</div>
        <div className="overview-card-name">{shortenName(system?.name)}</div>
        <div className="overview-card-main">
          {formatUptime(system?.uptimeSeconds)}
        </div>
        <div className="overview-card-secondary">
          {system?.fanStatus || "—"}
        </div>
      </div>
    </div>
  )
}