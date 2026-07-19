/* eslint-disable @typescript-eslint/no-unnecessary-type-conversion */
import { Link, useLocation } from "@tanstack/react-router";
import { Pin, PinOff, ChevronRight, X } from "lucide-react";
import { useEffect } from "react";
import { useTranslation } from "react-i18next";

import type { MenuGroupType, MenuLinkType, NavGroup } from "@/hooks/use-nav-menu-items";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import { useFocusTrap } from "@/hooks/use-focus-trap";
import { useFilteredMenuItems } from "@/hooks/use-nav-menu-items";
import { ROUTES } from "@/lib/routes";
import { cn } from "@/lib/utils";

// ── Types ──────────────────────────────────────────────────────────

interface FloatingPanelProps {
  group: NavGroup | null;
  isOpen: boolean;
  isPinned: boolean;
  onClose: () => void;
  onMouseEnter?: () => void;
  onMouseLeave?: () => void;
  onNavigate: () => void;
  onTogglePin: () => void;
}

// ── Helpers ────────────────────────────────────────────────────────

function findGroup(groups: MenuGroupType[], section: NavGroup): MenuGroupType | undefined {
  return groups.find((g) => g.section === section);
}

// ── Component ──────────────────────────────────────────────────────

export default function FloatingPanel({
  group,
  isPinned,
  isOpen,
  onClose,
  onNavigate,
  onTogglePin,
  onMouseEnter,
  onMouseLeave,
}: FloatingPanelProps) {
  const { t } = useTranslation();
  const location = useLocation();
  const { groups } = useFilteredMenuItems();
  const containerRef = useFocusTrap(isOpen);

  const currentGroup = group ? findGroup(groups, group) : undefined;

  // Close on Escape
  useEffect(() => {
    if (!isOpen || isPinned) return;

    const handleKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        onClose();
      }
    };
    document.addEventListener("keydown", handleKey);
    return () => { document.removeEventListener("keydown", handleKey); };
  }, [isOpen, isPinned, onClose]);

  // Close on click outside
  useEffect(() => {
    if (!isOpen || isPinned) return;

    const handleClick = (e: MouseEvent) => {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
        onClose();
      }
    };
    // Delay adding listener so the click that opened the panel doesn't close it
    const timer = setTimeout(() => {
      document.addEventListener("mousedown", handleClick);
    }, 0);
    return () => {
      clearTimeout(timer);
      document.removeEventListener("mousedown", handleClick);
    };
  }, [isOpen, isPinned, onClose, containerRef]);

  // When not pinned and panel is open, handle mouse entering/leaving the panel
  // Cancel the close timeout when mouse enters the panel
  const panelMouseEnter = () => {
    onMouseEnter?.();
  };

  const panelMouseLeave = () => {
    if (!isPinned) {
      onMouseLeave?.();
    }
  };

  if (!currentGroup || !isOpen) return null;

  return (
    <aside
      aria-label={currentGroup.label}
      className={cn(
        "hidden md:flex fixed top-0 h-full z-40 flex-col",
        "w-60", // 240px
        "bg-white/95 dark:bg-slate-950/95 backdrop-blur-xl",
        "border-r border-slate-100 dark:border-slate-800",
        "shadow-2xl shadow-slate-300/30 dark:shadow-black/40",
        "animate-in slide-in-from-left-2 fade-in duration-200",
        // Position: starts at the right edge of the 56px icon rail
        "left-14",
      )}
      data-testid={`nav-panel-${group}`}
      onMouseEnter={panelMouseEnter}
      onMouseLeave={panelMouseLeave}
      ref={containerRef}
    >
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-4 border-b border-slate-100 dark:border-slate-800 shrink-0 min-h-[56px]">
        <Typography className="text-sm font-bold text-slate-700 dark:text-slate-200 truncate" variant="h2">
          {currentGroup.label}
        </Typography>
        <div className="flex items-center gap-1">
          {/* Pin toggle */}
          <Button
            aria-label={isPinned ? t("common.collapse", "Unpin") : t("common.expand", "Pin")}
            className={cn(
              "h-8 w-8",
              isPinned
                ? "text-primary-600 dark:text-primary-400 bg-primary-50 dark:bg-primary-950"
                : "text-slate-400 dark:text-slate-500 hover:text-slate-600 dark:hover:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-800",
            )}
            data-testid={`nav-panel-pin-${group}`}
            icon={isPinned ? <PinOff className="h-3.5 w-3.5" /> : <Pin className="h-3.5 w-3.5" />}
            onClick={onTogglePin}
            variant="ghost"
          />
          {/* Close button (only when not pinned) */}
          {!isPinned && (
            <Button
              aria-label={t("common.close")}
              className="h-8 w-8 text-slate-400 dark:text-slate-500 hover:text-slate-600 dark:hover:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-800"
              data-testid={`nav-panel-close-${group}`}
              icon={<X className="h-3.5 w-3.5" />}
              onClick={onClose}
              variant="ghost"
            />
          )}
        </div>
      </div>

      {/* Sub-items */}
      <div className="flex-1 overflow-y-auto custom-scrollbar py-3 px-3 space-y-1" role="menu">
        {currentGroup.items.map((item) => (
          <FloatingPanelLink
            isActive={
              item.exact
                ? location.pathname === item.to
                : item.to === ROUTES.SHIFTS
                  ? location.pathname === item.to ||
                    (location.pathname.startsWith(item.to + "/") &&
                     !location.pathname.startsWith(ROUTES.SHIFTS_RULES))
                  : location.pathname === item.to || location.pathname.startsWith(item.to + "/")
            }
            item={item}
            key={item.to}
            onNavigate={onNavigate}
          />
        ))}
      </div>
    </aside>
  );
}

// ── Internal: Panel Link ───────────────────────────────────────────

function FloatingPanelLink({
  item,
  isActive,
  onNavigate,
}: {
  item: MenuLinkType;
  isActive: boolean;
  onNavigate: () => void;
}) {
  return (
    <Link
      aria-current={isActive ? "page" : undefined}
      className={cn(
        "flex items-center gap-3 px-3 py-2.5 rounded-xl transition-all duration-200 group",
        isActive
          ? "bg-primary-50 dark:bg-primary-950 text-primary-600 dark:text-primary-300 shadow-sm shadow-primary-100/50"
          : "text-slate-500 dark:text-slate-400 hover:bg-slate-50 dark:hover:bg-slate-800 hover:text-slate-700 dark:hover:text-slate-200",
      )}
      data-testid={item.testId}
      onClick={onNavigate}
      role="menuitem"
      to={item.to as "/"}
    >
      {item.icon ? (
        <span
          className={cn(
            "p-1.5 rounded-lg transition-colors",
            isActive
              ? "bg-white dark:bg-primary-900/30 shadow-sm"
              : "bg-slate-50 dark:bg-slate-800 group-hover:bg-white dark:group-hover:bg-slate-700",
          )}
        >
          <item.icon className="h-3.5 w-3.5" />
        </span>
      ) : null}
      <span className="text-sm font-bold truncate flex-1">{item.label}</span>
      {!!isActive && <ChevronRight className="h-3 w-3 opacity-50 shrink-0" />}
    </Link>
  );
}
