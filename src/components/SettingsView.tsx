//! Settings View component
//!
//! Allows users to customize application settings including thresholds,
//! sampling rates, units, and theme preferences.

import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useKeyboardNavigation } from "../hooks/useKeyboardNavigation";
import type { UserSettings } from "../types/index";

export function SettingsView() {
  const [settings, setSettings] = useState<UserSettings | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  const containerRef = useKeyboardNavigation();

  useEffect(() => {
    loadSettings();
  }, []);

  const loadSettings = async () => {
    try {
      setLoading(true);
      setError(null);
      const loadedSettings = await invoke<UserSettings>("get_settings");
      setSettings(loadedSettings);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load settings");
    } finally {
      setLoading(false);
    }
  };

  const saveSettings = async (updatedSettings: UserSettings) => {
    try {
      setSaving(true);
      setError(null);
      setSuccess(null);
      await invoke("update_settings", { settings: updatedSettings });
      setSettings(updatedSettings);
      setSuccess("Settings saved successfully!");
      setTimeout(() => setSuccess(null), 3000);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to save settings");
    } finally {
      setSaving(false);
    }
  };

  const resetToDefaults = async () => {
    if (!confirm("Are you sure you want to reset all settings to defaults?")) {
      return;
    }
    try {
      setSaving(true);
      setError(null);
      await invoke("reset_settings");
      await loadSettings();
      setSuccess("Settings reset to defaults!");
      setTimeout(() => setSuccess(null), 3000);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to reset settings");
    } finally {
      setSaving(false);
    }
  };

  if (loading) {
    return (
      <div className="settings-view">
        <div className="loading">Loading settings...</div>
      </div>
    );
  }

  if (!settings) {
    return (
      <div className="settings-view">
        <div className="error">Failed to load settings</div>
      </div>
    );
  }

  return (
    <div className="settings-view" ref={containerRef}>
      <h1>Settings</h1>

      {error && <div className="error-message">Error: {error}</div>}
      {success && <div className="success-message">{success}</div>}

      {/* Threshold Settings */}
      <section className="settings-section">
        <h2>Threshold Settings</h2>
        <p className="section-description">
          Configure thresholds for bottleneck detection. Values are percentages (0-100) for
          utilization metrics, or degrees Celsius for temperature.
        </p>
        <div className="settings-grid">
          <div className="setting-item">
            <label htmlFor="cpu-high">CPU High Threshold (%)</label>
            <input
              id="cpu-high"
              type="number"
              min="0"
              max="100"
              step="0.1"
              value={settings.thresholds.cpu_high}
              onChange={(e) => {
                const newSettings = {
                  ...settings,
                  thresholds: {
                    ...settings.thresholds,
                    cpu_high: parseFloat(e.target.value) || 0,
                  },
                };
                setSettings(newSettings);
              }}
            />
          </div>

          <div className="setting-item">
            <label htmlFor="gpu-high">GPU High Threshold (%)</label>
            <input
              id="gpu-high"
              type="number"
              min="0"
              max="100"
              step="0.1"
              value={settings.thresholds.gpu_high}
              onChange={(e) => {
                const newSettings = {
                  ...settings,
                  thresholds: {
                    ...settings.thresholds,
                    gpu_high: parseFloat(e.target.value) || 0,
                  },
                };
                setSettings(newSettings);
              }}
            />
          </div>

          <div className="setting-item">
            <label htmlFor="ram-high">RAM High Threshold (%)</label>
            <input
              id="ram-high"
              type="number"
              min="0"
              max="100"
              step="0.1"
              value={settings.thresholds.ram_high}
              onChange={(e) => {
                const newSettings = {
                  ...settings,
                  thresholds: {
                    ...settings.thresholds,
                    ram_high: parseFloat(e.target.value) || 0,
                  },
                };
                setSettings(newSettings);
              }}
            />
          </div>

          <div className="setting-item">
            <label htmlFor="vram-high">VRAM High Threshold (%)</label>
            <input
              id="vram-high"
              type="number"
              min="0"
              max="100"
              step="0.1"
              value={settings.thresholds.vram_high}
              onChange={(e) => {
                const newSettings = {
                  ...settings,
                  thresholds: {
                    ...settings.thresholds,
                    vram_high: parseFloat(e.target.value) || 0,
                  },
                };
                setSettings(newSettings);
              }}
            />
          </div>

          <div className="setting-item">
            <label htmlFor="temp-warning">Temperature Warning (°C)</label>
            <input
              id="temp-warning"
              type="number"
              min="0"
              max="150"
              step="1"
              value={settings.thresholds.temperature_warning}
              onChange={(e) => {
                const newSettings = {
                  ...settings,
                  thresholds: {
                    ...settings.thresholds,
                    temperature_warning: parseFloat(e.target.value) || 0,
                  },
                };
                setSettings(newSettings);
              }}
            />
          </div>

          <div className="setting-item">
            <label htmlFor="temp-critical">Temperature Critical (°C)</label>
            <input
              id="temp-critical"
              type="number"
              min="0"
              max="150"
              step="1"
              value={settings.thresholds.temperature_critical}
              onChange={(e) => {
                const newSettings = {
                  ...settings,
                  thresholds: {
                    ...settings.thresholds,
                    temperature_critical: parseFloat(e.target.value) || 0,
                  },
                };
                setSettings(newSettings);
              }}
            />
          </div>
        </div>
      </section>

      {/* Sampling Settings */}
      <section className="settings-section">
        <h2>Sampling Settings</h2>
        <p className="section-description">
          Configure how often metrics are collected and how many samples to keep in memory.
        </p>
        <div className="settings-grid">
          <div className="setting-item">
            <label htmlFor="interval-ms">Sampling Interval (ms)</label>
            <input
              id="interval-ms"
              type="number"
              min="100"
              max="10000"
              step="100"
              value={settings.sampling.interval_ms}
              onChange={(e) => {
                const newSettings = {
                  ...settings,
                  sampling: {
                    ...settings.sampling,
                    interval_ms: parseInt(e.target.value) || 1000,
                  },
                };
                setSettings(newSettings);
              }}
            />
            <small>Recommended: 1000ms (1 second)</small>
          </div>

          <div className="setting-item">
            <label htmlFor="buffer-size">Buffer Size (samples)</label>
            <input
              id="buffer-size"
              type="number"
              min="100"
              max="100000"
              step="100"
              value={settings.sampling.buffer_size}
              onChange={(e) => {
                const newSettings = {
                  ...settings,
                  sampling: {
                    ...settings.sampling,
                    buffer_size: parseInt(e.target.value) || 3600,
                  },
                };
                setSettings(newSettings);
              }}
            />
            <small>Number of samples to keep in memory</small>
          </div>
        </div>
      </section>

      {/* Unit Preferences */}
      <section className="settings-section">
        <h2>Unit Preferences</h2>
        <div className="settings-grid">
          <div className="setting-item">
            <label htmlFor="temp-unit">Temperature Unit</label>
            <select
              id="temp-unit"
              value={settings.units.temperature}
              onChange={(e) => {
                const newSettings = {
                  ...settings,
                  units: {
                    ...settings.units,
                    temperature: e.target.value as "celsius" | "fahrenheit",
                  },
                };
                setSettings(newSettings);
              }}
            >
              <option value="celsius">Celsius (°C)</option>
              <option value="fahrenheit">Fahrenheit (°F)</option>
            </select>
          </div>

          <div className="setting-item">
            <label htmlFor="memory-unit">Memory Unit</label>
            <select
              id="memory-unit"
              value={settings.units.memory}
              onChange={(e) => {
                const newSettings = {
                  ...settings,
                  units: {
                    ...settings.units,
                    memory: e.target.value as "mb" | "gb",
                  },
                };
                setSettings(newSettings);
              }}
            >
              <option value="mb">Megabytes (MB)</option>
              <option value="gb">Gigabytes (GB)</option>
            </select>
          </div>
        </div>
      </section>

      {/* Theme Preferences */}
      <section className="settings-section">
        <h2>Theme Preferences</h2>
        <div className="settings-grid">
          <div className="setting-item">
            <label htmlFor="theme-mode">Theme Mode</label>
            <select
              id="theme-mode"
              value={settings.theme.mode}
              onChange={(e) => {
                const newSettings = {
                  ...settings,
                  theme: {
                    ...settings.theme,
                    mode: e.target.value as "light" | "dark" | "system",
                  },
                };
                setSettings(newSettings);
              }}
            >
              <option value="light">Light</option>
              <option value="dark">Dark</option>
              <option value="system">System</option>
            </select>
          </div>

          <div className="setting-item">
            <label htmlFor="high-contrast">
              <input
                id="high-contrast"
                type="checkbox"
                checked={settings.theme.high_contrast}
                onChange={(e) => {
                  const newSettings = {
                    ...settings,
                    theme: {
                      ...settings.theme,
                      high_contrast: e.target.checked,
                    },
                  };
                  setSettings(newSettings);
                }}
              />
              High Contrast Mode
            </label>
          </div>
        </div>
      </section>

      {/* Advanced Settings */}
      <section className="settings-section">
        <h2>Advanced Settings</h2>
        <div className="settings-grid">
          <div className="setting-item">
            <label htmlFor="debug-logging">
              <input
                id="debug-logging"
                type="checkbox"
                checked={settings.advanced.enable_debug_logging}
                onChange={(e) => {
                  const newSettings = {
                    ...settings,
                    advanced: {
                      ...settings.advanced,
                      enable_debug_logging: e.target.checked,
                    },
                  };
                  setSettings(newSettings);
                }}
              />
              Enable Debug Logging
            </label>
          </div>

          <div className="setting-item">
            <label htmlFor="auto-save">
              <input
                id="auto-save"
                type="checkbox"
                checked={settings.advanced.auto_save_sessions}
                onChange={(e) => {
                  const newSettings = {
                    ...settings,
                    advanced: {
                      ...settings.advanced,
                      auto_save_sessions: e.target.checked,
                    },
                  };
                  setSettings(newSettings);
                }}
              />
              Auto-save Sessions
            </label>
          </div>

          <div className="setting-item">
            <label htmlFor="retention-days">Session Retention (days)</label>
            <input
              id="retention-days"
              type="number"
              min="1"
              max="365"
              value={settings.advanced.session_retention_days || 30}
              onChange={(e) => {
                const newSettings = {
                  ...settings,
                  advanced: {
                    ...settings.advanced,
                    session_retention_days: parseInt(e.target.value) || null,
                  },
                };
                setSettings(newSettings);
              }}
            />
            <small>Set to 0 or leave empty for unlimited retention</small>
          </div>
        </div>
      </section>

      {/* Action Buttons */}
      <div className="settings-actions">
        <button
          className="btn-primary"
          onClick={() => saveSettings(settings)}
          disabled={saving}
        >
          {saving ? "Saving..." : "Save Settings"}
        </button>
        <button
          className="btn-secondary"
          onClick={resetToDefaults}
          disabled={saving}
        >
          Reset to Defaults
        </button>
      </div>
    </div>
  );
}

