//! Theme management hook
//!
//! Provides theme switching functionality and applies theme preferences
//! following IMPLEMENTATION_PLAN.md Phase 3.5.

import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { UserSettings } from "../types/index";

export function useTheme() {
  const [theme, setTheme] = useState<"light" | "dark" | "system">("system");
  const [highContrast, setHighContrast] = useState(false);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadTheme();
  }, []);

  useEffect(() => {
    applyTheme(theme, highContrast);
  }, [theme, highContrast]);

  const loadTheme = async () => {
    try {
      const settings = await invoke<UserSettings>("get_settings");
      setTheme(settings.theme.mode);
      setHighContrast(settings.theme.high_contrast);
    } catch (err) {
      console.error("Failed to load theme settings:", err);
    } finally {
      setLoading(false);
    }
  };

  const applyTheme = (mode: "light" | "dark" | "system", contrast: boolean) => {
    const root = document.documentElement;
    
    // Determine actual theme (system mode uses OS preference)
    let actualTheme = mode;
    if (mode === "system") {
      const prefersDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
      actualTheme = prefersDark ? "dark" : "light";
    }

    // Remove existing theme classes
    root.classList.remove("theme-light", "theme-dark", "high-contrast");
    
    // Apply new theme
    root.classList.add(`theme-${actualTheme}`);
    if (contrast) {
      root.classList.add("high-contrast");
    }
  };

  const updateTheme = async (mode: "light" | "dark" | "system", contrast: boolean) => {
    try {
      await invoke("update_theme", {
        theme: {
          mode,
          high_contrast: contrast,
        },
      });
      setTheme(mode);
      setHighContrast(contrast);
    } catch (err) {
      console.error("Failed to update theme:", err);
    }
  };

  return {
    theme,
    highContrast,
    loading,
    updateTheme,
  };
}

