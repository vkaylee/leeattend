import React, { useEffect, useState } from "react";
import { createPortal } from "react-dom";
import { X, Menu } from "lucide-react";
import { cn } from "@/lib/utils";

interface SheetProps {
  isOpen: boolean;
  onClose: () => void;
  title?: string;
  subtitle?: string;
  children: React.ReactNode;
  side?: "left" | "right";
  className?: string;
}

export default function Sheet({
  isOpen,
  onClose,
  title,
  subtitle,
  children,
  side = "right",
  className,
}: SheetProps) {
  const [isVisible, setIsVisible] = useState(false);

  useEffect(() => {
    if (isOpen) {
      // eslint-disable-next-line react-hooks/set-state-in-effect
      setIsVisible(true);
      document.body.style.overflow = "hidden";
    } else {
      const timer = setTimeout(() => setIsVisible(false), 500); // Sync with duration-500
      document.body.style.overflow = "unset";
      return () => clearTimeout(timer);
    }
  }, [isOpen]);

  if (!isVisible && !isOpen) return null;

  return createPortal(
    <div className="fixed inset-0 z-[1000] flex overflow-hidden">
      {/* Backdrop */}
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
        <div 
          data-testid="sheet-panel"
          className={cn(
            "pointer-events-auto w-screen max-w-md transform transition duration-500 ease-in-out bg-white dark:bg-slate-900 shadow-premium-2xl flex flex-col h-full",
            side === "right" 
              ? (isOpen ? "translate-x-0 rounded-l-[3rem]" : "translate-x-full rounded-l-[3rem]")
              : (isOpen ? "translate-x-0 rounded-r-[3rem]" : "-translate-x-full rounded-r-[3rem]"),
            className
          )}
          onClick={(e) => e.stopPropagation()}
        >
           {/* Header */}
          <div className="flex items-center justify-between p-10 border-b border-slate-50 dark:border-slate-800 bg-slate-50/30 dark:bg-slate-800/30">
            <div className="flex items-center gap-4">
              <div className="w-12 h-12 rounded-2xl bg-primary-50 dark:bg-primary-950 flex items-center justify-center text-primary-600 dark:text-primary-400 shadow-inner border border-primary-100/50 dark:border-primary-800/50">
                <Menu size={24} />
              </div>
              <div>
                {title && (
                  <h2 className="text-xl font-bold text-slate-700 dark:text-slate-200 tracking-tight uppercase">
                    {title}
                  </h2>
                )}
                {subtitle && (
                  <p className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest mt-1 opacity-70">
                    {subtitle}
                  </p>
                )}
              </div>
            </div>
            <button
              onClick={onClose}
              className="p-3 bg-white dark:bg-slate-800 hover:bg-slate-50 dark:hover:bg-slate-700 rounded-xl transition-all shadow-md border border-slate-100 dark:border-slate-700 text-slate-500 dark:text-slate-400 hover:text-rose-600 dark:hover:text-rose-400 active:scale-90"
            >
              <X className="w-5 h-5" />
            </button>
          </div>

          {/* Content */}
          <div className="flex-1 overflow-y-auto p-10 relative scroll-smooth">
            <div className="animate-in fade-in slide-in-from-bottom-4 duration-700">
              {children}
            </div>
          </div>
        </div>
      </div>
    </div>,
    document.body
  );
}
