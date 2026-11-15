import { SlidersHorizontal, X } from "lucide-react";
import React, { useEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import { useFocusTrap } from "@/hooks/use-focus-trap";
import { useScrollLock } from "@/hooks/use-scroll-lock";
import { cn } from "@/lib/utils";

// ── Types ──────────────────────────────────────────────────────────

export interface ResponsiveToolbarProps {
  /** Action buttons (export, add, etc.) — rendered inline on desktop, icon-only strip on mobile */
  actions?: React.ReactNode;
  /** Breakpoint to switch. Default: "lg" */
  breakpoint?: "md" | "lg" | "xl";
  /** Filter controls — inline on desktop, bottom sheet on mobile */
  children: React.ReactNode;
  /** Additional class for the desktop toolbar wrapper */
  className?: string;
  /** Unified desktop content. Prefer this over separate searchSlot and children layout. */
  desktopContent?: React.ReactNode;
  /** Number of active filters (for badge on mobile trigger button). If > 0, a badge is shown. */
  filterCount?: number;
  /** Hide the inline desktop filter group while retaining the mobile sheet. */
  mobileOnly?: boolean;
  /** Search input slot — legacy desktop slot; use desktopContent for new callers. */
  searchSlot?: React.ReactNode;
}

// ── Breakpoint Visibility Map ──────────────────────────────────────

const BP: Record<NonNullable<ResponsiveToolbarProps["breakpoint"]>, { desktop: string; mobile: string }> = {
  md: { desktop: "hidden md:flex", mobile: "md:hidden" },
  lg: { desktop: "hidden lg:flex", mobile: "lg:hidden" },
  xl: { desktop: "hidden xl:flex", mobile: "xl:hidden" },
};

// ── Internal: Bottom Sheet State (reused pattern from bottom-drawer.tsx) ──

function useBottomSheetState(isOpen: boolean, onClose: () => void) {
  const [isVisible, setIsVisible] = useState(false);
  const closeTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  if (isOpen && !isVisible) {
    setIsVisible(true);
  }

  useEffect(() => {
    if (isOpen) {
      if (closeTimerRef.current) {
        clearTimeout(closeTimerRef.current);
        closeTimerRef.current = null;
      }
      const handleKeyDown = (e: KeyboardEvent) => {
        if (e.key === "Escape") onClose();
      };
      window.addEventListener("keydown", handleKeyDown);
      return () => {
        window.removeEventListener("keydown", handleKeyDown);
      };
    }

    closeTimerRef.current = setTimeout(() => {
      setIsVisible(false);
      closeTimerRef.current = null;
    }, 300);
    return () => {
      if (closeTimerRef.current) {
        clearTimeout(closeTimerRef.current);
        closeTimerRef.current = null;
      }
    };
  }, [isOpen, onClose]);

  return isVisible;
}

// ── Internal: Filter Bottom Sheet (mobile) ─────────────────────────

function FilterSheet({
  isOpen,
  onClose,
  children,
  filterCount,
}: {
  children: React.ReactNode;
  filterCount: number;
  isOpen: boolean;
  onClose: () => void;
}) {
  const { t } = useTranslation();
  const containerRef = useFocusTrap(isOpen);
  const isVisible = useBottomSheetState(isOpen, onClose);
  useScrollLock(isOpen);

  if (!isVisible && !isOpen) return null;

  return createPortal(
    <div className="fixed inset-0 z-[1000] flex items-end">
      {/* Backdrop */}
      {/* eslint-disable-next-line jsx-a11y/no-static-element-interactions, jsx-a11y/click-events-have-key-events */}
      <div
        className={cn(
          "fixed inset-0 bg-slate-900/40 dark:bg-slate-950/60 backdrop-blur-sm transition-opacity duration-300",
          isOpen ? "opacity-100" : "opacity-0",
        )}
        onClick={onClose}
      />

      {/* Panel */}
      <div
        aria-label={t("common.filters.title", "Filters")}
        aria-modal="true"
        className={cn(
          "relative w-full max-h-[80dvh] bg-white dark:bg-slate-950 rounded-t-3xl flex flex-col",
          "transform transition-transform duration-300 ease-[cubic-bezier(0.32,0.72,0,1)]",
          isOpen ? "translate-y-0" : "translate-y-full",
        )}
        data-testid="filter-sheet"
        ref={containerRef}
        role="dialog"
      >
        {/* Drag handle */}
        <div className="flex justify-center pt-3 pb-1 shrink-0">
          <div className="w-10 h-1 rounded-full bg-slate-300 dark:bg-slate-700" />
        </div>

        {/* Header */}
        <div className="flex items-center justify-between px-5 py-3 border-b border-slate-100 dark:border-slate-800 shrink-0">
          <div className="flex items-center gap-2">
            <SlidersHorizontal className="h-4 w-4 text-slate-500 dark:text-slate-400" />
            <Typography className="text-sm font-bold text-slate-700 dark:text-slate-200 uppercase tracking-widest" variant="h2">
              {t("common.filters.title", "Filters")}
            </Typography>
            {filterCount > 0 ? (
              <span className="inline-flex items-center justify-center w-5 h-5 rounded-full bg-primary-500 text-white text-[10px] font-bold">
                {filterCount}
              </span>
            ) : null}
          </div>
          <Button
            aria-label={t("common.close")}
            className="h-8 w-8 text-slate-400 dark:text-slate-500 hover:text-slate-600 dark:hover:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-800"
            data-testid="filter-sheet-close"
            icon={<X className="h-4 w-4" />}
            onClick={onClose}
            variant="ghost"
          />
        </div>

        {/* Scrollable filter content */}
        <div className="flex-1 overflow-y-auto overscroll-contain px-5 py-4 space-y-4">
          {children}
        </div>

        {/* Footer with apply button */}
        <div className="shrink-0 border-t border-slate-100 dark:border-slate-800 px-5 py-3 pb-safe">
          <Button
            className="w-full py-3.5 rounded-2xl font-bold uppercase tracking-widest text-xs"
            data-testid="filter-sheet-apply"
            onClick={onClose}
            variant="primary"
          >
            {t("common.apply", "Apply")}
          </Button>
        </div>
      </div>
    </div>,
    document.body,
  );
}

// ── Main Component ─────────────────────────────────────────────────

export default function ResponsiveToolbar({
  children,
  actions,
  desktopContent,
  searchSlot,
  filterCount = 0,
  breakpoint: bp = "lg",
  className,
  mobileOnly = false,
}: ResponsiveToolbarProps) {
  const { t } = useTranslation();
  const [isFilterOpen, setIsFilterOpen] = useState(false);
  const { desktop, mobile } = BP[bp];

  return (
    <>
      <div className={cn("flex w-full min-w-0 flex-wrap items-center gap-2", mobileOnly && "lg:hidden", className)}>
        {desktopContent ? (
          <div className={cn(desktop, "min-w-0 flex-1 items-center", mobileOnly && "lg:hidden")}>
            {desktopContent}
          </div>
        ) : (
          <>
            {searchSlot ? <div className="min-w-0 flex-1 lg:max-w-md">{searchSlot}</div> : null}
            <div className={cn(desktop, "min-w-0 flex-1 items-center gap-3", mobileOnly && "lg:hidden")}>
              <div className="flex flex-1 flex-wrap items-center gap-3">{children}</div>
            </div>
          </>
        )}

        <div className={cn(mobile, "items-center gap-2")}>
          <Button
            aria-label={t("common.filters.title", "Filters")}
            className="relative shrink-0 px-3 py-3 rounded-2xl bg-slate-100 dark:bg-slate-800 text-slate-600 dark:text-slate-300 hover:bg-slate-200 dark:hover:bg-slate-700 border border-slate-200 dark:border-slate-700"
            data-testid="filter-trigger"
            onClick={() => { setIsFilterOpen(true); }}
            variant="ghost"
          >
            <SlidersHorizontal className="h-5 w-5" />
            {filterCount > 0 ? (
              <span className="absolute -top-1 -right-1 w-4 h-4 rounded-full bg-primary-500 text-white text-[9px] font-bold flex items-center justify-center">
                {filterCount}
              </span>
            ) : null}
          </Button>
        </div>

        {actions ? <div className="flex shrink-0 items-center gap-2">{actions}</div> : null}
      </div>

      {/* ── Mobile Filter Sheet ────────────────────────────────────── */}
      <FilterSheet
        filterCount={filterCount}
        isOpen={isFilterOpen}
        onClose={() => { setIsFilterOpen(false); }}
      >
        {children}
      </FilterSheet>
    </>
  );
}
