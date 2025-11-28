//! React hook for hardware detection

import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { HardwareConfig } from "../types/index";

export function useHardware() {
  const [hardware, setHardware] = useState<HardwareConfig | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const loadHardware = async () => {
    try {
      setLoading(true);
      setError(null);
      const config = await invoke<HardwareConfig>("get_hardware_config");
      setHardware(config);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load hardware");
    } finally {
      setLoading(false);
    }
  };

  const refreshHardware = async () => {
    try {
      setLoading(true);
      setError(null);
      const config = await invoke<HardwareConfig>("refresh_hardware_config");
      setHardware(config);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to refresh hardware");
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadHardware();
  }, []);

  return { hardware, loading, error, refreshHardware };
}

