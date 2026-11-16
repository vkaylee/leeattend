import { SlidersHorizontal, X } from "lucide-react";
import { useEffect, useRef, useState, type ReactNode } from "react";
import { createPortal } from "react-dom";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import { useFocusTrap } from "@/hooks/use-focus-trap";
import { useScrollLock } from "@/hooks/use-scroll-lock";
import { cn } from "@/lib/utils";

export interface ResponsiveToolbarProps {
  actions?: ReactNode;
  breakpoint?: "md" | "lg" | "xl";
  children: ReactNode;
  className?: string;
  desktopContent?: ReactNode;
  filterCount?: number;
  mobileOnly?: boolean;
  searchSlot?: ReactNode;
}

type Breakpoint = NonNullable<ResponsiveToolbarProps["breakpoint"]>;
interface VisibilityClasses {
  desktop: string;
  mobile: string;
}

const BP: Record<Breakpoint, VisibilityClasses> = {
  md: { desktop: "hidden md:flex", mobile: "md:hidden" },
  lg: { desktop: "hidden lg:flex", mobile: "lg:hidden" },
  xl: { desktop: "hidden xl:flex", mobile: "xl:hidden" },
};

function useBottomSheetState(isOpen: boolean, onClose: () => void) {
  const [isVisible, setIsVisible] = useState(false);
  const closeTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  if (isOpen && !isVisible) setIsVisible(true);

  useEffect(() => {
    if (isOpen) {
      if (closeTimerRef.current) clearTimeout(closeTimerRef.current);
      const handleKeyDown = (event: KeyboardEvent) => { if (event.key === "Escape") onClose(); };
      window.addEventListener("keydown", handleKeyDown);
      return () => { window.removeEventListener("keydown", handleKeyDown); };
    }
    closeTimerRef.current = setTimeout(() => { setIsVisible(false); closeTimerRef.current = null; }, 300);
    return () => { if (closeTimerRef.current) clearTimeout(closeTimerRef.current); };
  }, [isOpen, onClose]);

  return isVisible;
}

function SheetHeader({ filterCount, onClose }: { filterCount: number; onClose: () => void }) {
  const { t } = useTranslation();
  return <div className="flex items-center justify-between border-b border-slate-100 px-5 py-4 dark:border-slate-800"><Typography className="font-bold" variant="h2">{t("common.filters.title", "Filters")}{filterCount > 0 ? ` (${filterCount})` : ""}</Typography><Button aria-label={t("common.close", "Close")} onClick={onClose} size="icon" variant="ghost"><X className="h-4 w-4" /></Button></div>;
}

interface FilterSheetProps {
  children: ReactNode;
  filterCount: number;
  isOpen: boolean;
  onClose: () => void;
}

function FilterSheet({ children, filterCount, isOpen, onClose }: FilterSheetProps) {
  const { t } = useTranslation();
  const containerRef = useFocusTrap(isOpen);
  const isVisible = useBottomSheetState(isOpen, onClose);
  useScrollLock(isOpen);
  if (!isVisible && !isOpen) return null;

  return createPortal(
    <div className="fixed inset-0 z-[1000] flex items-end">
      <button
        aria-label={t("common.close", "Close")}
        className={cn(
          "fixed inset-0 bg-slate-900/40 dark:bg-slate-950/60 backdrop-blur-sm transition-opacity duration-300",
          isOpen ? "opacity-100" : "opacity-0",
        )}
        onClick={onClose}
        type="button"
      />
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
        <SheetHeader filterCount={filterCount} onClose={onClose} />
        <div className="flex-1 overflow-y-auto overscroll-contain px-5 py-4 space-y-4">
          {children}
        </div>
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
