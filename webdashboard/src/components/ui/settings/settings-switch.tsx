import React from "react";

import { cn } from "@/lib/utils";

export interface SettingsSwitchProps {
  ariaLabel?: string;
  checked: boolean;
  className?: string;
  disabled?: boolean;
  id?: string;
  onCheckedChange: (checked: boolean) => void;
  testId?: string;
}

export function SettingsSwitch({
  ariaLabel,
  checked,
  className,
  disabled = false,
  id,
  onCheckedChange,
  testId,
}: SettingsSwitchProps) {
  return (
    <button
      aria-checked={checked}
      aria-label={ariaLabel}
      className={cn(
        "relative inline-flex h-6 w-11 shrink-0 cursor-pointer rounded-full p-[2px] transition-colors duration-200 ease-in-out",
        "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary-500 focus-visible:ring-offset-2 dark:focus-visible:ring-offset-slate-900",
        checked
          ? "bg-[#34C759] dark:bg-[#30D158]"
          : "bg-slate-200 dark:bg-slate-700",
        disabled && "opacity-50 pointer-events-none cursor-not-allowed",
        className
      )}
      data-testid={testId}
      disabled={disabled}
      id={id}
      onClick={() => {
        if (!disabled) {
          onCheckedChange(!checked);
        }
      }}
      role="switch"
      type="button"
    >
      <span
        className={cn(
          "pointer-events-none inline-block h-5 w-5 rounded-full bg-white shadow-xs ring-0 transition-transform duration-200 ease-in-out",
          checked ? "translate-x-5" : "translate-x-0"
        )}
      />
    </button>
  );
}

export default SettingsSwitch;
