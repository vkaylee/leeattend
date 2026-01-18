import React, { useEffect, useState } from "react";
import { createPortal } from "react-dom";
import { X } from "lucide-react";
import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

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
      const timer = setTimeout(() => setIsVisible(false), 300); // Wait for animation
      document.body.style.overflow = "unset";
      return () => clearTimeout(timer);
    }
  }, [isOpen]);

  if (!isVisible && !isOpen) return null;

  return createPortal(
    <div className="fixed inset-0 z-50 flex overflow-hidden">
      {/* Backdrop */}
      <div 
        className={cn(
          "fixed inset-0 bg-black/20 backdrop-blur-sm transition-opacity duration-300",
          isOpen ? "opacity-100" : "opacity-0"
        )}
        onClick={onClose}
      />

      {/* Panel */}
      <div className={cn(
        "absolute inset-y-0 flex max-w-full pointer-events-none",
        side === "right" ? "right-0 pl-10" : "left-0 pr-10"
      )}>
        <div 
          data-testid="sheet-panel"
          className={cn(
            "pointer-events-auto w-screen max-w-md transform transition duration-300 ease-in-out bg-white shadow-2xl flex flex-col h-full",
            side === "right" 
              ? (isOpen ? "translate-x-0" : "translate-x-full")
              : (isOpen ? "translate-x-0" : "-translate-x-full"),
            className
          )}
          onClick={(e) => e.stopPropagation()}
        >
           {/* Header */}
          <div className="flex items-center justify-between p-6 border-b border-slate-100">
            <div>
              {title && (
                <h2 className="text-lg font-bold text-slate-900 tracking-tight">
                  {title}
                </h2>
              )}
              {subtitle && (
                <p className="text-[10px] text-slate-400 font-bold uppercase tracking-widest mt-0.5">
                  {subtitle}
                </p>
              )}
            </div>
            <button
              onClick={onClose}
              className="p-2 hover:bg-slate-100 rounded-full transition-colors text-slate-400 hover:text-slate-600"
            >
              <X className="w-5 h-5" />
            </button>
          </div>

          {/* Content */}
          <div className="flex-1 overflow-y-auto p-6 relative">
            {children}
          </div>
        </div>
      </div>
    </div>,
    document.body
  );
}
