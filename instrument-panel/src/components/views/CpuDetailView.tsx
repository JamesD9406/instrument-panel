import { SensorData } from "../../types/sensors"

interface Props {
  data: SensorData | null
}

function formatValue(value: number | null | undefined, unit: string): string {
  if (value === null || value === undefined) return "—"
  return `${value.toFixed(1)}${unit}`
}

export function CpuDetailView({ data }: Props) {
  const cpu = data?.cpu
  const tempWarning = (cpu?.packageTempC ?? 0) >= 85

  return (
    <div className="detail-view">
      <div className="detail-header">
        <span className="detail-title">CPU</span>
        {cpu?.name && <span className="detail-subtitle">{cpu.name}</span>}
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
      </div>
    </div>
  )
}
