//! React hook for metrics collection

import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { MetricSample } from "../types/index";

export function useMetrics() {
  const [metrics, setMetrics] = useState<MetricSample[]>([]);
  const [isCollecting, setIsCollecting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const startCollection = useCallback(async (samplingIntervalMs?: number) => {
    try {
      setError(null);
      await invoke("start_metrics_collection", { samplingIntervalMs });
      setIsCollecting(true);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to start metrics collection");
      setIsCollecting(false);
    }
  }, []);

  const stopCollection = useCallback(async () => {
    try {
      setError(null);
      await invoke("stop_metrics_collection");
      setIsCollecting(false);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to stop metrics collection");
    }
  }, []);

  const loadMetrics = useCallback(async () => {
    try {
      setError(null);
      const buffer = await invoke<MetricSample[]>("get_metrics_buffer");
      setMetrics(buffer);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load metrics");
    }
  }, []);

  useEffect(() => {
    if (isCollecting) {
      // Poll metrics buffer periodically (Tauri events can be added later)
      const interval = setInterval(loadMetrics, 1000);

      return () => {
        clearInterval(interval);
      };
    }
  }, [isCollecting, loadMetrics]);

  return {
    metrics,
    isCollecting,
    error,
    startCollection,
    stopCollection,
    loadMetrics,
  };
}
