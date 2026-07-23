import { AlertTriangle, Clock, Gauge } from "lucide-react";
import React, { memo } from "react";

import { cn } from "@/lib/utils";

export type ComplianceBadgeType = "overlap" | "rest" | "hours";

interface ComplianceAlertBadgeProps {
  type: ComplianceBadgeType;
  message: string;
  /** badge = standalone pill, inline = compact text+icon */
  variant?: "badge" | "inline";
  className?: string;
}

const TYPE_STYLES: Record<
  ComplianceBadgeType,
  {
    icon: React.ComponentType<{ size?: number; className?: string }>;
    classes: string;
    animation?: string;
  }
> = {
  overlap: {
    icon: AlertTriangle,
    classes:
      "bg-rose-50 text-rose-700 border-rose-200 dark:bg-rose-950/50 dark:text-rose-300 dark:border-rose-800/50",
    animation: "animate-pulse",
  },
  rest: {
    icon: Clock,
    classes:
      "bg-amber-50 text-amber-700 border-amber-200 dark:bg-amber-950/50 dark:text-amber-300 dark:border-amber-800/50",
  },
  hours: {
    icon: Gauge,
    classes:
      "bg-sky-50 text-sky-700 border-sky-200 dark:bg-sky-950/50 dark:text-sky-300 dark:border-sky-800/50",
  },
};

const TYPE_LABELS: Record<ComplianceBadgeType, string> = {
  overlap: "overlap",
  rest: "rest",
  hours: "hours",
};

export const ComplianceAlertBadge = memo(function ComplianceAlertBadge({
  type,
  message,
  variant = "badge",
  className,
}: ComplianceAlertBadgeProps) {
  const { icon: Icon, classes, animation } = TYPE_STYLES[type];

  if (variant === "inline") {
    return (
      <div
        aria-label={`${TYPE_LABELS[type]}: ${message}`}
        className={cn(
          "inline-flex items-center gap-1 text-[10px] font-semibold",
          classes.replace(/bg-\S+/, "").replace(/border-\S+/, ""),
          animation,
          className,
        )}
        role="alert"
      >
        <Icon className="flex-shrink-0" size={12} />
        <span>{message}</span>
      </div>
    );
  }

  // badge variant — pill with border and background
  return (
    <div
      aria-label={`${TYPE_LABELS[type]}: ${message}`}
      className={cn(
        "inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full border text-[9px] font-bold uppercase tracking-wider shadow-sm transition-all",
        classes,
        animation,
        className,
      )}
      role="alert"
    >
      <Icon className="flex-shrink-0" size={11} />
      <span className="truncate max-w-[14rem]">{message}</span>
    </div>
  );
});

ComplianceAlertBadge.displayName = "ComplianceAlertBadge";
