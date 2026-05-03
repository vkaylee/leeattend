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
        {!!(title ?? icon) && (
          <div className="flex items-center justify-between p-8 border-b border-slate-50 dark:border-slate-800" style={{ flexShrink: 0 }}>
            <div className="flex items-center gap-4">
              {!!icon && (
                <div className="bg-primary-50 dark:bg-primary-900/30 p-3 rounded-2xl text-primary-600 dark:text-primary-400">
                  {icon}
                </div>
              )}
              <div>
                {!!title && (
                  <h2 className="text-xl font-bold text-primary-600 tracking-tight">
                    {title}
                  </h2>
                )}
                {!!subtitle && (
                  <p className="text-[10px] text-primary-400 font-bold uppercase tracking-widest mt-0.5">
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
