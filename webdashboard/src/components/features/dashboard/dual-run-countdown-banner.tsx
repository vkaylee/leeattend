import { AlertTriangle, CheckCircle, Clock, Timer } from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";

import { Typography } from "@/components/ui/typography";
import { cn } from "@/lib/utils";
import { useCascadeDualRun } from "@/hooks/use-cascade-dual-run";

// ── Phase config ────────────────────────────────────────────────────────

interface PhaseConfig {
  icon: React.ComponentType<{ className?: string }>;
  containerClass: string;
  iconClass: string;
  textClass: string;
  labelKey: string;
}

const PHASE_MAP: Record<string, PhaseConfig> = {
  active: {
    icon: Clock,
    containerClass: "bg-blue-50/80 dark:bg-blue-950/50 border-blue-200 dark:border-blue-800",
    iconClass: "text-blue-600 dark:text-blue-400",
    textClass: "text-blue-700 dark:text-blue-300",
    labelKey: "dual_run.status_active",
  },
  closing_soon: {
    icon: Timer,
    containerClass: "bg-amber-50/80 dark:bg-amber-950/50 border-amber-200 dark:border-amber-800",
    iconClass: "text-amber-600 dark:text-amber-400",
    textClass: "text-amber-700 dark:text-amber-300",
    labelKey: "dual_run.status_closing_soon",
  },
  critical: {
    icon: AlertTriangle,
    containerClass: "bg-orange-50/80 dark:bg-orange-950/50 border-orange-200 dark:border-orange-800",
    iconClass: "text-orange-600 dark:text-orange-400",
    textClass: "text-orange-700 dark:text-orange-300",
    labelKey: "dual_run.status_critical",
  },
  final: {
    icon: AlertTriangle,
    containerClass: "bg-rose-50/80 dark:bg-rose-950/50 border-rose-200 dark:border-rose-800 animate-pulse",
    iconClass: "text-rose-600 dark:text-rose-400",
    textClass: "text-rose-700 dark:text-rose-300",
    labelKey: "dual_run.status_final",
  },
  closed: {
    icon: CheckCircle,
    containerClass: "bg-emerald-50/80 dark:bg-emerald-950/50 border-emerald-200 dark:border-emerald-800",
    iconClass: "text-emerald-600 dark:text-emerald-400",
    textClass: "text-emerald-700 dark:text-emerald-300",
    labelKey: "dual_run.status_closed",
  },
};

const DISMISS_STORAGE_KEY = "adr005_dual_run_banner_dismissed";

// ── Component ───────────────────────────────────────────────────────────

/**
 * ADR-005 §3: Dual-run window countdown banner.
 *
 * Displayed on the admin dashboard during (and shortly after) the 30-day
 * dual-run window.  Shows days remaining, warns about modifier/exception
 * loss on rollback, and auto-hides when the window has been closed for
 * more than 7 days.
 */
export default function DualRunCountdownBanner() {
  const { t } = useTranslation();
  const { data, isLoading } = useCascadeDualRun();
  const [dismissed, setDismissed] = useState(
    () => localStorage.getItem(DISMISS_STORAGE_KEY) === "true",
  );

  const handleDismiss = useCallback(() => {
    setDismissed(true);
    localStorage.setItem(DISMISS_STORAGE_KEY, "true");
  }, []);

  // Reset dismiss when window phase changes (e.g. re-opened after rollback)
  useEffect(() => {
    if (data?.window_phase) {
      const lastPhase = localStorage.getItem("adr005_last_dual_run_phase");
      if (lastPhase !== data.window_phase) {
        localStorage.setItem("adr005_last_dual_run_phase", data.window_phase);
        setDismissed(false);
        localStorage.removeItem(DISMISS_STORAGE_KEY);
      }
    }
  }, [data?.window_phase]);

  // Don't render during loading or when dismissed or when window long-closed
  if (isLoading || !data || dismissed) return null;

  // If closed and no data to show, hide after grace period
  if (data.window_phase === "closed" && !data.enabled) {
    // Still show for up to 7 days after closure to inform admin
    const closedKey = "adr005_dual_run_closed_seen";
    if (localStorage.getItem(closedKey) === "true") return null;
    localStorage.setItem(closedKey, "true");
  }

  const phase: PhaseConfig = PHASE_MAP[data.window_phase] ?? PHASE_MAP.closed;
  const Icon = phase.icon;

  return (
    <div
      className={cn(
        "flex items-center gap-4 px-5 py-3 rounded-2xl border",
        "transition-all duration-300 animate-in fade-in slide-in-from-top-2",
        phase.containerClass,
      )}
      data-testid="dual-run-banner"
    >
      <Icon className={cn("w-5 h-5 shrink-0", phase.iconClass)} />

      <div className="flex-1 min-w-0">
        <Typography
          className={cn("text-sm font-bold", phase.textClass)}
          variant="p"
        >
          {t(phase.labelKey, { defaultValue: data.message })}
        </Typography>
        <Typography
          className={cn("text-xs mt-0.5 opacity-75", phase.textClass)}
          variant="p"
        >
          {data.message}
        </Typography>
      </div>

      {/* Countdown badge */}
      {data.days_remaining != null && data.window_phase !== "closed" && (
        <div
          className={cn(
            "shrink-0 px-3 py-1 rounded-xl text-xs font-black",
            "bg-white/60 dark:bg-white/10 backdrop-blur-sm",
            phase.textClass,
          )}
        >
          {data.days_remaining === 0
            ? t("dual_run.last_day", { defaultValue: "Last day" })
            : t("dual_run.days_remaining", {
                count: data.days_remaining,
                defaultValue: "{{count}}d left",
              })}
        </div>
      )}

      {/* Dismiss button */}
      <button
        aria-label={t("common.dismiss", { defaultValue: "Dismiss" })}
        className={cn(
          "shrink-0 p-1.5 rounded-lg transition-colors",
          "hover:bg-black/5 dark:hover:bg-white/5",
          phase.textClass,
        )}
        onClick={handleDismiss}
        type="button"
      >
        <svg
          className="w-4 h-4"
          fill="none"
          stroke="currentColor"
          strokeWidth={2}
          viewBox="0 0 24 24"
        >
          <path d="M18 6 6 18M6 6l12 12" />
        </svg>
      </button>
    </div>
  );
}
