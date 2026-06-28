import { Link } from "@tanstack/react-router";
import React from "react";
import { useTranslation } from "react-i18next";

import type { AppDefinition } from "@/lib/app-registry";

import { cn } from "@/lib/utils";

// ── Types ──────────────────────────────────────────────────────────

interface AppCardProps {
  app: AppDefinition;
  onNavigate: (appId: string) => void;
  style?: React.CSSProperties;
}

// ── Component ──────────────────────────────────────────────────────

export default function AppCard({ app, onNavigate, style }: AppCardProps) {
  const { t } = useTranslation();

  return (
    <Link
      aria-label={t(app.labelKey)}
      className={cn(
        "group flex flex-row items-start gap-4 p-4 rounded-2xl transition-all duration-300 h-full",
        "glass-premium",
        "hover:scale-[1.03] hover:shadow-2xl hover:shadow-primary-100/10 dark:hover:shadow-primary-900/10",
        "focus-visible:ring-2 focus-visible:ring-primary-500 focus-visible:ring-offset-2 dark:focus-visible:ring-offset-slate-900",
        "outline-none",
      )}
      data-testid={app.testId}
      onClick={() => { onNavigate(app.id); }}
      style={style}
      to={app.to as "/"}
    >
      {/* Icon */}
      <div
        className={cn(
          "w-10 h-10 rounded-xl flex items-center justify-center shrink-0 transition-all duration-300",
          "bg-primary-50 dark:bg-primary-950",
          "group-hover:bg-primary-100 dark:group-hover:bg-primary-900",
          "group-hover:shadow-md group-hover:shadow-primary-200/50 dark:group-hover:shadow-primary-900/30",
        )}
      >
        <app.icon
          className={cn(
            "h-5 w-5 transition-colors duration-300",
            "text-primary-600 dark:text-primary-400",
            "group-hover:text-primary-700 dark:group-hover:text-primary-300",
          )}
        />
      </div>

      {/* Text */}
      <div className="flex flex-col gap-0.5 min-w-0 flex-1">
        <span className="text-sm font-bold text-slate-700 dark:text-slate-200 truncate leading-snug group-hover:text-primary-600 dark:group-hover:text-primary-400 transition-colors">
          {t(app.labelKey)}
        </span>
        <span className="text-[11px] text-slate-500 dark:text-slate-400 line-clamp-2 leading-relaxed">
          {t(app.descriptionKey, { defaultValue: "" })}
        </span>
      </div>
    </Link>
  );
}
