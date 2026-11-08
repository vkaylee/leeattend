import { Eye, History } from "lucide-react";
import { useTranslation } from "react-i18next";
import type { WorkspaceLeaveBalance } from "@/lib/api/types";

import Button from "@/components/ui/button";
import { cn } from "@/lib/utils";

interface DesktopBalanceRowProps {
  balance: WorkspaceLeaveBalance;
  onViewDetails: () => void;
  onViewHistory: () => void;
}

export const DesktopBalanceRow: React.FC<DesktopBalanceRowProps> = ({
  balance,
  onViewDetails,
  onViewHistory,
}) => {
  const { t } = useTranslation();
  const totalEntitled = balance.entitlement + balance.accrued_amount + balance.carried_over;
  const usagePercent = Math.round((balance.used / (totalEntitled || 1)) * 100);

  return (
    <tr className="hover:bg-slate-50 dark:hover:bg-slate-800/50 transition-all group cursor-pointer" onClick={onViewDetails}>
      <td className="px-6 py-6">
        <div className="flex items-center gap-4">
          <div className="w-12 h-12 bg-primary-50 dark:bg-primary-950 rounded-xl flex items-center justify-center text-primary-600 dark:text-primary-400 font-bold text-sm shadow-inner border border-primary-100 dark:border-primary-800/50 uppercase group-hover:scale-110 transition-transform flex-shrink-0">
            {balance.employee_name.split(" ").map((n) => n[0]).join("").slice(0, 2)}
          </div>
          <div className="font-bold text-slate-800 dark:text-slate-100 text-base tracking-tight uppercase leading-tight">
            {balance.employee_name}
          </div>
        </div>
      </td>

      <td className="px-5 py-6">
        <span className="px-3 py-1.5 bg-slate-50 dark:bg-slate-800 text-slate-600 dark:text-slate-300 rounded-xl text-xs font-bold uppercase tracking-wider border border-slate-200 dark:border-slate-700 shadow-sm inline-block">
          {balance.leave_type_name}
        </span>
      </td>

      <td className="px-5 py-6 text-right">
        <div className="flex items-center justify-end gap-2">
          <span className={cn(
            "text-2xl font-bold px-4 py-2 rounded-xl font-mono shadow-inner border transition-all uppercase tracking-tighter",
            balance.remaining > 0
              ? "text-emerald-600 dark:text-emerald-300 bg-emerald-50 dark:bg-emerald-950 border-emerald-100 dark:border-emerald-800"
              : "text-slate-500 dark:text-slate-400 bg-slate-100 dark:bg-slate-700 border-slate-200 dark:border-slate-700 opacity-60"
          )}>
            {balance.remaining}
          </span>
          <span className="text-xs font-bold text-slate-500 dark:text-slate-400 uppercase">
            {t("common.days_short")}
          </span>
        </div>
      </td>

      <td className="px-5 py-6">
        <div className="flex items-center gap-3">
          <div className="w-32 h-4 bg-slate-100 dark:bg-slate-700 rounded-full overflow-hidden shadow-inner border border-slate-200 dark:border-slate-700/50">
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
          <span className="text-xs font-bold text-slate-500 dark:text-slate-400 font-mono tracking-tighter min-w-[3ch]">
            {usagePercent}%
          </span>
        </div>
      </td>

      <td className="px-6 py-6 text-right" onClick={(e) => { e.stopPropagation(); }}>
        <div className="flex items-center justify-end gap-2 opacity-60 group-hover:opacity-100 transition-all">
          <Button
            className="h-11 w-11 p-2.5 border-primary-200 text-primary-600 hover:bg-primary-100 dark:border-primary-800 dark:bg-primary-950 dark:text-primary-400 dark:hover:bg-primary-900"
            onClick={() => { onViewDetails(); }}
            size="icon"
            title={t("leave_balances.view_details")}
            variant="outline"
          >
            <Eye className="h-5 w-5" />
          </Button>
          <Button
            className="h-11 w-11 p-2.5 border-slate-200 text-slate-500 dark:border-slate-700 dark:text-slate-400"
            onClick={() => { onViewHistory(); }}
            size="icon"
            title={t("leave_balances.table.history_tooltip")}
            variant="outline"
          >
            <History className="h-5 w-5" />
          </Button>
        </div>
      </td>
    </tr>
  );
};
