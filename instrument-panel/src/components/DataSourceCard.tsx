import { useState } from "react"
import { invoke } from "@tauri-apps/api/core"
import { SensorData, AppSettings } from "../types/sensors"

interface Props {
  data: SensorData | null
  settings: AppSettings
  onSettingsChange: (settings: Partial<AppSettings>) => void
  onRetry: () => void
  onShowSetupGuide: () => void
}

export function DataSourceCard({
  data,
  settings,
  onSettingsChange,
  onRetry,
  onShowSetupGuide,
}: Props) {
  const [showDiagnostics, setShowDiagnostics] = useState(false)
  const [launching, setLaunching] = useState(false)

  const isConnected = data?.status === "connected"

  const handleDumpSensors = async () => {
    try {
      const result = await invoke("debug_dump_sensors") as { header: unknown; sensors: unknown[] }
      console.log("=== HWiNFO Debug Dump ===")
      console.log("Header:", result.header)
      console.log("Sensors:")
      console.table(result.sensors)
    } catch (e) {
      console.error("Failed to dump sensors:", e)
    }
  }

  const handleLaunchHwinfo = async () => {
    setLaunching(true)
    try {
      await invoke("launch_hwinfo", { customPath: settings.hwinfoPath })
      // Wait a moment for HWiNFO to start
      setTimeout(onRetry, 2000)
    } catch (e) {
      console.error("Failed to launch HWiNFO:", e)
    } finally {
      setLaunching(false)
    }
  }

  const getTimeSinceUpdate = (): string => {
    if (!data?.lastReadAt) return "—"
    const diff = Date.now() - new Date(data.lastReadAt).getTime()
    if (diff < 1000) return "just now"
    return `${(diff / 1000).toFixed(1)}s ago`
  }

  return (
    <div
      className={`card data-source-card ${isConnected ? "connected" : "disconnected"}`}
    >
      <div className="card-header">
        <h2>Data Source</h2>
        <span
          className={`status-badge ${isConnected ? "connected" : "disconnected"}`}
        >
          {isConnected ? "✓ Connected" : "✗ Not Connected"}
        </span>
      </div>

      {isConnected ? (
        <div className="card-body">
          <div className="info-row">
            <span className="label">Last update:</span>
            <span className="value">{getTimeSinceUpdate()}</span>
          </div>

          <label className="toggle-row">
            <input
              type="checkbox"
              checked={settings.mockMode}
              onChange={(e) => onSettingsChange({ mockMode: e.target.checked })}
            />
            <span>Mock mode (fake data for testing)</span>
          </label>

          <button
            className="link-button"
            onClick={() => setShowDiagnostics(!showDiagnostics)}
          >
            {showDiagnostics ? "Hide" : "Show"} diagnostics
          </button>

          {showDiagnostics && (
            <div className="diagnostics">
              <div className="info-row">
                <span>HWiNFO process:</span>
                <span>
                  {data.diagnostics.hwinfoProcessDetected
                    ? "✓ Detected"
                    : "✗ Not detected"}
                </span>
              </div>
              <div className="info-row">
                <span>Shared memory:</span>
                <span>
                  {data.diagnostics.sharedMemoryDetected
                    ? "✓ Detected"
                    : "✗ Not detected"}
                </span>
              </div>
              {data.diagnostics.message && (
                <div className="info-row">
                  <span>Message:</span>
                  <span>{data.diagnostics.message}</span>
                </div>
              )}
              <button onClick={handleDumpSensors} style={{ marginTop: "8px" }}>
                Dump Sensors (Console)
              </button>
            </div>
          )}
        </div>
      ) : (
        <div className="card-body not-connected">
          <p className="message">Sensor source not connected</p>

          <div className="diagnostics">
            <div className="info-row">
              <span>HWiNFO process:</span>
              <span>
                {data?.diagnostics.hwinfoProcessDetected
                  ? "✓ Detected"
                  : "✗ Not detected"}
              </span>
            </div>
            <div className="info-row">
              <span>Shared memory:</span>
              <span>
                {data?.diagnostics.sharedMemoryDetected
                  ? "✓ Detected"
                  : "✗ Not detected"}
              </span>
            </div>
            {data?.diagnostics.message && (
              <p className="error-message">{data.diagnostics.message}</p>
            )}
          </div>

          <div className="button-group">
            <button onClick={handleLaunchHwinfo} disabled={launching}>
              {launching ? "Launching..." : "Launch HWiNFO"}
            </button>
            <button onClick={onRetry}>Retry</button>
            <button onClick={onShowSetupGuide}>Setup Guide</button>
          </div>

          <label className="toggle-row">
            <input
              type="checkbox"
              checked={settings.autoLaunchHwinfo}
              onChange={(e) =>
                onSettingsChange({ autoLaunchHwinfo: e.target.checked })
              }
            />
            <span>Auto-launch HWiNFO when needed</span>
          </label>

          <label className="toggle-row">
            <input
              type="checkbox"
              checked={settings.mockMode}
              onChange={(e) => onSettingsChange({ mockMode: e.target.checked })}
            />
            <span>Mock mode (fake data for testing)</span>
          </label>
        </div>
      )}
    </div>
  )
}
