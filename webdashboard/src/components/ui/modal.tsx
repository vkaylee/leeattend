import * as Dialog from "@radix-ui/react-dialog";
import { X } from "lucide-react";
import { memo, useCallback } from "react";

import { Typography } from "@/components/ui/typography";
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

const Modal = memo(function Modal({
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
  const handleOpenChange = useCallback((open: boolean) => {
    if (!open) onClose();
  }, [onClose]);

  return (
    <Dialog.Root onOpenChange={handleOpenChange} open={isOpen}>
      <Dialog.Portal>
        <Dialog.Overlay
          className={cn(
            "modal-backdrop",
            "fixed inset-0 z-[9999] bg-slate-900/20 backdrop-blur-lg animate-in fade-in duration-500",
            backdropClassName
          )}
          onClick={onClose}
        />
        <Dialog.Content
          className={cn(
            "modal-content",
            "fixed top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 z-[10000]",
            "bg-white dark:bg-slate-900 w-full rounded-[3rem] shadow-premium-2xl border border-white/60 dark:border-slate-700/60 animate-in zoom-in-95 duration-500 overflow-hidden",
            "max-h-[90dvh] flex flex-col",
            maxWidthMap[maxWidth],
            className
          )}
          data-testid={testId}
        >
          {!title && <Dialog.Title className="sr-only">Dialog</Dialog.Title>}
          {!subtitle && <Dialog.Description className="sr-only">{title ? `${title} description` : "Dialog description"}</Dialog.Description>}

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
                  {title ? (
                    <Dialog.Title asChild>
                      <Typography className="text-xl font-bold text-slate-800 dark:text-slate-100 tracking-tight truncate leading-normal pt-1 uppercase" variant="h2">
                        {title}
                      </Typography>
                    </Dialog.Title>
                  ) : null}
                  {subtitle ? (
                    <Dialog.Description asChild>
                      <Typography className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest opacity-70 truncate" variant="p">
                        {subtitle}
                      </Typography>
                    </Dialog.Description>
                  ) : null}
                </div>
              </div>
              <Dialog.Close asChild>
                <button
                  aria-label="Close dialog"
                  className="p-2 hover:bg-slate-100 dark:hover:bg-slate-800 rounded-full transition-colors group"
                >
                  <X className="w-5 h-5 text-slate-300 dark:text-slate-500 group-hover:text-primary-700 dark:group-hover:text-primary-400" />
                </button>
              </Dialog.Close>
            </div>
          )}

          {/* Body — scrollable area */}
          <div className="p-8 overflow-y-auto overscroll-contain flex-1 min-h-0">
            {children}
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
});

Modal.displayName = "Modal";

export default Modal;
