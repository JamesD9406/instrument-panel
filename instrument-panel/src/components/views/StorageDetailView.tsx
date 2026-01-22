import { SensorData } from "../../types/sensors"

interface Props {
  data: SensorData | null
}

function formatValue(value: number | null | undefined, unit: string): string {
  if (value === null || value === undefined) return "—"
  return `${value.toFixed(1)}${unit}`
}

export function StorageDetailView({ data }: Props) {
  const storage = data?.storage
  const tempWarning = (storage?.nvmeTempC ?? 0) >= 70

  const healthText = {
    good: "✓ Good",
    warning: "⚠ Warning",
    unknown: "— Unknown",
  }

  return (
    <div className="detail-view">
      <div className="detail-header">
        <span className="detail-title">Storage</span>
        {storage?.name && (
          <span className="detail-subtitle">{storage.name}</span>
        )}
      </div>

      <div className="detail-rows">
        <div className={`detail-row ${tempWarning ? "warning" : ""}`}>
          <span className="detail-label">Drive Temp</span>
          <span className="detail-value">
            {formatValue(storage?.nvmeTempC, "°C")}
          </span>
        </div>
        <div
          className={`detail-row status-${storage?.smartHealth ?? "unknown"}`}
        >
          <span className="detail-label">SMART Health</span>
          <span className="detail-value">
            {healthText[storage?.smartHealth ?? "unknown"]}
          </span>
        </div>
      </div>
    </div>
  )
}
