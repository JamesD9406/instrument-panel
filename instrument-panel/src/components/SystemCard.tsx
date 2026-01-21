import { MetricCard } from "./MetricCard"
import { SensorData } from "../types/sensors"

interface Props {
  data: SensorData | null
}

function formatUptime(seconds: number | null): string {
  if (seconds === null) return "—"

  const days = Math.floor(seconds / 86400)
  const hours = Math.floor((seconds % 86400) / 3600)
  const mins = Math.floor((seconds % 3600) / 60)

  if (days > 0) {
    return `${days}d ${hours}h ${mins}m`
  } else if (hours > 0) {
    return `${hours}h ${mins}m`
  } else {
    return `${mins}m`
  }
}

export function SystemCard({ data }: Props) {
  const system = data?.system

  return (
    <MetricCard title="System" subtitle={system?.name} icon="🖥️">
      <div className="metric-row">
        <span className="label">Uptime</span>
        <span className="value">
          {formatUptime(system?.uptimeSeconds ?? null)}
        </span>
      </div>
      <MetricCard.Status
        label="Fan Status"
        status={system?.fanStatus ?? "unknown"}
      />
    </MetricCard>
  )
}
