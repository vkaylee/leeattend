import type { KeyboardEvent, ReactNode } from "react";

import { ChevronRight } from "lucide-react";

import { cn } from "@/lib/utils";

export interface SettingsRowProps {
  className?: string;
  control?: ReactNode;
  description?: ReactNode;
  destructive?: boolean;
  disabled?: boolean;
  disclosure?: boolean;
  icon?: ReactNode;
  iconBg?: string;
  label: ReactNode;
  onClick?: () => void;
  testId?: string;
}

interface SettingsRowIconProps {
  background?: string;
  children: ReactNode;
}

interface SettingsRowDetailsProps {
  description?: ReactNode;
  destructive: boolean;
  label: ReactNode;
}

interface SettingsRowActionsProps {
  control?: ReactNode;
  disclosure: boolean;
}

function handleRowKeyDown(event: KeyboardEvent<HTMLButtonElement>, onClick: () => void) {
  if (event.key !== "Enter" && event.key !== " ") return;

  event.preventDefault();
  onClick();
}

function SettingsRowIcon({ background, children }: SettingsRowIconProps) {
  return (
    <div
      className={cn(
        "flex h-7 w-7 shrink-0 items-center justify-center rounded-lg text-primary-600 dark:text-primary-400",
        background ?? "bg-primary-50 dark:bg-primary-950/70",
      )}
    >
      {children}
    </div>
  );
}

function SettingsRowDetails({ destructive, description, label }: SettingsRowDetailsProps) {
  return (
    <div className="min-w-0 flex-1">
      <div
        className={cn(
          "truncate font-medium leading-snug",
          destructive
            ? "font-semibold text-rose-600 dark:text-rose-400"
            : "text-slate-800 dark:text-slate-100",
        )}
      >
        {label}
      </div>
      {description ? (
        <div className="mt-0.5 text-[11px] leading-normal text-slate-400 dark:text-slate-500">
          {description}
        </div>
      ) : null}
    </div>
  );
}

function SettingsRowActions({ control, disclosure }: SettingsRowActionsProps) {
  if (!control && !disclosure) return null;

  return (
    <div className="flex shrink-0 items-center gap-2">
      {control}
      {disclosure ? <ChevronRight className="h-4 w-4 text-slate-400 dark:text-slate-500" /> : null}
    </div>
  );
}

interface SettingsRowSurfaceProps extends SettingsRowProps {
  isClickable: boolean;
}

function SettingsRowFields({
  control,
  description,
  destructive,
  disclosure,
  icon,
  iconBg,
  label,
}: SettingsRowProps) {
  return (
    <>
      <div className="flex min-w-0 flex-1 items-center gap-3.5">
        {icon ? <SettingsRowIcon background={iconBg}>{icon}</SettingsRowIcon> : null}
        <SettingsRowDetails description={description} destructive={destructive ?? false} label={label} />
      </div>
      <SettingsRowActions control={control} disclosure={disclosure ?? false} />
    </>
  );
}

function getSurfaceClassName(isClickable: boolean, disabled: boolean, className?: string) {
  return cn(
    "flex min-h-[50px] items-center justify-between gap-4 px-4 py-3 text-xs transition-colors",
    isClickable &&
      "cursor-pointer hover:bg-slate-50/80 active:bg-slate-100/80 dark:hover:bg-slate-800/50 dark:active:bg-slate-800/80",
    disabled && "pointer-events-none opacity-50",
    className,
  );
}

function SettingsRowSurface({
  className,
  disabled,
  isClickable,
  onClick,
  testId,
  ...fields
}: SettingsRowSurfaceProps) {
  const surfaceClassName = getSurfaceClassName(isClickable, disabled ?? false, className);

  if (isClickable && onClick) {
    return (
      <button
        className={cn(surfaceClassName, "border-0 bg-transparent text-left")}
        data-testid={testId}
        onClick={onClick}
        onKeyDown={(event) => {
          handleRowKeyDown(event, onClick);
        }}
        type="button"
      >
        <SettingsRowFields {...fields} />
      </button>
    );
  }

  return (
    <div className={surfaceClassName} data-testid={testId}>
      <SettingsRowFields {...fields} />
    </div>
  );
}

export function SettingsRow({ disabled = false, onClick, ...props }: SettingsRowProps) {
  return <SettingsRowSurface {...props} disabled={disabled} isClickable={Boolean(onClick) && !disabled} onClick={onClick} />;
}

export default SettingsRow;
