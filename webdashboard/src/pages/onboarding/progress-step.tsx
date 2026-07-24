import { Check, Loader2 } from "lucide-react";
import React from "react";

interface ProgressStepProps {
  label: string;
  progressStep: number;
  stepNum: number;
}

export function ProgressStep({ label, progressStep, stepNum }: ProgressStepProps) {
  const status: "completed" | "active" | "pending" =
    progressStep === 5 || progressStep > stepNum
      ? "completed"
      : progressStep === stepNum
      ? "active"
      : "pending";

  const badgeContents = {
    completed: <Check size={12} />,
    active: <Loader2 className="animate-spin" size={12} />,
    pending: stepNum,
  };

  const badgeColors = {
    completed: "bg-emerald-500 text-white",
    active: "bg-primary-500 text-white animate-pulse",
    pending: "bg-slate-100 dark:bg-slate-800 text-slate-400 dark:text-slate-600",
  };

  const textColors = {
    completed: "text-slate-800 dark:text-slate-200",
    active: "text-primary-600 dark:text-primary-400 font-bold",
    pending: "text-slate-500 dark:text-slate-400",
  };

  return (
    <div className={`flex items-center gap-3 transition-opacity duration-300 ${status === "pending" ? "opacity-40" : "opacity-100"}`}>
      <div className={`w-6 h-6 rounded-full flex items-center justify-center text-[10px] font-bold transition-all ${badgeColors[status]}`}>
        {badgeContents[status]}
      </div>
      <span className={`text-xs font-semibold ${textColors[status]}`}>
        {label}
      </span>
    </div>
  );
}
