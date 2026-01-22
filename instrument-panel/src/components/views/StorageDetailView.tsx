import { useState } from "react"
import { SensorData, DriveData } from "../../types/sensors"

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

function formatDriveName(drive: DriveData): string {
  const letter = drive.driveLetter || ""
  // Extract just the drive model from the full S.M.A.R.T. name
  const name = drive.name || ""
  const cleanName = name
    .replace(/^S\.M\.A\.R\.T\.:\s*/i, "")
    .replace(/\s*\[[A-Z]:\].*$/, "")
    .trim()
  return letter ? `${letter} ${cleanName}` : cleanName
}

function formatSpace(
  total: number | null | undefined,
  free: number | null | undefined,
): string {
  if (!total) return "—"
  const used = total - (free ?? 0)
  return `${used.toFixed(0)} / ${total.toFixed(0)} GB`
}

export function StorageDetailView({ data }: Props) {
  const drives = data?.drives ?? []
  const [selectedIndex, setSelectedIndex] = useState(0)

  const selectedDrive = drives[selectedIndex]
  const tempWarning = (selectedDrive?.tempC ?? 0) >= 70

  const healthText: Record<string, string> = {
    good: "✓ Good",
    warning: "⚠ Warning",
    critical: "✗ Critical",
    unknown: "— Unknown",
  }

  if (drives.length === 0) {
    return (
      <div className="detail-view">
        <div className="detail-header">
          <span className="detail-title">Storage</span>
        </div>
        <div className="detail-rows">
          <div className="detail-row">
            <span className="detail-label">No drives detected</span>
          </div>
        </div>
      </div>
    )
  }

  return (
    <div className="detail-view">
      <div className="detail-header">
        <span className="detail-title">Storage</span>
        {drives.length > 1 && (
          <select
            className="drive-selector"
            value={selectedIndex}
            onChange={(e) => setSelectedIndex(Number(e.target.value))}
          >
            {drives.map((drive, idx) => (
              <option key={idx} value={idx}>
                {drive.driveLetter || `Drive ${idx + 1}`}
              </option>
            ))}
          </select>
        )}
      </div>

      {selectedDrive && (
        <>
          <div className="detail-subheader">
            {formatDriveName(selectedDrive)}
          </div>
          <div className="detail-rows">
            <div className={`detail-row ${tempWarning ? "warning" : ""}`}>
              <span className="detail-label">Temperature</span>
              <span className="detail-value">
                {formatValue(selectedDrive.tempC, "°C")}
              </span>
            </div>
            <div className={`detail-row status-${selectedDrive.smartHealth}`}>
              <span className="detail-label">SMART Health</span>
              <span className="detail-value">
                {healthText[selectedDrive.smartHealth]}
              </span>
            </div>
            {selectedDrive.totalGb && (
              <div className="detail-row">
                <span className="detail-label">Space Used</span>
                <span className="detail-value">
                  {formatSpace(selectedDrive.totalGb, selectedDrive.freeGb)}
                </span>
              </div>
            )}
            {selectedDrive.freeGb && (
              <div className="detail-row">
                <span className="detail-label">Free Space</span>
                <span className="detail-value">
                  {formatValue(selectedDrive.freeGb, " GB", 0)}
                </span>
              </div>
            )}
          </div>
        </>
      )}
    </div>
  )
}
