import { Award } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { LeaveBalance } from "@/lib/api";
import { cn } from "@/lib/utils";

interface LeaveBalanceHeroProps {
  balance: LeaveBalance;
  usagePercent: number;
}

export function LeaveBalanceHero({ balance, usagePercent }: LeaveBalanceHeroProps) {
  const { t } = useTranslation();
  return (
    <div className="bg-gradient-to-br from-primary-50 via-primary-50/50 to-transparent dark:from-primary-950 dark:via-primary-950/50 dark:to-transparent rounded-3xl p-8 border border-primary-100 dark:border-primary-800/50 shadow-inner">
      <div className="flex items-center gap-3 mb-3">
        <Award className="w-6 h-6 text-primary-600 dark:text-primary-400" />
        <div className="text-xs font-bold text-primary-600 dark:text-primary-400 uppercase tracking-[0.2em]">
          {t("leave_balances.table.remaining")}
        </div>
      </div>
      <div className={cn(
        "text-6xl md:text-7xl font-bold font-mono tracking-tighter",
        balance.remaining > 0 ? "text-emerald-600 dark:text-emerald-300" : "text-slate-500 dark:text-slate-400"
      )}>
        {balance.remaining}
        <span className="text-2xl md:text-3xl ml-2 text-slate-500 dark:text-slate-400 font-bold uppercase tracking-tight">
          {t("common.days_short", "days")}
        </span>
      </div>

      <div className="mt-6 space-y-3">
        <div className="flex items-center justify-between text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-wider">
          <span>{t("leave_balances.table.usage")}</span>
          <span className="font-mono">{usagePercent}%</span>
        </div>
        <div className="w-full h-3 bg-white dark:bg-slate-900/50 rounded-full overflow-hidden shadow-inner border border-primary-100 dark:border-primary-800/30">
          <div
            className={cn(
              "h-full rounded-full transition-all duration-1000 shadow-sm",
              usagePercent > 80 ? "bg-rose-500 dark:bg-rose-400 shadow-rose-200" :
                usagePercent > 50 ? "bg-amber-500 dark:bg-amber-400 shadow-amber-200" :
                  "bg-primary-600 shadow-primary-200"
            )}
            style={{ width: `${Math.min(100, usagePercent)}%` }}
          />
        </div>
      </div>
    </div>
  );
}
