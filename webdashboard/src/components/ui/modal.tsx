import { X } from "lucide-react";
import React, { useEffect } from "react";
import { createPortal } from "react-dom";

import { Typography } from "@/components/ui/typography";
import { useFocusTrap } from "@/hooks/use-focus-trap";
import { cn } from "@/lib/utils";

interface ModalProps {
  backdropClassName?: string;
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
  backdropClassName,
  children,
  className,
  icon,
  isOpen,
  maxWidth = "md",
  onClose,
  subtitle,
  testId,
  title,
}: ModalProps) {
  const containerRef = useFocusTrap(isOpen);

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
    <div className={cn("modal-backdrop", backdropClassName)} data-testid={testId} onClick={onClose} onKeyDown={(e) => { if (e.key === 'Escape') onClose(); }} role="presentation">
      {/* eslint-disable-next-line jsx-a11y/click-events-have-key-events, jsx-a11y/no-noninteractive-element-interactions -- Element has interactive role via onClick/onKeyDown */}
      <div 
        aria-modal="true"
        className={cn(
          "modal-content", 
          maxWidthMap[maxWidth],
          className
        )}
        onClick={(e) => { e.stopPropagation(); }} 
        ref={containerRef}
        role="dialog"
        style={{ maxHeight: '90vh', display: 'flex', flexDirection: 'column' }}
      >
        {/* Header — fixed at top, never shrinks */}
        {Boolean(title ?? icon) && (
          <div className="px-8 pt-8 pb-6 border-b border-slate-50 dark:border-slate-800 flex items-center justify-between shrink-0 bg-white/80 dark:bg-slate-900/80 backdrop-blur-md rounded-t-[3rem]">
            <div className="flex items-center gap-4 min-w-0">
              {Boolean(icon) && (
                <div className="h-10 w-10 rounded-2xl bg-primary-50 dark:bg-primary-950 flex items-center justify-center text-primary-600 dark:text-primary-400 shrink-0 border border-primary-100 dark:border-primary-800 shadow-sm">
                  {icon}
                </div>
              )}
              <div className="min-w-0">
                {Boolean(title) && (
                  <Typography className="text-xl font-bold text-slate-800 dark:text-slate-100 tracking-tight truncate leading-normal pt-1 uppercase" variant="h2">
                    {title}
                  </Typography>
                )}
                {Boolean(subtitle) && (
                  <Typography className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest opacity-70 truncate" variant="p">
                    {subtitle}
                  </Typography>
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
