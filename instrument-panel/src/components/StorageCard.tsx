import { MetricCard } from "./MetricCard"
import { SensorData } from "../types/sensors"

interface Props {
  data: SensorData | null
}

export function StorageCard({ data }: Props) {
  const storage = data?.storage

  return (
    <MetricCard title="Storage" subtitle={storage?.name} icon="💾">
      <MetricCard.Row
        label="NVMe Temp"
        value={storage?.nvmeTempC ?? null}
        unit="°C"
        warningThreshold={70}
      />
      <MetricCard.Status
        label="SMART Health"
        status={storage?.smartHealth ?? "unknown"}
      />
    </MetricCard>
  )
}
