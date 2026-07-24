import { Check, DatabaseZap } from "lucide-react";
import React from "react";

import { Typography } from "@/components/ui/typography";

import { ProgressStep } from "./progress-step";

interface ProgressOverlayProps {
  dbMode: "auto" | "byod";
  progressPercent: number;
  progressStep: number;
  showProgress: boolean;
  t: (key: string) => string;
}

export function ProgressOverlay({
  dbMode,
  progressPercent,
  progressStep,
  showProgress,
  t,
}: ProgressOverlayProps) {
  if (!showProgress) return null;

  return (
    <div className="fixed inset-0 bg-slate-950/90 dark:bg-slate-950/95 backdrop-blur-md flex items-center justify-center z-50 animate-in fade-in duration-500">
      <div className="w-full max-w-md p-6 sm:p-8 bg-white dark:bg-slate-900 rounded-2xl sm:rounded-[2.5rem] shadow-premium-2xl border border-white/10 dark:border-slate-800/50 text-center space-y-8 animate-in zoom-in-95 duration-500 relative overflow-hidden">
        {/* Top glowing background blob inside loader card */}
        <div className="absolute top-[-20%] left-[-20%] w-[140%] h-[140%] bg-primary-500/10 rounded-full blur-[80px] pointer-events-none" />

        <div className="flex flex-col items-center gap-4 relative z-10">
          <div className="relative bg-slate-100 dark:bg-slate-800 w-16 h-16 rounded-2xl flex items-center justify-center shadow-inner mb-2 animate-bounce">
            {progressStep === 5 ? (
              <Check className="text-emerald-500" size={32} />
            ) : (
              <DatabaseZap className="text-primary-500 dark:text-primary-400 animate-pulse" size={28} />
            )}
          </div>
          <Typography className="text-2xl font-bold text-slate-800 dark:text-slate-100 tracking-tight" variant="h2">
            {t("onboarding.progress.title")}
          </Typography>
          <Typography className="text-xs text-slate-500 dark:text-slate-400 max-w-xs leading-relaxed" variant="p">
            {t("onboarding.progress.subtitle")}
          </Typography>
        </div>

        {/* Circular Progress Bar Indicator */}
        <div className="relative w-full h-2.5 bg-slate-100 dark:bg-slate-800 rounded-full overflow-hidden z-10 shadow-inner">
          <div
            className="h-full bg-gradient-to-r from-primary-500 to-blue-500 rounded-full transition-all duration-300 shadow-lg"
            style={{ width: `${progressPercent}%` }}
          />
        </div>

        <div className="flex justify-between items-center text-[10px] font-bold text-slate-400 dark:text-slate-500 px-1 relative z-10">
          <span>{dbMode === "auto" ? "AUTO-PROVISION" : "BYOD DATABASE"}</span>
          <span className="text-primary-500 dark:text-primary-400">{Math.round(progressPercent)}%</span>
        </div>

        {/* Steps Timeline Grid */}
        <div className="text-left space-y-4 pt-2 border-t border-slate-100 dark:border-slate-800 relative z-10">
          {[
            { label: t("onboarding.progress.step_1"), stepNum: 1 },
            { label: t("onboarding.progress.step_2"), stepNum: 2 },
            { label: t("onboarding.progress.step_3"), stepNum: 3 },
            { label: t("onboarding.progress.step_4"), stepNum: 4 },
          ].map((step) => (
            <ProgressStep
              key={step.stepNum}
              label={step.label}
              progressStep={progressStep}
              stepNum={step.stepNum}
            />
          ))}
        </div>
      </div>
    </div>
  );
}
