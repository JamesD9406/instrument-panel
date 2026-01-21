import { useState } from "react"
import { useSensorData } from "./hooks/useSensorData"
import { useSettings } from "./hooks/useSettings"
import { DataSourceCard } from "./components/DataSourceCard"
import { CpuCard } from "./components/CpuCard"
import { GpuCard } from "./components/GpuCard"
import { StorageCard } from "./components/StorageCard"
import { SystemCard } from "./components/SystemCard"
import { SetupGuide } from "./components/SetupGuide"
import "./App.css"

function App() {
  const { data, isLoading, refresh } = useSensorData(1000)
  const { settings, updateSettings } = useSettings()
  const [showSetupGuide, setShowSetupGuide] = useState(false)

  if (isLoading) {
    return (
      <div className="app loading">
        <div className="loading-spinner">Loading...</div>
      </div>
    )
  }

  return (
    <div className="app">
      <header className="app-header">
        <h1>The Instrument Panel</h1>
      </header>

      <main className="app-main">
        <DataSourceCard
          data={data}
          settings={settings}
          onSettingsChange={updateSettings}
          onRetry={refresh}
          onShowSetupGuide={() => setShowSetupGuide(true)}
        />

        <div className="metrics-grid">
          <CpuCard data={data} />
          <GpuCard data={data} />
          <StorageCard data={data} />
          <SystemCard data={data} />
        </div>
      </main>

      {showSetupGuide && (
        <SetupGuide onClose={() => setShowSetupGuide(false)} />
      )}
    </div>
  )
}

export default App
