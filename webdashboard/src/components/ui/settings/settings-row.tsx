import { ChevronRight } from "lucide-react";
import React from "react";

import { cn } from "@/lib/utils";

export interface SettingsRowProps {
  className?: string;
  control?: React.ReactNode;
  description?: React.ReactNode;
  destructive?: boolean;
  disabled?: boolean;
  disclosure?: boolean;
  icon?: React.ReactNode;
  iconBg?: string;
  label: React.ReactNode;
  onClick?: () => void;
  testId?: string;
}

export function SettingsRow({
  className,
  control,
  description,
  destructive = false,
  disabled = false,
  disclosure = false,
  icon,
  iconBg,
  label,
  onClick,
  testId,
}: SettingsRowProps) {
  const isClickable = Boolean(onClick) && !disabled;

  const content = (
    <div
      className={cn(
        "flex min-h-[50px] items-center justify-between gap-4 px-4 py-3 text-xs transition-colors",
        isClickable &&
          "cursor-pointer hover:bg-slate-50/80 active:bg-slate-100/80 dark:hover:bg-slate-800/50 dark:active:bg-slate-800/80",
        disabled && "opacity-50 pointer-events-none",
        className
      )}
      data-testid={testId}
      onClick={isClickable ? onClick : undefined}
      onKeyDown={
        isClickable
          ? (e) => {
              if (e.key === "Enter" || e.key === " ") {
                e.preventDefault();
                onClick?.();
              }
            }
          : undefined
      }
      role={isClickable ? "button" : undefined}
      tabIndex={isClickable ? 0 : undefined}
    >
      <div className="flex min-w-0 items-center gap-3.5 flex-1">
        {icon && (
          <div
            className={cn(
              "flex h-7 w-7 shrink-0 items-center justify-center rounded-lg text-primary-600 dark:text-primary-400",
              iconBg ?? "bg-primary-50 dark:bg-primary-950/70"
            )}
          >
            {icon}
          </div>
        )}
        <div className="min-w-0 flex-1">
          <div
            className={cn(
              "font-medium leading-snug truncate",
              destructive
                ? "text-rose-600 dark:text-rose-400 font-semibold"
                : "text-slate-800 dark:text-slate-100"
            )}
          >
            {label}
          </div>
          {description && (
            <div className="mt-0.5 text-[11px] leading-normal text-slate-400 dark:text-slate-500">
              {description}
            </div>
          )}
        </div>
      </div>

      {(control || disclosure) && (
        <div className="flex shrink-0 items-center gap-2">
          {control}
          {disclosure && (
            <ChevronRight className="h-4 w-4 text-slate-400 dark:text-slate-500" />
          )}
        </div>
      )}
    </div>
  );

  return content;
}

export default SettingsRow;
