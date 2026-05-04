import { X } from "lucide-react";
import React, { useEffect } from "react";
import { createPortal } from "react-dom";

import { cn } from "@/lib/utils";

interface ModalProps {
  children: React.ReactNode;
  className?: string;
  icon?: React.ReactNode;
  isOpen: boolean;
  maxWidth?: "sm" | "md" | "lg" | "xl" | "2xl" | "3xl" | "4xl";
  onClose: () => void;
  subtitle?: string;
  testId?: string;
  title?: string;
}

const maxWidthMap = {
  sm: "max-w-sm",
  md: "max-w-md",
  lg: "max-w-lg",
  xl: "max-w-xl",
  "2xl": "max-w-2xl",
  "3xl": "max-w-3xl",
  "4xl": "max-w-4xl",
};

export default function Modal({
  isOpen,
  onClose,
  title,
  subtitle,
  icon,
  children,
  maxWidth = "md",
  className,
  testId,
}: ModalProps) {
  // Prevent scrolling when modal is open
  useEffect(() => {
    if (isOpen) {
      document.body.style.overflow = "hidden";
    } else {
      document.body.style.overflow = "unset";
    }
    return () => {
      document.body.style.overflow = "unset";
    };
  }, [isOpen]);

  if (!isOpen) return null;

  return createPortal(
    <div className="modal-backdrop" data-testid={testId} onClick={onClose} onKeyDown={(e) => { if (e.key === 'Escape') onClose(); }} role="presentation">
      {/* eslint-disable-next-line jsx-a11y/click-events-have-key-events, jsx-a11y/no-noninteractive-element-interactions -- Element has interactive role via onClick/onKeyDown */}
      <div 
        aria-modal="true"
        className={cn(
          "modal-content", 
          maxWidthMap[maxWidth],
          className
        )}
        onClick={(e) => { e.stopPropagation(); }} 
        role="dialog"
        style={{ maxHeight: '90vh', display: 'flex', flexDirection: 'column' }}
      >
        {/* Header — fixed at top, never shrinks */}
        {Boolean(title ?? icon) && (
          <div className="px-8 pt-8 pb-6 border-b border-slate-50 dark:border-slate-800 flex items-center justify-between shrink-0 bg-white/80 dark:bg-slate-900/80 backdrop-blur-md rounded-t-[2rem]">
            <div className="flex items-center gap-4 min-w-0">
              {Boolean(icon) && (
                <div className="h-10 w-10 rounded-2xl bg-primary-50 dark:bg-primary-950 flex items-center justify-center text-primary-600 dark:text-primary-400 shrink-0 border border-primary-100 dark:border-primary-800 shadow-sm">
                  {icon}
                </div>
              )}
              <div className="min-w-0">
                {Boolean(title) && (
                  <h2 className="text-xl font-bold text-slate-800 dark:text-slate-100 tracking-tight truncate leading-normal pt-1 uppercase">
                    {title}
                  </h2>
                )}
                {Boolean(subtitle) && (
                  <p className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest opacity-70 truncate">
                    {subtitle}
                  </p>
                )}
              </div>
            </div>
            <button
              className="p-2 hover:bg-slate-100 dark:hover:bg-slate-800 rounded-full transition-colors group"
              onClick={onClose}
            >
              <X className="w-5 h-5 text-slate-300 dark:text-slate-500 group-hover:text-primary-700 dark:group-hover:text-primary-400" />
            </button>
          </div>
        )}

        {/* Body — scrollable area */}
        <div className="p-8" style={{ overflowY: 'auto', flex: 1, minHeight: 0 }}>
          {children}
        </div>
      </div>
    </div>,
    document.body
  );
}
