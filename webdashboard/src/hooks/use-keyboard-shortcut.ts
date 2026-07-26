import { useEffect } from "react";

interface UseKeyboardShortcutOptions {
  /** Keyboard shortcut string like "Ctrl+Shift+D". Parsed case-insensitively. */
  shortcut?: string;
  /** Callback fired when the shortcut combination is pressed. */
  onTrigger: () => void;
  /** When true, the shortcut is suppressed (e.g. panel already open). */
  disabled?: boolean;
}

/**
 * Register a global keyboard shortcut that fires `onTrigger` when the key
 * combination is pressed. Ignores events from input/textarea/contenteditable
 * elements so the shortcut doesn't steal focus from form fields.
 *
 * Intended for accessibility — gives power users a fast path to open a filter
 * or search panel without reaching for the mouse.
 */
export function useKeyboardShortcut({
  shortcut,
  onTrigger,
  disabled = false,
}: UseKeyboardShortcutOptions): void {
  useEffect(() => {
    if (!shortcut) return;

    const isInputTarget = (target: HTMLElement): boolean =>
      ["INPUT", "TEXTAREA"].includes(target.tagName) || target.isContentEditable;

    const handler = (e: KeyboardEvent): void => {
      if (disabled) return;
      if (isInputTarget(e.target as HTMLElement)) return;

      const parts = shortcut.toLowerCase().split("+");
      const key = parts[parts.length - 1];
      const ctrl = parts.includes("ctrl");
      const shift = parts.includes("shift");
      const alt = parts.includes("alt");
      const meta = parts.includes("meta");

      if (
        e.key.toLowerCase() === key &&
        e.ctrlKey === ctrl &&
        e.shiftKey === shift &&
        e.altKey === alt &&
        e.metaKey === meta
      ) {
        e.preventDefault();
        onTrigger();
      }
    };

    window.addEventListener("keydown", handler);
    return () => {
      window.removeEventListener("keydown", handler);
    };
  }, [shortcut, onTrigger, disabled]);
}
