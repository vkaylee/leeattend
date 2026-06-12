import { Minus, TrendingDown, TrendingUp } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

interface CompensationDiffBadgeProps {
  className?: string;
  delta: string | number;
}

export default function CompensationDiffBadge({
  className = "",
  delta,
}: CompensationDiffBadgeProps) {
  const { t } = useTranslation();
  const deltaNum = typeof delta === "string" ? parseFloat(delta) : delta;
  
  const formatCurrency = (val: number) => {
    return new Intl.NumberFormat("vi-VN", {
      currency: "VND",
      style: "currency",
    }).format(Math.abs(val));
  };

  const formattedDelta = formatCurrency(deltaNum);

  if (deltaNum > 0) {
    return (
      <div
        className={`inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-bold bg-emerald-50 dark:bg-emerald-950/40 text-emerald-600 dark:text-emerald-400 border border-emerald-100 dark:border-emerald-900/50 shadow-sm ${className}`}
      >
        <TrendingUp className="animate-pulse" size={14} />
        <span>
          {t("employees.compensation.delta.increase", {
            amount: formattedDelta,
            defaultValue: `+${formattedDelta}`,
          })}
        </span>
      </div>
    );
  }

  if (deltaNum < 0) {
    return (
      <div
        className={`inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-bold bg-rose-50 dark:bg-rose-950/40 text-rose-600 dark:text-rose-400 border border-rose-100 dark:border-rose-900/50 shadow-sm ${className}`}
      >
        <TrendingDown className="animate-pulse" size={14} />
        <span>
          {t("employees.compensation.delta.decrease", {
            amount: formattedDelta,
            defaultValue: `-${formattedDelta}`,
          })}
        </span>
      </div>
    );
  }

  return (
    <div
      className={`inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-bold bg-slate-50 dark:bg-slate-800 text-slate-500 dark:text-slate-400 border border-slate-100 dark:border-slate-700/50 shadow-sm ${className}`}
    >
      <Minus size={14} />
      <span>{t("employees.compensation.delta.no_change", "Không thay đổi")}</span>
    </div>
  );
}
