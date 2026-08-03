import { useCallback, useEffect, useRef, useState } from "react";

import type { NavGroup } from "@/hooks/use-nav-menu-items";

import { getActiveNavGroup } from "@/hooks/use-nav-menu-items";

// ── Types ──────────────────────────────────────────────────────────

export interface NavSidebarState {
  /** Which group's floating panel is currently visible. */
  activeGroup: NavGroup | null;
  /** Hover handler with 150ms debounce for open, 300ms for close. */
  handleGroupHover: (group: NavGroup | null) => void;
  /** Call when mouse enters the floating panel — cancels pending close. */
  handlePanelEnter: () => void;
  /** Call when mouse leaves the floating panel while not pinned. */
  handlePanelLeave: () => void;
  /** Whether the bottom drawer (mobile "More" menu) is open. */
  isBottomDrawerOpen: boolean;
  /** Whether the floating panel is pinned (locked open). Persisted in localStorage. */
  isPinned: boolean;
  /** The auto-detected active group from the current path. */
  pathActiveGroup: NavGroup | null;
  /** Set the active group (for explicit click — bypasses debounce, blocks hover-close). */
  setActiveGroup: (group: NavGroup | null) => void;
  /** Open/close the bottom drawer. */
  setBottomDrawerOpen: (open: boolean) => void;
  /** Toggle pin state. */
  togglePin: () => void;
}

const PIN_STORAGE_KEY = "nav-rail-pinned";

function getInitialPinned(): boolean {
  try {
    return localStorage.getItem(PIN_STORAGE_KEY) === "true";
  } catch { /* localStorage unavailable (private browsing) — default to unpinned */
    return false;
  }
}

// ── Hook ───────────────────────────────────────────────────────────

export function useNavSidebar(locationPathname: string): NavSidebarState {
  const [isPinned, setIsPinned] = useState(getInitialPinned);
  const [activeGroup, setActiveGroupState] = useState<NavGroup | null>(null);
  const [isBottomDrawerOpen, setIsBottomDrawerOpen] = useState(false);

  // Debounce refs
  const hoverOpenRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const hoverCloseRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  // When true, hover-close is blocked (set after explicit click)
  const clickedOpenRef = useRef(false);

  const pathActiveGroup = getActiveNavGroup(locationPathname);

  // ── Pin toggle ───────────────────────────────────────────────
  const togglePin = useCallback(() => {
    setIsPinned((prev) => {
      const next = !prev;
      try { localStorage.setItem(PIN_STORAGE_KEY, String(next)); } catch { /* localStorage unavailable — pin state won't persist */ /* noop */ }
      return next;
    });
  }, []);

  // ── Clear all timers ─────────────────────────────────────────
  const clearTimers = useCallback(() => {
    if (hoverOpenRef.current) { clearTimeout(hoverOpenRef.current); hoverOpenRef.current = null; }
    if (hoverCloseRef.current) { clearTimeout(hoverCloseRef.current); hoverCloseRef.current = null; }
  }, []);

  // ── Explicit click: bypass debounce, block hover-close ───────
  const setActiveGroup = useCallback((group: NavGroup | null) => {
    clearTimers();
    setActiveGroupState(group);
    if (group !== null) {
      clickedOpenRef.current = true; // Block hover-close until mouse re-enters rail
    } else {
      clickedOpenRef.current = false;
    }
  }, [clearTimers]);

  // ── Hover handler with debounce ──────────────────────────────
  const handleGroupHover = useCallback(
    (group: NavGroup | null) => {
      if (isPinned) return;
      clearTimers();

      if (group === null) {
        // Mouse left the rail — if panel was opened by click, don't close
        if (clickedOpenRef.current) return;
        // Capture pathname; if it changes before timer fires, ignore
        const pathAtSet = pathRef.current;
        hoverCloseRef.current = setTimeout(() => {
          if (pathRef.current !== pathAtSet) return; // navigation happened
          setActiveGroupState(null);
        }, 300);
      } else {
        // Mouse entered a group icon — clear the clicked-open lock since user
        // is now actively hovering, which means hover behavior takes over
        clickedOpenRef.current = false;
        const pathAtSet = pathRef.current;
        hoverOpenRef.current = setTimeout(() => {
          if (pathRef.current !== pathAtSet) return; // navigation happened
          setActiveGroupState(group);
        }, 150);
      }
    },
    [isPinned, clearTimers],
  );

  // ── Panel mouse enter: cancel any pending close timer ────────
  const handlePanelEnter = useCallback(() => {
    clearTimers();
    // Do NOT clear clickedOpenRef here — the lock must survive
    // so the panel stays open after the user moves the mouse away.
  }, [clearTimers]);

  // ── Panel mouse leave: do nothing. Close is handled by rail leave ─
  // or explicit actions (Escape, click-outside, navigate).
  const handlePanelLeave = useCallback(() => {
    // Intentionally empty — the panel stays open as long as the mouse
    // is on either the rail or the panel. Closing is triggered by
    // handleGroupHover(null) when the mouse leaves the rail entirely.
  }, []);

  // ── Bottom drawer state ──────────────────────────────────────
  const setBottomDrawerOpen = useCallback((open: boolean) => {
    setIsBottomDrawerOpen(open);
  }, []);

  // Track pathname at timer set time — stale timers from before
  // navigation are silently ignored when they fire.
  // pathRef is intentionally mutated inside effect (not state) — safe pattern.
  const pathRef = useRef(locationPathname);
  // eslint-disable-next-line react-hooks/immutability
  useEffect(() => { pathRef.current = locationPathname; }, [locationPathname]);

  // Cleanup all timers on unmount
  useEffect(() => {
    return () => { clearTimers(); };
  }, [clearTimers]);

  return {
    isPinned,
    activeGroup,
    isBottomDrawerOpen,
    togglePin,
    setActiveGroup,
    handleGroupHover,
    handlePanelEnter,
    handlePanelLeave,
    setBottomDrawerOpen,
    pathActiveGroup,
  };
}
