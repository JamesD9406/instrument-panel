import { useState, useEffect, useCallback } from "react"
import { invoke } from "@tauri-apps/api/core"
import { AppSettings } from "../types/sensors"

const DEFAULT_SETTINGS: AppSettings = {
  autoLaunchHwinfo: false,
  hwinfoPath: null,
  mockMode: false,
}

export function useSettings() {
  const [settings, setSettings] = useState<AppSettings>(DEFAULT_SETTINGS)
  const [isLoading, setIsLoading] = useState(true)

  useEffect(() => {
    invoke<AppSettings>("get_settings")
      .then(setSettings)
      .catch(console.error)
      .finally(() => setIsLoading(false))
  }, [])

  const updateSettings = useCallback(
    async (newSettings: Partial<AppSettings>) => {
      const updated = { ...settings, ...newSettings }
      setSettings(updated)
      try {
        await invoke("save_settings", { settings: updated })
      } catch (e) {
        console.error("Failed to save settings:", e)
      }
    },
    [settings],
  )

  return { settings, updateSettings, isLoading }
}
