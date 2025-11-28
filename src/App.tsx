import { useState, useEffect } from "react";
import { Dashboard } from "./components/Dashboard";
import { ComparisonView } from "./components/ComparisonView";
import { SettingsView } from "./components/SettingsView";
import { useTheme } from "./hooks/useTheme";
import { useKeyboardShortcuts } from "./hooks/useKeyboardNavigation";
import "./App.css";

type View = "dashboard" | "comparison" | "settings";

function App() {
  const [currentView, setCurrentView] = useState<View>("dashboard");
  const { loading: themeLoading } = useTheme();

  // Keyboard shortcuts
  useKeyboardShortcuts({
    "ctrl+1": () => setCurrentView("dashboard"),
    "ctrl+2": () => setCurrentView("comparison"),
    "ctrl+3": () => setCurrentView("settings"),
  });

  // Focus management for accessibility
  useEffect(() => {
    // Ensure focus is visible
    const style = document.createElement("style");
    style.textContent = `
      *:focus-visible {
        outline: 2px solid var(--accent-color);
        outline-offset: 2px;
      }
    `;
    document.head.appendChild(style);
    return () => {
      document.head.removeChild(style);
    };
  }, []);

  if (themeLoading) {
    return (
      <div className="container">
        <div className="loading">Loading...</div>
      </div>
    );
  }

  return (
    <div className="container">
      <nav className="main-nav" role="navigation" aria-label="Main navigation">
        <button
          className={currentView === "dashboard" ? "active" : ""}
          onClick={() => setCurrentView("dashboard")}
          aria-current={currentView === "dashboard" ? "page" : undefined}
          aria-label="Dashboard view"
        >
          Dashboard
        </button>
        <button
          className={currentView === "comparison" ? "active" : ""}
          onClick={() => setCurrentView("comparison")}
          aria-current={currentView === "comparison" ? "page" : undefined}
          aria-label="Comparison view"
        >
          Comparison
        </button>
        <button
          className={currentView === "settings" ? "active" : ""}
          onClick={() => setCurrentView("settings")}
          aria-current={currentView === "settings" ? "page" : undefined}
          aria-label="Settings view"
        >
          Settings
        </button>
      </nav>
      <main role="main">
        {currentView === "dashboard" && <Dashboard />}
        {currentView === "comparison" && <ComparisonView />}
        {currentView === "settings" && <SettingsView />}
      </main>
    </div>
  );
}

export default App;
