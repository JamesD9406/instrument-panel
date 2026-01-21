import { MetricCard } from "./MetricCard"
import { SensorData } from "../types/sensors"

interface Props {
  data: SensorData | null
}

export function CpuCard({ data }: Props) {
  const cpu = data?.cpu

  return (
    <MetricCard title="CPU" subtitle={cpu?.name} icon="⚡">
      <MetricCard.Row
        label="Package Temp"
        value={cpu?.packageTempC ?? null}
        unit="°C"
        warningThreshold={85}
      />
      <MetricCard.Row
        label="Package Power"
        value={cpu?.packagePowerW ?? null}
        unit="W"
      />
    </MetricCard>
  )
}
