import { Eye, History } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { WorkspaceLeaveBalance } from "@/lib/api/types";
import { cn } from "@/lib/utils";

interface MobileBalanceCardProps {
  balance: WorkspaceLeaveBalance;
  onViewDetails: () => void;
  onViewHistory: () => void;
}

export const MobileBalanceCard: React.FC<MobileBalanceCardProps> = ({
  balance,
  onViewDetails,
  onViewHistory,
}) => {
  const { t } = useTranslation();
  const totalEntitled = balance.entitlement + balance.accrued_amount + balance.carried_over;
  const usagePercent = Math.round((balance.used / (totalEntitled || 1)) * 100);

  return (
    <div className="p-5 space-y-5 bg-white dark:bg-slate-900 hover:bg-slate-50 dark:hover:bg-slate-800/30 transition-all">
      <div className="flex items-center gap-3">
        <div className="w-14 h-14 bg-primary-50 dark:bg-primary-950 rounded-2xl flex items-center justify-center text-primary-600 dark:text-primary-400 font-bold text-base shadow-inner border border-primary-100 dark:border-primary-800/50 uppercase flex-shrink-0">
          {balance.employee_name.split(" ").map((n) => n[0]).join("").slice(0, 2)}
        </div>
        <div className="flex-1 min-w-0">
          <div className="font-bold text-slate-800 dark:text-slate-100 text-base tracking-tight uppercase leading-tight truncate">
            {balance.employee_name}
          </div>
          <span className="inline-block mt-1.5 px-3 py-1.5 bg-slate-50 dark:bg-slate-800 text-slate-600 dark:text-slate-300 rounded-xl text-xs font-bold uppercase tracking-wider border border-slate-200 dark:border-slate-700 shadow-sm">
            {balance.leave_type_name}
          </span>
        </div>
      </div>

      <div className="bg-gradient-to-br from-primary-50/50 to-transparent dark:from-primary-950/50 dark:to-transparent rounded-2xl p-5 border border-primary-100 dark:border-primary-800/30">
        <div className="text-[10px] font-bold text-primary-600 dark:text-primary-400 uppercase tracking-[0.2em] mb-2">
          {t("leave_balances.table.remaining")}
        </div>
        <div className="flex items-baseline gap-2">
          <span className={cn(
            "text-5xl font-bold font-mono tracking-tighter",
            balance.remaining > 0 ? "text-emerald-600 dark:text-emerald-300" : "text-slate-500 dark:text-slate-400"
          )}>
            {balance.remaining}
          </span>
          <span className="text-sm font-bold text-slate-500 dark:text-slate-400 uppercase">
            {t("common.days_short")}
          </span>
        </div>
      </div>

      <div className="space-y-2">
        <div className="flex items-center justify-between text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-wider">
          <span>{t("leave_balances.table.usage")}</span>
          <span className="font-mono">{usagePercent}%</span>
        </div>
        <div className="w-full h-4 bg-slate-100 dark:bg-slate-700 rounded-full overflow-hidden shadow-inner border border-slate-200 dark:border-slate-700/50">
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

      <div className="flex items-center gap-3 pt-3 border-t border-slate-100 dark:border-slate-800">
        <Button
          className="flex-1 min-h-[44px] h-auto py-3.5 px-4 rounded-xl gap-2 font-bold uppercase tracking-wider text-xs shadow-lg shadow-primary-200"
          onClick={onViewDetails}
          variant="primary"
        >
          <Eye className="h-5 w-5" />
          <span>{t("leave_balances.view_details")}</span>
        </Button>
        <Button
          className="p-3.5 bg-slate-100 dark:bg-slate-700 hover:bg-slate-200 dark:hover:bg-slate-600 text-slate-600 dark:text-slate-300 rounded-xl min-w-[44px] min-h-[44px] h-auto flex-shrink-0 flex items-center justify-center border-none shadow-none"
          onClick={onViewHistory}
          title={t("leave_balances.table.history_tooltip")}
          variant="ghost"
        >
          <History className="h-5 w-5" />
        </Button>
      </div>
    </div>
  );
};

