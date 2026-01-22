import { SensorData } from "../../types/sensors"

interface Props {
  data: SensorData | null
}

function formatUptime(seconds: number | null): string {
  if (seconds === null) return "—"
  const days = Math.floor(seconds / 86400)
  const hours = Math.floor((seconds % 86400) / 3600)
  const mins = Math.floor((seconds % 3600) / 60)
  if (days > 0) return `${days}d ${hours}h`
  if (hours > 0) return `${hours}h ${mins}m`
  return `${mins}m`
}

function formatTemp(value: number | null | undefined): string {
  if (value === null || value === undefined) return "—"
  return `${Math.round(value)}°`
}

function formatPower(value: number | null | undefined): string {
  if (value === null || value === undefined) return "—"
  return `${Math.round(value)}W`
}

export function OverviewView({ data }: Props) {
  const cpu = data?.cpu
  const gpu = data?.gpu
  const storage = data?.storage
  const system = data?.system

  const cpuTempWarning = (cpu?.packageTempC ?? 0) >= 85
  const gpuTempWarning = (gpu?.hotspotTempC ?? 0) >= 95
  const storageTempWarning = (storage?.nvmeTempC ?? 0) >= 70

  return (
    <div className="overview-view">
      <div className="overview-grid">
        <div className="overview-item">
          <div className="overview-item-header">
            <span className="overview-label">CPU</span>
          </div>
          <div className="overview-item-values">
            <span
              className={`overview-value ${cpuTempWarning ? "warning" : ""}`}
            >
              {formatTemp(cpu?.packageTempC)}
            </span>
            <span className="overview-value secondary">
              {formatPower(cpu?.packagePowerW)}
            </span>
          </div>
        </div>

        <div className="overview-item">
          <div className="overview-item-header">
            <span className="overview-label">GPU</span>
          </div>
          <div className="overview-item-values">
            <span
              className={`overview-value ${gpuTempWarning ? "warning" : ""}`}
            >
              {formatTemp(gpu?.hotspotTempC)}
            </span>
            <span className="overview-value secondary">
              {formatPower(gpu?.powerW)}
            </span>
          </div>
        </div>

        <div className="overview-item">
          <div className="overview-item-header">
            <span className="overview-label">NVMe</span>
          </div>
          <div className="overview-item-values">
            <span
              className={`overview-value ${storageTempWarning ? "warning" : ""}`}
            >
              {formatTemp(storage?.nvmeTempC)}
            </span>
            <span
              className={`overview-value secondary status-${storage?.smartHealth ?? "unknown"}`}
            >
              {storage?.smartHealth === "good"
                ? "✓"
                : storage?.smartHealth === "warning"
                  ? "⚠"
                  : "—"}
            </span>
          </div>
        </div>

        <div className="overview-item">
          <div className="overview-item-header">
            <span className="overview-label">System</span>
          </div>
          <div className="overview-item-values">
            <span className="overview-value">
              {formatUptime(system?.uptimeSeconds ?? null)}
            </span>
            <span
              className={`overview-value secondary status-${system?.fanStatus ?? "unknown"}`}
            >
              {system?.fanStatus === "ok"
                ? "✓"
                : system?.fanStatus === "warning"
                  ? "⚠"
                  : "—"}
            </span>
          </div>
        </div>
      </div>
    </div>
  )
}
