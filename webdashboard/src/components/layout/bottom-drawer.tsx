import { Link, useLocation } from "@tanstack/react-router";
import { X, Moon, Sun, LogOut } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { useTranslation } from "react-i18next";

import type { NavGroup } from "@/hooks/use-nav-menu-items";

import Button from "@/components/ui/button";
import { useDarkModeContext } from "@/context/use-dark-mode-context";
import { useFocusTrap } from "@/hooks/use-focus-trap";
import { useLanguage } from "@/hooks/use-language";
import { useFilteredMenuItems, getActiveNavGroup } from "@/hooks/use-nav-menu-items";
import { ROUTES } from "@/lib/routes";
import { cn } from "@/lib/utils";

interface BottomDrawerProps {
  isOpen: boolean;
  onClose: () => void;
}

// ── Internal: Bottom Sheet State ─────────────────────────────────────

/**
 * Manages bottom sheet visibility for animation.
 * Opens immediately; delays hide by 300ms for exit transition.
 */
function useBottomSheetState(isOpen: boolean, onClose: () => void) {
  const [isVisible, setIsVisible] = useState(false);
  const closeTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Handle open: show immediately during render
  if (isOpen && !isVisible) {
    setIsVisible(true);
  }

  useEffect(() => {
    if (isOpen) {
      if (closeTimerRef.current) {
        clearTimeout(closeTimerRef.current);
        closeTimerRef.current = null;
      }
      document.body.style.overflow = "hidden";

      const handleKeyDown = (e: KeyboardEvent) => {
        if (e.key === "Escape") onClose();
      };
      window.addEventListener("keydown", handleKeyDown);
      return () => {
        window.removeEventListener("keydown", handleKeyDown);
        document.body.style.overflow = "unset";
      };
    }

    // isOpen became false — delay hide for exit animation
    closeTimerRef.current = setTimeout(() => {
      setIsVisible(false);
      closeTimerRef.current = null;
    }, 300);
    document.body.style.overflow = "unset";

    return () => {
      if (closeTimerRef.current) {
        clearTimeout(closeTimerRef.current);
        closeTimerRef.current = null;
      }
    };
  }, [isOpen, onClose]);

  return isVisible;
}

// ── Internal: Swipe-to-close ─────────────────────────────────────────

function useSwipeToClose(
  panelRef: React.RefObject<HTMLDivElement | null>,
  onClose: () => void,
  isOpen: boolean,
) {
  const touchStartY = useRef(0);
  const touchCurrentY = useRef(0);

  useEffect(() => {
    const panel = panelRef.current;
    if (!panel || !isOpen) return;

    const handleTouchStart = (e: TouchEvent) => {
      // Only activate swipe when scroll is at top
      if (panel.scrollTop > 0) return;
      touchStartY.current = e.touches[0].clientY;
      touchCurrentY.current = e.touches[0].clientY;
    };

    const handleTouchMove = (e: TouchEvent) => {
      touchCurrentY.current = e.touches[0].clientY;
    };

    const handleTouchEnd = () => {
      const deltaY = touchCurrentY.current - touchStartY.current;
      // Swipe down > 80px from top of scroll
      if (deltaY > 80 && panel.scrollTop <= 0) {
        onClose();
      }
    };

    panel.addEventListener("touchstart", handleTouchStart, { passive: true });
    panel.addEventListener("touchmove", handleTouchMove, { passive: true });
    panel.addEventListener("touchend", handleTouchEnd, { passive: true });

    return () => {
      panel.removeEventListener("touchstart", handleTouchStart);
      panel.removeEventListener("touchmove", handleTouchMove);
      panel.removeEventListener("touchend", handleTouchEnd);
    };
  }, [panelRef, onClose, isOpen]);
}

// ── Main Component ───────────────────────────────────────────────────

export default function BottomDrawer({ isOpen, onClose }: BottomDrawerProps) {
  const { t } = useTranslation();
  const location = useLocation();
  const { groups } = useFilteredMenuItems();
  const containerRef = useFocusTrap(isOpen);
  const panelRef = useRef<HTMLDivElement>(null);
  const isVisible = useBottomSheetState(isOpen, onClose);

  useSwipeToClose(panelRef, onClose, isOpen);

  const pathActiveGroup = getActiveNavGroup(location.pathname);

  if (!isVisible && !isOpen) return null;

  return createPortal(
    <div className="fixed inset-0 z-[1000] flex items-end">
      {/* Backdrop */}
      {/* eslint-disable-next-line jsx-a11y/no-static-element-interactions */}
      <div
        aria-hidden="true"
        className={cn(
          "fixed inset-0 bg-slate-900/40 dark:bg-slate-950/60 backdrop-blur-sm transition-opacity duration-300",
          isOpen ? "opacity-100" : "opacity-0",
        )}
        onClick={onClose}
        onKeyDown={(e) => { if (e.key === "Enter" || e.key === " ") onClose(); }}
        role="button"
        tabIndex={0}
      />

      {/* Panel — role="dialog" is interactive */}
      {/* eslint-disable-next-line jsx-a11y/no-noninteractive-element-interactions, jsx-a11y/click-events-have-key-events */}
      <div
        aria-label={t("common.settings.navigation", "Navigation")}
        aria-modal="true"
        className={cn(
          "relative w-full max-h-[85vh] bg-white dark:bg-slate-950 rounded-t-3xl flex flex-col",
          "transform transition-transform duration-300 ease-[cubic-bezier(0.32,0.72,0,1)]",
          isOpen ? "translate-y-0" : "translate-y-full",
        )}
        data-testid="bottom-drawer"
        onClick={(e) => { e.stopPropagation(); }}
        // eslint-disable-next-line react-hooks/immutability -- ref callback merging multiple refs is standard React pattern
        ref={(node) => {
          // Merge refs for focus trap and swipe detection
          // eslint-disable-next-line react-hooks/immutability
          (containerRef as React.MutableRefObject<HTMLDivElement | null>).current = node;
          // eslint-disable-next-line react-hooks/immutability
          (panelRef as React.MutableRefObject<HTMLDivElement | null>).current = node;
        }}
        role="dialog"
      >
        {/* Drag handle */}
        <div className="flex justify-center pt-3 pb-1 shrink-0">
          <div className="w-10 h-1 rounded-full bg-slate-300 dark:bg-slate-700" />
        </div>

        {/* Header */}
        <div className="flex items-center justify-between px-5 py-3 border-b border-slate-100 dark:border-slate-800 shrink-0">
          <h2 className="text-sm font-bold text-slate-700 dark:text-slate-200 uppercase tracking-widest">
            {t("common.settings.navigation", "Navigation")}
          </h2>
          <Button
            aria-label={t("common.close")}
            className="h-8 w-8 text-slate-400 dark:text-slate-500 hover:text-slate-600 dark:hover:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-800"
            data-testid="bottom-drawer-close"
            icon={<X className="h-4 w-4" />}
            onClick={onClose}
            variant="ghost"
          />
        </div>

        {/* Scrollable sections */}
        <div className="flex-1 overflow-y-auto custom-scrollbar px-4 py-4 space-y-6" ref={panelRef}>
          {groups.map((group) => (
            <DrawerSection
              activeGroup={pathActiveGroup}
              group={group}
              key={group.section}
              onClose={onClose}
            />
          ))}
        </div>

        {/* Utility row */}
        <DrawerUtilityRow onClose={onClose} />
      </div>
    </div>,
    document.body,
  );
}

// ── Internal: Drawer Section ─────────────────────────────────────────

function DrawerSection({
  group,
  activeGroup,
  onClose,
}: {
  activeGroup: NavGroup | null;
  group: { items: { icon?: React.ElementType; label: string; testId: string; to: string; exact?: boolean }[]; label: string; section: string; sectionIcon: React.ElementType };
  onClose: () => void;
}) {
  const location = useLocation();
  const SectionIcon = group.sectionIcon;

  return (
    <div className="space-y-2">
      <div className="flex items-center gap-2 px-3 py-1">
        <SectionIcon className="h-3.5 w-3.5 text-slate-400 dark:text-slate-500" />
        <span className="text-[10px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-[0.2em]">
          {group.label}
        </span>
      </div>
      <div className="space-y-1">
        {group.items.map((item) => {
          const isActive = item.exact
            ? location.pathname === item.to
            : location.pathname === item.to || location.pathname.startsWith(item.to + "/");

          return (
            <Link
              aria-current={isActive ? "page" : undefined}
              className={cn(
                "flex items-center gap-3 px-4 py-3 rounded-2xl transition-all duration-200",
                isActive
                  ? "bg-primary-50 dark:bg-primary-950 text-primary-600 dark:text-primary-300"
                  : "text-slate-600 dark:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-800",
              )}
              data-testid={item.testId}
              key={item.to}
              onClick={onClose}
              to={item.to as "/"}
            >
              {item.icon ? (
                <span className={cn(
                  "p-2 rounded-xl transition-colors",
                  isActive
                    ? "bg-white dark:bg-primary-900/30 shadow-sm"
                    : "bg-slate-50 dark:bg-slate-800",
                )}>
                  <item.icon className="h-4 w-4" />
                </span>
              ) : null}
              <span className="text-sm font-bold truncate">{item.label}</span>
            </Link>
          );
        })}
      </div>
    </div>
  );
}

// ── Internal: Utility Row ────────────────────────────────────────────

function DrawerUtilityRow({ onClose }: { onClose: () => void }) {
  const { t } = useTranslation();
  const { mode, setMode } = useDarkModeContext();
  const { currentLanguage, changeLanguage, currentOption } = useLanguage();

  const cycleTheme = () => {
    const modes: ("light" | "system" | "dark")[] = ["light", "system", "dark"];
    setMode(modes[(modes.indexOf(mode) + 1) % 3]);
  };
  const ThemeIcon = mode === "dark" ? Moon : Sun;

  const cycleLanguage = () => {
    const codes: ("en" | "vi" | "zh")[] = ["vi", "en", "zh"];
    void changeLanguage(codes[(codes.indexOf(currentLanguage) + 1) % codes.length]);
  };

  const handleLogout = () => {
    onClose();
    localStorage.clear();
    window.location.href = ROUTES.LOGIN;
  };

  return (
    <div className="shrink-0 border-t border-slate-100 dark:border-slate-800 px-4 py-3 pb-safe">
      <div className="flex items-center justify-between gap-2">
        {/* Theme toggle */}
        <button
          aria-label={t(`common.theme.${mode}`, mode)}
          className="flex items-center gap-2 px-4 py-3 rounded-2xl text-slate-500 dark:text-slate-400 hover:bg-slate-50 dark:hover:bg-slate-800 transition-colors flex-1"
          data-testid="bottom-drawer-theme-toggle"
          onClick={cycleTheme}
          type="button"
        >
          <ThemeIcon className="h-4 w-4" />
          <span className="text-xs font-bold">{t(`common.theme.${mode}`, mode)}</span>
        </button>

        {/* Language toggle */}
        <button
          aria-label={t("appLauncher.language", "Language") + ": " + currentOption.nativeName}
          className="flex items-center gap-2 px-4 py-3 rounded-2xl text-slate-500 dark:text-slate-400 hover:bg-slate-50 dark:hover:bg-slate-800 transition-colors flex-1"
          data-testid="bottom-drawer-lang-toggle"
          onClick={cycleLanguage}
          type="button"
        >
          <span className="text-[11px] font-black tracking-tighter">
            {currentLanguage.toUpperCase()}
          </span>
          <span className="text-xs font-bold truncate">{currentOption.nativeName}</span>
        </button>

        {/* Logout */}
        <button
          aria-label={t("common.logout")}
          className="flex items-center gap-2 px-4 py-3 rounded-2xl text-rose-500 dark:text-rose-400 hover:bg-rose-50 dark:hover:bg-rose-950/30 transition-colors flex-1"
          data-testid="bottom-drawer-logout"
          onClick={handleLogout}
          type="button"
        >
          <LogOut className="h-4 w-4" />
          <span className="text-xs font-bold">{t("common.logout")}</span>
        </button>
      </div>
    </div>
  );
}
