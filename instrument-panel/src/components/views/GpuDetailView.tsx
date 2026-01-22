import { SensorData } from "../../types/sensors"

interface Props {
  data: SensorData | null
}

function formatValue(value: number | null | undefined, unit: string): string {
  if (value === null || value === undefined) return "—"
  return `${value.toFixed(1)}${unit}`
}

export function GpuDetailView({ data }: Props) {
  const gpu = data?.gpu
  const hotspotWarning = (gpu?.hotspotTempC ?? 0) >= 95
  const memJunctionWarning = (gpu?.memoryJunctionTempC ?? 0) >= 100

  return (
    <div className="detail-view">
      <div className="detail-header">
        <span className="detail-title">GPU</span>
        {gpu?.name && <span className="detail-subtitle">{gpu.name}</span>}
      </div>

      <div className="detail-rows">
        <div className={`detail-row ${hotspotWarning ? "warning" : ""}`}>
          <span className="detail-label">Hotspot Temp</span>
          <span className="detail-value">
            {formatValue(gpu?.hotspotTempC, "°C")}
          </span>
        </div>
        <div className={`detail-row ${memJunctionWarning ? "warning" : ""}`}>
          <span className="detail-label">Memory Junction</span>
          <span className="detail-value">
            {formatValue(gpu?.memoryJunctionTempC, "°C")}
          </span>
        </div>
        <div className="detail-row">
          <span className="detail-label">Power Draw</span>
          <span className="detail-value">{formatValue(gpu?.powerW, "W")}</span>
        </div>
      </div>
    </div>
  )
}
