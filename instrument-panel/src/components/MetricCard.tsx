interface MetricRowProps {
  label: string
  value: number | null
  unit: string
  warningThreshold?: number
}

function MetricRow({ label, value, unit, warningThreshold }: MetricRowProps) {
  const isWarning = warningThreshold && value && value >= warningThreshold

  return (
    <div className={`metric-row ${isWarning ? "warning" : ""}`}>
      <span className="label">{label}</span>
      <span className="value">
        {value !== null ? `${value.toFixed(1)} ${unit}` : "—"}
      </span>
    </div>
  )
}

interface StatusRowProps {
  label: string
  status: "good" | "ok" | "warning" | "critical" | "unknown"
}

function StatusRow({ label, status }: StatusRowProps) {
  const statusText = {
    good: "✓ Good",
    ok: "✓ OK",
    warning: "⚠ Warning",
    critical: "✗ Critical",
    unknown: "— Unknown",
  }

  return (
    <div className={`metric-row status-${status}`}>
      <span className="label">{label}</span>
      <span className="value">{statusText[status]}</span>
    </div>
  )
}

interface Props {
  title: string
  subtitle?: string | null
  icon: string
  children: React.ReactNode
}

export function MetricCard({ title, subtitle, icon, children }: Props) {
  return (
    <div className="card metric-card">
      <div className="card-header">
        <div className="card-header-left">
          <span className="icon">{icon}</span>
          <h3>{title}</h3>
        </div>
        {subtitle && <span className="card-subtitle">{subtitle}</span>}
      </div>
      <div className="card-body">{children}</div>
    </div>
  )
}

MetricCard.Row = MetricRow
MetricCard.Status = StatusRow
