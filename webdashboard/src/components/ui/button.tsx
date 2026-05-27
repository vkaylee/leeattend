import { Loader2 } from "lucide-react";
import React, { memo, forwardRef } from "react";

import { cn } from "@/lib/utils";

// ── Variant styles ─────────────────────────────────────────────────

const VARIANT_STYLES = {
  primary:
    "bg-primary-600 text-white hover:bg-primary-700 shadow-xl shadow-primary-100 dark:shadow-black/30",
  secondary:
    "bg-slate-900 text-white hover:bg-primary-700 shadow-xl shadow-slate-200 dark:shadow-black/30",
  outline:
    "bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 text-slate-700 dark:text-slate-200 hover:bg-slate-50 dark:hover:bg-slate-800 shadow-sm",
  ghost:
    "hover:bg-slate-100 dark:hover:bg-slate-700 text-slate-600 dark:text-slate-300",
  destructive:
    "bg-rose-600 text-white hover:bg-rose-700 shadow-sm",
} as const;

const SIZE_STYLES = {
  sm: "h-[var(--ui-height-sm)] px-4 text-[10px] rounded-xl gap-2",
  md: "h-[var(--ui-height-md)] px-8 text-xs rounded-2xl gap-3",
  lg: "h-[var(--ui-height-lg)] px-10 text-xs rounded-2xl gap-4",
  icon: "h-[var(--ui-height-md)] w-[var(--ui-height-md)] rounded-xl gap-0 p-0",
} as const;

// ── Component ──────────────────────────────────────────────────────

export type ButtonVariant = keyof typeof VARIANT_STYLES;
export type ButtonSize = keyof typeof SIZE_STYLES;

export interface ButtonProps
  extends Omit<React.ButtonHTMLAttributes<HTMLButtonElement>, "children"> {
  children?: React.ReactNode;
  icon?: React.ReactNode;
  loading?: boolean;
  size?: ButtonSize;
  variant?: ButtonVariant;
}

 
const Button = memo(
 
  forwardRef<HTMLButtonElement, ButtonProps>(function Button(
    {
      variant = "primary",
      size = "md",
      icon,
      loading = false,
      disabled,
      className,
      children,
      type = "button",
      ...rest
    },
    ref,
  ) {
    return (
      <button
        className={cn(
          "inline-flex items-center justify-center font-bold uppercase tracking-widest transition-all active:scale-95 disabled:opacity-50 disabled:pointer-events-none",
          VARIANT_STYLES[variant],
          SIZE_STYLES[size],
          className,
        )}
        disabled={disabled ?? loading}
        ref={ref}
        type={type}
        {...rest}
      >
        {loading ? <Loader2 className="animate-spin" size={18} /> : icon}
        {children}
      </button>
    );
  }),
);

export default Button;
