import { useState } from "react"
import { useSensorData } from "./hooks/useSensorData"
import { useSettings } from "./hooks/useSettings"
import { DataSourceCard } from "./components/DataSourceCard"
import { ViewSelector } from "./components/ViewSelector"
import { OverviewView } from "./components/views/OverviewView"
import { CpuDetailView } from "./components/views/CpuDetailView"
import { GpuDetailView } from "./components/views/GpuDetailView"
import { StorageDetailView } from "./components/views/StorageDetailView"
import { SetupGuide } from "./components/SetupGuide"
import "./App.css"

const VIEWS = [
  { id: "overview", label: "Overview" },
  { id: "cpu", label: "CPU Details" },
  { id: "gpu", label: "GPU Details" },
  { id: "storage", label: "Storage" },
]

function App() {
  const { data, isLoading, refresh } = useSensorData(1000)
  const { settings, updateSettings } = useSettings()
  const [showSetupGuide, setShowSetupGuide] = useState(false)
  const [activeView, setActiveView] = useState("overview")
  const [showDataSource, setShowDataSource] = useState(false)

  if (isLoading) {
    return (
      <div className="app loading">
        <div className="loading-spinner">Loading...</div>
      </div>
    )
  }

  const isConnected = data?.status === "connected"

  const renderView = () => {
    switch (activeView) {
      case "overview":
        return <OverviewView data={data} />
      case "cpu":
        return <CpuDetailView data={data} />
      case "gpu":
        return <GpuDetailView data={data} />
      case "storage":
        return <StorageDetailView data={data} />
      default:
        return <OverviewView data={data} />
    }
  }

  return (
    <div className="app">
      <header className="app-header">
        <div className="app-header-left">
          <h1>Instrument Panel</h1>
          <button
            className={`status-indicator ${isConnected ? "connected" : "disconnected"}`}
            onClick={() => setShowDataSource(!showDataSource)}
            title={isConnected ? "Connected to HWiNFO" : "Not connected"}
          >
            {isConnected ? "●" : "○"}
          </button>
        </div>
        <ViewSelector
          views={VIEWS}
          activeView={activeView}
          onViewChange={setActiveView}
        />
      </header>

      {showDataSource && (
        <DataSourceCard
          data={data}
          settings={settings}
          onSettingsChange={updateSettings}
          onRetry={refresh}
          onShowSetupGuide={() => setShowSetupGuide(true)}
        />
      )}

      <main className="app-main">{renderView()}</main>

      {showSetupGuide && (
        <SetupGuide onClose={() => setShowSetupGuide(false)} />
      )}
    </div>
  )
}

export default App
