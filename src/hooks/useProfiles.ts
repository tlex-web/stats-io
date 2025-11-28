//! React hook for workload profile management

import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { WorkloadProfile } from "../types/index";

export function useProfiles() {
  const [profiles, setProfiles] = useState<WorkloadProfile[]>([]);
  const [selectedProfile, setSelectedProfile] = useState<WorkloadProfile | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const loadProfiles = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const presetProfiles = await invoke<WorkloadProfile[]>("get_preset_profiles");
      setProfiles(presetProfiles);
      
      // Set default profile if none selected
      if (!selectedProfile && presetProfiles.length > 0) {
        setSelectedProfile(presetProfiles[0]);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load profiles");
    } finally {
      setLoading(false);
    }
  }, [selectedProfile]);

  const loadProfileById = useCallback(async (id: string) => {
    try {
      setError(null);
      const profile = await invoke<WorkloadProfile>("get_profile_by_id", { id });
      setSelectedProfile(profile);
      return profile;
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load profile");
      throw err;
    }
  }, []);

  useEffect(() => {
    loadProfiles();
  }, [loadProfiles]);

  return {
    profiles,
    selectedProfile,
    loading,
    error,
    setSelectedProfile,
    loadProfileById,
    loadProfiles,
  };
}

