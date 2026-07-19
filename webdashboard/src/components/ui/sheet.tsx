import { X, Menu } from "lucide-react";
import React, { useEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";

import { Typography } from "@/components/ui/typography";
import { useFocusTrap } from "@/hooks/use-focus-trap";
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
    }, 500);
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

function getPanelTransform(side: "left" | "right", isOpen: boolean): string {
  const roundedClass = side === "right" ? "rounded-l-[3rem]" : "rounded-r-[3rem]";
  const translateClass = isOpen ? "translate-x-0" : side === "right" ? "translate-x-full" : "-translate-x-full";
  return `${translateClass} ${roundedClass}`;
}

function SheetHeader({
  title,
  subtitle,
  headerExtra,
  onClose
}: {
  headerExtra?: React.ReactNode;
  onClose: () => void;
  subtitle?: string;
  title?: string;
}) {
  return (
    <div className="flex items-center justify-between p-10 border-b border-slate-50 dark:border-slate-800 bg-slate-50/30 dark:bg-slate-800/30">
      <div className="flex items-center gap-4">
        <div className="w-12 h-12 rounded-2xl bg-primary-50 dark:bg-primary-950 flex items-center justify-center text-primary-600 dark:text-primary-400 shadow-inner border border-primary-100/50 dark:border-primary-800/50">
          <Menu size={24} />
        </div>
        <div>
          {title ? (
            <Typography className="text-xl font-bold text-slate-700 dark:text-slate-200 tracking-tight uppercase" variant="h2">
              {title}
            </Typography>
          ) : null}
          {subtitle ? (
            <Typography className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest mt-1 opacity-70" variant="p">
              {subtitle}
            </Typography>
          ) : null}
        </div>
      </div>
      {headerExtra ? (
        <div className="flex items-center gap-2">
          {headerExtra}
        </div>
      ) : null}
      <button
        className="p-3 bg-white dark:bg-slate-800 hover:bg-slate-50 dark:hover:bg-slate-700 rounded-xl transition-all shadow-md border border-slate-100 dark:border-slate-700 text-slate-500 dark:text-slate-400 hover:text-rose-600 dark:hover:text-rose-400 active:scale-90"
        onClick={onClose}
      >
        <X className="w-5 h-5" />
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
  const isVisible = useSheetState(isOpen, onClose);
  const containerRef = useFocusTrap(isOpen);

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
            aria-labelledby={ariaLabelledBy}
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
          {!hideHeader && <SheetHeader headerExtra={headerExtra} onClose={onClose} subtitle={subtitle} title={title} />}

          {/* Content */}
          <div className={cn("flex-1 overflow-y-auto p-10 relative scroll-smooth", contentClassName)}>
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
