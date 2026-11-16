import type React from "react";

import { Ban, CheckCircle2, Clock, Loader2, RefreshCw, ShieldAlert, XCircle } from "lucide-react";

interface StatusStyle {
  className: string;
  icon: React.ReactNode;
}

export function getStatusStyle(status: string): StatusStyle {
  switch (status) {
    case "completed":
      return {
        className: "bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300",
        icon: <CheckCircle2 size={12} />,
      };
    case "failed":
      return {
        className: "bg-rose-50 dark:bg-rose-950 text-rose-600 dark:text-rose-300",
        icon: <XCircle size={12} />,
      };
    case "processing":
      return {
        className: "bg-amber-50 dark:bg-amber-950 text-amber-600 dark:text-amber-300",
        icon: <Loader2 className="motion-safe:animate-spin" size={12} />,
      };
    case "retryable":
      return {
        className: "bg-sky-50 dark:bg-sky-950 text-sky-600 dark:text-sky-300",
        icon: <RefreshCw size={12} />,
      };
    case "dead_letter":
      return {
        className: "bg-orange-50 dark:bg-orange-950 text-orange-600 dark:text-orange-300",
        icon: <ShieldAlert size={12} />,
      };
    case "lost":
      return {
        className: "bg-red-100 dark:bg-red-950 text-red-700 dark:text-red-300",
        icon: <Ban size={12} />,
      };
    case "queued":
    case "dispatching":
      return {
        className: "bg-blue-50 dark:bg-blue-950 text-blue-600 dark:text-blue-300",
        icon: <Clock size={12} />,
      };
    default:
      return {
        className: "bg-slate-100 dark:bg-slate-700 text-slate-500 dark:text-slate-400",
        icon: <Clock size={12} />,
      };
  }
}
