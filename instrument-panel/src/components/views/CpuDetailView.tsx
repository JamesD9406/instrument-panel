import { SensorData } from "../../types/sensors"

interface Props {
  data: SensorData | null
}

function formatValue(
  value: number | null | undefined,
  unit: string,
  decimals = 1,
): string {
  if (value === null || value === undefined) return "—"
  return `${value.toFixed(decimals)}${unit}`
}

function formatMhz(value: number | null | undefined): string {
  if (value === null || value === undefined) return "—"
  if (value >= 1000) return `${(value / 1000).toFixed(2)} GHz`
  return `${Math.round(value)} MHz`
}

function cleanCpuName(name: string | null | undefined): string {
  if (!name) return ""
  // Remove "CPU [#0]: " prefix if present
  return name.replace(/^CPU\s*\[#\d+\]:\s*/i, "").trim()
}

export function CpuDetailView({ data }: Props) {
  const cpu = data?.cpu
  const tempWarning = (cpu?.packageTempC ?? 0) >= 85
  const cpuName = cleanCpuName(cpu?.name)

  return (
    <div className="detail-view">
      <div className="detail-header">
        <span className="detail-title">CPU</span>
        {cpuName && <span className="detail-subtitle">{cpuName}</span>}
      </div>

      <div className="detail-rows">
        <div className={`detail-row ${tempWarning ? "warning" : ""}`}>
          <span className="detail-label">Package Temp</span>
          <span className="detail-value">
            {formatValue(cpu?.packageTempC, "°C")}
          </span>
        </div>
        <div className="detail-row">
          <span className="detail-label">Package Power</span>
          <span className="detail-value">
            {formatValue(cpu?.packagePowerW, "W")}
          </span>
        </div>
        {cpu?.coreClockMhz && (
          <div className="detail-row">
            <span className="detail-label">Core Clock</span>
            <span className="detail-value">{formatMhz(cpu.coreClockMhz)}</span>
          </div>
        )}
        {cpu?.usagePercent !== null && cpu?.usagePercent !== undefined && (
          <div className="detail-row">
            <span className="detail-label">CPU Usage</span>
            <span className="detail-value">
              {formatValue(cpu.usagePercent, "%", 0)}
            </span>
          </div>
        )}
        {cpu?.coreTemps && cpu.coreTemps.length > 0 && (
          <div className="detail-row">
            <span className="detail-label">Core Temps</span>
            <span className="detail-value detail-value-small">
              {cpu.coreTemps.map((t) => `${Math.round(t)}°`).join(" ")}
            </span>
          </div>
        )}
      </div>
    </div>
  )
}
