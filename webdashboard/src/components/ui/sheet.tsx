import { X, Menu } from "lucide-react";
import React, { useEffect, useId, useRef, useState } from "react";
import { createPortal } from "react-dom";

import { Typography } from "@/components/ui/typography";
import { useFocusTrap } from "@/hooks/use-focus-trap";
import { useTranslation } from "react-i18next";
import { useScrollLock } from "@/hooks/use-scroll-lock";
import { cn } from "@/lib/utils";

interface SheetProps {
  ariaDescribedBy?: string;
  ariaLabelledBy?: string;
  children: React.ReactNode;
  className?: string;
  contentClassName?: string;
  headerExtra?: React.ReactNode;
  /** If true, the built-in header (icon + title + X button) is hidden — use when providing a fully custom header in children */
  hideHeader?: boolean;
  isOpen: boolean;
  onClose: () => void;
  side?: "left" | "right";
  subtitle?: string;
  title?: string;
}

/**
 * Tracks sheet visibility for animation purposes.
 * Opens immediately when isOpen→true; delays hide by 500ms when isOpen→false
 * to allow the exit transition to complete.
 */
function useSheetState(isOpen: boolean, onClose: () => void) {
  const [isVisible, setIsVisible] = useState(false);
  const closeTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Handle open: show immediately during render if isOpen changed to true
  if (isOpen && !isVisible) {
    // Sync during render — no additional render pass
    setIsVisible(true);
  }

  // Handle close & body scroll: side effects only (no setState in useEffect)
  useEffect(() => {
    if (isOpen) {
      // Cancel any pending close timer when re-opening
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

    // isOpen became false — delay hide for exit animation
    closeTimerRef.current = setTimeout(() => {
      setIsVisible(false);
      closeTimerRef.current = null;
    }, 500);
    return () => {
      if (closeTimerRef.current) {
        clearTimeout(closeTimerRef.current);
        closeTimerRef.current = null;
      }
    };
  }, [isOpen, onClose]);

  return isVisible;
}

function getPanelTransform(side: "left" | "right", isOpen: boolean): string {
  const roundedClass = side === "right" ? "rounded-l-[3rem]" : "rounded-r-[3rem]";
  const translateClass = isOpen ? "translate-x-0" : side === "right" ? "translate-x-full" : "-translate-x-full";
  return `${translateClass} ${roundedClass}`;
}

function SheetHeader({
  title,
  titleId,
  subtitle,
  headerExtra,
  onClose,
  closeLabel,
}: {
  closeLabel: string;
  headerExtra?: React.ReactNode;
  onClose: () => void;
  subtitle?: string;
  title?: string;
  titleId?: string;
}) {
  return (
    <div className={`grid items-start gap-4 border-b border-slate-50 bg-slate-50/30 p-4 dark:border-slate-800 dark:bg-slate-800/30 sm:items-center sm:p-10 ${headerExtra ? "grid-cols-[minmax(0,1fr)_auto] sm:grid-cols-[minmax(0,1fr)_auto_auto]" : "grid-cols-[minmax(0,1fr)_auto]"}`}>
      <div className="row-start-1 flex min-w-0 items-center gap-3 sm:gap-4">
        <div className="flex h-12 w-12 shrink-0 items-center justify-center rounded-2xl border border-primary-100/50 bg-primary-50 text-primary-600 shadow-inner dark:border-primary-800/50 dark:bg-primary-950 dark:text-primary-400">
          <Menu aria-hidden="true" size={24} />
        </div>
        <div className="min-w-0">
          {title ? (
            <Typography className="break-words text-xl font-bold uppercase tracking-tight text-slate-700 dark:text-slate-200" id={titleId} variant="h2">
              {title}
            </Typography>
          ) : null}
          {subtitle ? (
            <Typography className="mt-1 text-[10px] font-bold uppercase tracking-widest text-slate-500 opacity-70 dark:text-slate-400" variant="p">
              {subtitle}
            </Typography>
          ) : null}
        </div>
      </div>
      {headerExtra ? (
        <div className="col-span-2 row-start-2 flex min-w-0 flex-wrap items-center gap-2 sm:col-span-1 sm:col-start-2 sm:row-start-1 sm:justify-self-end">
          {headerExtra}
        </div>
      ) : null}
      <button
        aria-label={closeLabel}
        className={`col-start-2 row-start-1 flex min-h-11 min-w-11 items-center justify-center rounded-xl border border-slate-100 bg-white p-3 text-slate-500 shadow-md transition-all hover:bg-slate-50 hover:text-rose-600 active:scale-90 focus-visible:ring-2 focus-visible:ring-primary-500 focus-visible:ring-offset-2 dark:border-slate-700 dark:bg-slate-800 dark:text-slate-400 dark:hover:bg-slate-700 dark:hover:text-rose-400 ${headerExtra ? "sm:col-start-3" : ""}`}
        onClick={onClose}
        type="button"
      >
        <X aria-hidden="true" className="h-5 w-5" />
      </button>
    </div>
  );
}

export default function Sheet({
  ariaLabelledBy,
  ariaDescribedBy,
  isOpen,
  onClose,
  title,
  subtitle,
  children,
  side = "right",
  className,
  contentClassName,
  headerExtra,
  hideHeader,
}: SheetProps) {
  const generatedTitleId = useId();
  const { t } = useTranslation();
  const resolvedAriaLabelledBy = ariaLabelledBy ?? (title ? generatedTitleId : undefined);
  const isVisible = useSheetState(isOpen, onClose);
  const containerRef = useFocusTrap(isOpen);
  useScrollLock(isOpen);

  if (!isVisible && !isOpen) return null;

  return createPortal(
    <div className="fixed inset-0 z-[1000] flex overflow-hidden">
      {/* Backdrop */}
      {/* eslint-disable-next-line jsx-a11y/no-static-element-interactions, jsx-a11y/click-events-have-key-events -- Custom interactive element (click-away/backdrop/dropzone); keyboard interaction not applicable */}
      <div
        className={cn(
          "fixed inset-0 bg-slate-900/20 backdrop-blur-lg transition-opacity duration-500",
          isOpen ? "opacity-100" : "opacity-0"
        )}
        onClick={onClose}
      />

      {/* Panel */}
      <div className={cn(
        "absolute inset-y-0 flex max-w-full pointer-events-none",
        side === "right" ? "right-0" : "left-0"
 
 
      )}>
        {/* eslint-disable-next-line jsx-a11y/click-events-have-key-events, jsx-a11y/no-noninteractive-element-interactions -- Custom interactive element (click-away/backdrop/dropzone); keyboard interaction not applicable */}
        <div
            aria-describedby={ariaDescribedBy}
            aria-labelledby={resolvedAriaLabelledBy}
            aria-modal="true"
          className={cn(
              "pointer-events-auto w-screen max-w-md transform transition duration-500 ease-in-out bg-white dark:bg-slate-900 shadow-premium-2xl flex flex-col h-full",
              getPanelTransform(side, isOpen),
              className
            )}
          data-testid="sheet-panel"
          onClick={(e) => { e.stopPropagation(); }}
          ref={containerRef}
          role="dialog"
        >
          {/* Header */}
          {!hideHeader && <SheetHeader closeLabel={t("common.close", { defaultValue: "Close" })} headerExtra={headerExtra} onClose={onClose} subtitle={subtitle} title={title} titleId={title ? generatedTitleId : undefined} />}

          {/* Content */}
          <div className={cn("flex-1 overflow-y-auto overscroll-contain p-4 sm:p-10 relative scroll-smooth", contentClassName)}>
            <div className="animate-in fade-in slide-in-from-bottom-4 duration-700 h-full">
              {children}
            </div>
          </div>
        </div>
      </div>
    </div>,
    document.body
  );
}
