import React, { useEffect } from "react";
import { createPortal } from "react-dom";
import { X } from "lucide-react";
import { cn } from "@/lib/utils";

interface ModalProps {
  isOpen: boolean;
  onClose: () => void;
  title?: string;
  subtitle?: string;
  icon?: React.ReactNode;
  children: React.ReactNode;
  maxWidth?: "sm" | "md" | "lg" | "xl" | "2xl" | "3xl" | "4xl";
  className?: string;
  testId?: string;
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
    <div className="modal-backdrop" onClick={onClose} data-testid={testId}>
      <div 
        role="dialog"
        aria-modal="true"
        className={cn(
          "modal-content", 
          maxWidthMap[maxWidth],
          className
        )} 
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        {(title || icon) && (
          <div className="flex items-center justify-between p-8 border-b border-slate-50">
            <div className="flex items-center gap-4">
              {icon && (
                <div className="bg-primary-50 p-3 rounded-2xl text-primary-600">
                  {icon}
                </div>
              )}
              <div>
                {title && (
                  <h2 className="text-xl font-bold text-primary-600 tracking-tight">
                    {title}
                  </h2>
                )}
                {subtitle && (
                  <p className="text-[10px] text-primary-400 font-bold uppercase tracking-widest mt-0.5">
                    {subtitle}
                  </p>
                )}
              </div>
            </div>
            <button
              onClick={onClose}
              className="p-2 hover:bg-slate-100 rounded-full transition-colors group"
            >
              <X className="w-5 h-5 text-slate-300 group-hover:text-primary-700" />
            </button>
          </div>
        )}

        <div className="p-8">
          {children}
        </div>
      </div>
    </div>,
    document.body
  );
}
