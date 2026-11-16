import { Loader2 } from "lucide-react";
import React, { memo, forwardRef } from "react";

import { cn } from "@/lib/utils";

import {
  CONTROL_FOCUS_RING,
  CONTROL_SIZE_STYLES,
  CONTROL_TEXT_STYLE,
} from "./control-styles";

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
  sm: cn(CONTROL_SIZE_STYLES.sm, "min-w-[var(--ui-height-sm)]"),
  md: cn(CONTROL_SIZE_STYLES.md, "min-w-[var(--ui-height-md)] px-8"),
  lg: cn(CONTROL_SIZE_STYLES.lg, "min-w-[var(--ui-height-lg)] px-10"),
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
          "inline-flex items-center justify-center transition-all active:scale-95 disabled:bg-slate-200 disabled:text-slate-600 disabled:shadow-none disabled:cursor-not-allowed dark:disabled:bg-slate-800 dark:disabled:text-slate-400",
          CONTROL_FOCUS_RING,
          VARIANT_STYLES[variant],
          SIZE_STYLES[size],
          CONTROL_TEXT_STYLE,
          className,
        )}
        data-component="Button"
        disabled={disabled ? true : loading}
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
