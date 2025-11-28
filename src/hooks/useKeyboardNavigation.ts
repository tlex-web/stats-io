//! Keyboard navigation hook
//!
//! Provides keyboard navigation support for accessibility
//! following IMPLEMENTATION_PLAN.md Phase 3.5.

import { useEffect, useRef } from "react";

export function useKeyboardNavigation() {
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Tab navigation is handled by browser
      // Arrow key navigation for custom components
      if (e.key === "ArrowDown" || e.key === "ArrowUp") {
        const focusableElements = containerRef.current?.querySelectorAll(
          'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
        ) as NodeListOf<HTMLElement> | undefined;

        if (!focusableElements || focusableElements.length === 0) return;

        const currentIndex = Array.from(focusableElements).findIndex(
          (el) => el === document.activeElement
        );

        if (currentIndex === -1) {
          focusableElements[0]?.focus();
          return;
        }

        let nextIndex: number;
        if (e.key === "ArrowDown") {
          nextIndex = (currentIndex + 1) % focusableElements.length;
        } else {
          nextIndex = (currentIndex - 1 + focusableElements.length) % focusableElements.length;
        }

        focusableElements[nextIndex]?.focus();
        e.preventDefault();
      }

      // Escape key to close modals/dropdowns
      if (e.key === "Escape") {
        const activeElement = document.activeElement as HTMLElement;
        if (activeElement?.blur) {
          activeElement.blur();
        }
      }

      // Enter/Space for button activation (handled by browser, but we can add custom logic)
      if ((e.key === "Enter" || e.key === " ") && e.target instanceof HTMLElement) {
        if (e.target.getAttribute("role") === "button" && !e.target.hasAttribute("tabindex")) {
          e.preventDefault();
          e.target.click();
        }
      }
    };

    document.addEventListener("keydown", handleKeyDown);
    return () => {
      document.removeEventListener("keydown", handleKeyDown);
    };
  }, []);

  return containerRef;
}

/// Keyboard shortcut handler
export function useKeyboardShortcuts(shortcuts: Record<string, () => void>) {
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      const key = e.key.toLowerCase();
      const modifier = e.ctrlKey || e.metaKey ? "ctrl" : e.altKey ? "alt" : "";
      const shortcutKey = modifier ? `${modifier}+${key}` : key;

      if (shortcuts[shortcutKey]) {
        e.preventDefault();
        shortcuts[shortcutKey]();
      }
    };

    document.addEventListener("keydown", handleKeyDown);
    return () => {
      document.removeEventListener("keydown", handleKeyDown);
    };
  }, [shortcuts]);
}

