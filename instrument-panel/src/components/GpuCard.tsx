import { MetricCard } from "./MetricCard"
import { SensorData } from "../types/sensors"

interface Props {
  data: SensorData | null
}

export function GpuCard({ data }: Props) {
  const gpu = data?.gpu

  return (
    <MetricCard title="GPU" subtitle={gpu?.name} icon="🎮">
      <MetricCard.Row
        label="Hotspot Temp"
        value={gpu?.hotspotTempC ?? null}
        unit="°C"
        warningThreshold={95}
      />
      <MetricCard.Row
        label="Memory Junction"
        value={gpu?.memoryJunctionTempC ?? null}
        unit="°C"
        warningThreshold={100}
      />
      <MetricCard.Row label="Power Draw" value={gpu?.powerW ?? null} unit="W" />
    </MetricCard>
  )
}
