import { useState, useEffect, useCallback } from "react"
import { invoke } from "@tauri-apps/api/core"
import { SensorData } from "../types/sensors"

interface UseSensorDataResult {
  data: SensorData | null
  isLoading: boolean
  error: string | null
  refresh: () => Promise<void>
  lastRefreshTime: Date | null
}

export function useSensorData(pollingInterval = 1000): UseSensorDataResult {
  const [data, setData] = useState<SensorData | null>(null)
  const [isLoading, setIsLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [lastRefreshTime, setLastRefreshTime] = useState<Date | null>(null)

  const refresh = useCallback(async () => {
    try {
      const result = await invoke<SensorData>("get_sensor_data")
      setData(result)
      setError(null)
      setLastRefreshTime(new Date())
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e))
    } finally {
      setIsLoading(false)
    }
  }, [])

  useEffect(() => {
    // Initial fetch
    refresh()

    // Set up polling
    const interval = setInterval(refresh, pollingInterval)

    return () => clearInterval(interval)
  }, [refresh, pollingInterval])

  return { data, isLoading, error, refresh, lastRefreshTime }
}
