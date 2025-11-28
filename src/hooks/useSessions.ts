//! React hook for session management

import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { Session, WorkloadProfile, HardwareConfig, Run } from "../types/index";

export function useSessions() {
  const [sessions, setSessions] = useState<Session[]>([]);
  const [currentSession, setCurrentSession] = useState<Session | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadSessions = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const sessionIds = await invoke<string[]>("list_sessions");
      
      // Load each session
      const loadedSessions = await Promise.all(
        sessionIds.map(async (id) => {
          return await invoke<Session>("load_session", { sessionId: id });
        })
      );
      
      setSessions(loadedSessions);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load sessions");
    } finally {
      setLoading(false);
    }
  }, []);

  const createSession = useCallback(
    async (
      name: string,
      profile: WorkloadProfile,
      hardwareConfig: HardwareConfig
    ) => {
      try {
        setError(null);
        const session = await invoke<Session>("create_session", {
          name,
          profile,
          hardwareConfig,
        });
        setCurrentSession(session);
        await loadSessions();
        return session;
      } catch (err) {
        setError(err instanceof Error ? err.message : "Failed to create session");
        throw err;
      }
    },
    [loadSessions]
  );

  const endSession = useCallback(
    async (sessionId: string) => {
      try {
        setError(null);
        const session = await invoke<Session>("end_session", { sessionId });
        if (currentSession?.id === sessionId) {
          setCurrentSession(null);
        }
        await loadSessions();
        return session;
      } catch (err) {
        setError(err instanceof Error ? err.message : "Failed to end session");
        throw err;
      }
    },
    [currentSession, loadSessions]
  );

  const addRun = useCallback(
    async (sessionId: string, run: Run) => {
      try {
        setError(null);
        const session = await invoke<Session>("add_run_to_session", {
          sessionId,
          run,
        });
        if (currentSession?.id === sessionId) {
          setCurrentSession(session);
        }
        await loadSessions();
        return session;
      } catch (err) {
        setError(err instanceof Error ? err.message : "Failed to add run");
        throw err;
      }
    },
    [currentSession, loadSessions]
  );

  useEffect(() => {
    loadSessions();
  }, [loadSessions]);

  return {
    sessions,
    currentSession,
    loading,
    error,
    createSession,
    endSession,
    addRun,
    loadSessions,
    setCurrentSession,
  };
}

