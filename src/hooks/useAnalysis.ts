//! React hook for bottleneck analysis

import { useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type {
  BottleneckAnalysisResult,
  MetricSample,
  UserFacingInsights,
  WorkloadProfile,
} from "../types/index";

export function useAnalysis() {
  const [analysisResult, setAnalysisResult] =
    useState<BottleneckAnalysisResult | null>(null);
  const [insights, setInsights] = useState<UserFacingInsights | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const analyze = useCallback(
    async (
      metrics: MetricSample[],
      timeWindowSeconds?: number,
      profile?: WorkloadProfile
    ) => {
      try {
        setLoading(true);
        setError(null);
        const result = await invoke<BottleneckAnalysisResult>(
          "analyze_bottlenecks",
          {
            metrics,
            timeWindowSeconds: timeWindowSeconds ?? 30,
            profile: profile ?? null,
          }
        );
        setAnalysisResult(result);

        // Generate insights
        const generatedInsights = await invoke<UserFacingInsights>(
          "generate_insights",
          { result, profile: profile ?? null }
        );
        setInsights(generatedInsights);
      } catch (err) {
        setError(
          err instanceof Error ? err.message : "Failed to analyze bottlenecks"
        );
      } finally {
        setLoading(false);
      }
    },
    []
  );

  return {
    analysisResult,
    insights,
    loading,
    error,
    analyze,
  };
}

