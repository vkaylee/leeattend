import type React from "react";

import { ChevronDown } from "lucide-react";
import { forwardRef } from "react";

import type { ControlSize } from "@/components/ui/control-styles";

import Button from "@/components/ui/button";
import { cn } from "@/lib/utils";

export type FilterTriggerVariant = "pill-button" | "icon-button";

export interface FilterTriggerProps extends Omit<React.ButtonHTMLAttributes<HTMLButtonElement>, "children"> {
  activeCount?: number;
  chevron?: boolean;
  children?: React.ReactNode;
  hasActive?: boolean;
  icon?: React.ReactNode;
  label: string;
  size?: ControlSize;
  variant: FilterTriggerVariant;
}

function CountBadge({ count, variant }: { count: number; variant: FilterTriggerVariant }) {
  if (count <= 0) return null;
  const className = variant === "icon-button"
    ? "absolute -top-1 -right-1 min-w-[14px] h-[14px] px-0.5 flex items-center justify-center bg-primary-500 text-white text-[8px] font-bold rounded-full leading-none animate-in zoom-in"
    : "inline-flex h-5 min-w-5 items-center justify-center rounded-full bg-primary-600 dark:bg-primary-500 px-1.5 text-[10px] text-white animate-in zoom-in";
  return <span className={className} key={count}>{count}</span>;
}

type IconButtonTriggerProps = {
  activeCount?: number;
  className?: string;
  hasActive?: boolean;
  icon?: React.ReactNode;
  label: string;
} & Omit<React.ButtonHTMLAttributes<HTMLButtonElement>, "children"> & { ref: React.Ref<HTMLButtonElement> };

function IconButtonTrigger({ label, icon, hasActive, activeCount = 0, className, ref, ...restProps }: IconButtonTriggerProps) {
  return (
    <Button aria-label={label} className={cn("relative h-5 w-5 rounded transition-colors p-0 border-none shadow-none hover:bg-transparent", hasActive ? "text-primary-600 dark:text-primary-400" : "text-slate-400 dark:text-slate-500 opacity-60 hover:opacity-100 hover:text-primary-600", className)} ref={ref} title={label} type="button" variant="ghost" {...restProps}>
      {icon ?? null}
      <CountBadge count={activeCount} variant="icon-button" />
    </Button>
  );
}

type PillButtonProps = Omit<FilterTriggerProps, "variant"> & { ref: React.Ref<HTMLButtonElement> };

function PillButton({ activeCount = 0, children, chevron, className, hasActive, icon, label, ref, size, ...restProps }: PillButtonProps) {
  const showBadge = activeCount > 0;
  return (
    <Button aria-label={label} className={cn("relative flex items-center gap-2 border shadow-inner transition-all", hasActive || showBadge ? "border-primary-500 dark:border-primary-500 bg-primary-50 dark:bg-primary-950 text-primary-600 dark:text-primary-400" : "border-slate-100 dark:border-slate-700 bg-slate-50 dark:bg-slate-800 text-slate-500 dark:text-slate-400 hover:border-primary-200 dark:hover:border-primary-800", size === "sm" ? "rounded-xl" : "rounded-2xl", className)} ref={ref} size={size} type="button" variant="ghost" {...restProps}>
      {children ?? <>{icon ?? null}<span className="truncate">{label}</span></>}
      <CountBadge count={activeCount} variant="pill-button" />
      {chevron ? <ChevronDown className="h-3.5 w-3.5 shrink-0 opacity-50" /> : null}
    </Button>
  );
}

const FilterTrigger = forwardRef<HTMLButtonElement, FilterTriggerProps>(function FilterTrigger({ variant, label, icon, activeCount = 0, hasActive = false, chevron = false, className, children, size = "md", ...restProps }, ref) {
  if (variant === "icon-button") return <IconButtonTrigger activeCount={activeCount} className={className} hasActive={hasActive} icon={icon} label={label} ref={ref} {...restProps} />;
  return <PillButton activeCount={activeCount} chevron={chevron} className={className} hasActive={hasActive} icon={icon} label={label} ref={ref} size={size} {...restProps}>{children}</PillButton>;
});

FilterTrigger.displayName = "FilterTrigger";

export { FilterTrigger };
export default FilterTrigger;
