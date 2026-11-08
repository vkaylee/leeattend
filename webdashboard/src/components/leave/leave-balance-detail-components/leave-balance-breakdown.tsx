import { TrendingUp } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { Typography } from "@/components/ui/typography";
import { LeaveBalance } from "@/lib/api";

export function LeaveBalanceBreakdown({ balance }: { balance: LeaveBalance }) {
  const { t } = useTranslation();
  return (
    <div className="space-y-4">
      <div className="flex items-center gap-3 pb-3 border-b border-slate-100 dark:border-slate-800">
        <TrendingUp className="w-5 h-5 text-slate-600 dark:text-slate-300" />
        <Typography className="text-sm font-bold text-slate-600 dark:text-slate-300 uppercase tracking-[0.15em]" variant="h3">
          {t("leave_balances.detail.breakdown", "Balance Breakdown")}
        </Typography>
      </div>

      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        <div className="bg-slate-50 dark:bg-slate-800 rounded-2xl p-4 border border-slate-100 dark:border-slate-800">
          <div className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-wider mb-1">
            {t("leave_balances.table.base")}
          </div>
          <div className="text-2xl font-bold font-mono text-slate-600 dark:text-slate-300">
            {balance.entitlement}
          </div>
        </div>

        <div className="bg-primary-50 dark:bg-primary-950 rounded-2xl p-4 border border-primary-100 dark:border-primary-800">
          <div className="text-[10px] font-bold text-primary-600 dark:text-primary-400 uppercase tracking-wider mb-1">
            {t("leave_balances.table.accrued")}
          </div>
          <div className="text-2xl font-bold font-mono text-primary-600 dark:text-primary-400">
            +{balance.accrued_amount}
          </div>
        </div>

        <div className="bg-amber-50 dark:bg-amber-950 rounded-2xl p-4 border border-amber-100 dark:border-amber-800">
          <div className="text-[10px] font-bold text-amber-600 dark:text-amber-300 uppercase tracking-wider mb-1">
            {t("leave_balances.table.carried_over")}
          </div>
          <div className="text-2xl font-bold font-mono text-amber-600 dark:text-amber-300">
            +{balance.carried_over}
          </div>
        </div>

        <div className="bg-rose-50 dark:bg-rose-950 rounded-2xl p-4 border border-rose-100 dark:border-rose-800">
          <div className="text-[10px] font-bold text-rose-600 dark:text-rose-300 uppercase tracking-wider mb-1">
            {t("leave_balances.table.used")}
          </div>
          <div className="text-2xl font-bold font-mono text-rose-600 dark:text-rose-300">
            -{balance.used}
          </div>
        </div>
      </div>
    </div>
  );
}
