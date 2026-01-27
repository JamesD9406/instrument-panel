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

function formatVram(
  used: number | null | undefined,
  total: number | null | undefined,
): string {
  if (used === null || used === undefined) return "—"
  const usedGb = used / 1024
  if (total) {
    const totalGb = total / 1024
    return `${usedGb.toFixed(1)} / ${totalGb.toFixed(0)} GB`
  }
  return `${usedGb.toFixed(1)} GB`
}

function cleanGpuName(name: string | null | undefined): string {
  if (!name) return ""
  // Remove "GPU [#0]: " or similar prefixes
  return name.replace(/^GPU\s*\[#\d+\]:\s*/i, "").trim()
}

export function GpuDetailView({ data }: Props) {
  const gpu = data?.gpu
  const hotspotWarning = (gpu?.hotspotTempC ?? 0) >= 95
  const memJunctionWarning = (gpu?.memoryJunctionTempC ?? 0) >= 100
  const gpuName = cleanGpuName(gpu?.name)

  return (
    <div className="detail-view">
      <div className="detail-header">
        <span className="detail-title">GPU</span>
        {gpuName && <span className="detail-subtitle">{gpuName}</span>}
      </div>

      <div className="detail-rows">
        <div className={`detail-row ${hotspotWarning ? "warning" : ""}`}>
          <span className="detail-label">Hotspot Temp</span>
          <span className="detail-value">
            {formatValue(gpu?.hotspotTempC, "°C")}
          </span>
        </div>
        {gpu?.memoryJunctionTempC && (
          <div className={`detail-row ${memJunctionWarning ? "warning" : ""}`}>
            <span className="detail-label">Memory Junction</span>
            <span className="detail-value">
              {formatValue(gpu.memoryJunctionTempC, "°C")}
            </span>
          </div>
        )}
        <div className="detail-row">
          <span className="detail-label">Power Draw</span>
          <span className="detail-value">{formatValue(gpu?.powerW, "W")}</span>
        </div>
        {gpu?.coreClockMhz && (
          <div className="detail-row">
            <span className="detail-label">Core Clock</span>
            <span className="detail-value">{formatMhz(gpu.coreClockMhz)}</span>
          </div>
        )}
        {gpu?.memoryClockMhz && (
          <div className="detail-row">
            <span className="detail-label">Memory Clock</span>
            <span className="detail-value">
              {formatMhz(gpu.memoryClockMhz)}
            </span>
          </div>
        )}
        {gpu?.usagePercent !== null && gpu?.usagePercent !== undefined && (
          <div className="detail-row">
            <span className="detail-label">GPU Usage</span>
            <span className="detail-value">
              {formatValue(gpu.usagePercent, "%", 0)}
            </span>
          </div>
        )}
        {gpu?.vramUsedMb && (
          <div className="detail-row">
            <span className="detail-label">VRAM</span>
            <span className="detail-value">
              {formatVram(gpu.vramUsedMb, gpu.vramTotalMb)}
            </span>
          </div>
        )}
        {gpu?.fanSpeedRpm != null && gpu.fanSpeedRpm > 0 && (
          <div className="detail-row">
            <span className="detail-label">Fan Speed</span>
            <span className="detail-value">
              {Math.round(gpu.fanSpeedRpm)} RPM
              {gpu.fanSpeedPercent
                ? ` (${Math.round(gpu.fanSpeedPercent)}%)`
                : ""}
            </span>
          </div>
        )}
      </div>
    </div>
  )
}
