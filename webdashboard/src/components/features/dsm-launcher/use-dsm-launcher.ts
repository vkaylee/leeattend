import { useCallback, useEffect, useState } from "react";

export interface UseDsmLauncherReturn {
  close: () => void;
  isOpen: boolean;
  open: () => void;
  setIsOpen: (open: boolean) => void;
  toggle: () => void;
}

/**
 * Globally triggers the DSM 7 Launcher overlay from anywhere in the application.
 */
export function openDsmLauncher(): void {
  if (typeof window !== "undefined") {
    window.dispatchEvent(new CustomEvent("open-dsm-launcher"));
  }
}

/**
 * Hook to manage DSM 7 Launcher visibility and register Cmd+K / Ctrl+K keyboard shortcut.
 */
export function useDsmLauncher(): UseDsmLauncherReturn {
  const [isOpen, setIsOpen] = useState(false);

  const open = useCallback(() => {
    setIsOpen(true);
  }, []);

  const close = useCallback(() => {
    setIsOpen(false);
  }, []);

  const toggle = useCallback(() => {
    setIsOpen((prev) => !prev);
  }, []);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Cmd+K or Ctrl+K
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        setIsOpen((prev) => !prev);
      }
    };

    const handleOpenEvent = () => {
      setIsOpen(true);
    };

    window.addEventListener("keydown", handleKeyDown);
    window.addEventListener("open-dsm-launcher", handleOpenEvent);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
      window.removeEventListener("open-dsm-launcher", handleOpenEvent);
    };
  }, []);

  return {
    isOpen,
    setIsOpen,
    open,
    close,
    toggle,
  };
}
