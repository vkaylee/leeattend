import { Loader2, Info } from "lucide-react";
import React from "react";

import ResponsiveTable from "@/components/ui/responsive-table";
import { cn } from "@/lib/utils";

interface HistoryItem {
  amount: number;
  date: string;
  id: string;
  reason?: string | null;
  transaction_type: string;
}

interface HistoryTableProps {
  formatDate: (date: string) => string;
  history: HistoryItem[];
  isLoading: boolean;
  t: (key: string) => string;
}

const getTransactionColor = (type: string) => {
  switch (type) {
    case "accrual": return "bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 border-emerald-100 dark:border-emerald-800";
    case "usage": return "bg-rose-50 dark:bg-rose-950 text-rose-600 dark:text-rose-300 border-rose-100 dark:border-rose-800";
    case "adjustment": return "bg-amber-50 dark:bg-amber-950 text-amber-600 dark:text-amber-300 border-amber-100 dark:border-amber-800";
    case "carry_over": return "bg-blue-50 dark:bg-blue-950 text-blue-600 dark:text-blue-300 border-blue-100 dark:border-blue-800";
    default: return "bg-slate-50 dark:bg-slate-800 text-slate-500 dark:text-slate-400 border-slate-100 dark:border-slate-800";
  }
};

function HistoryMobileCard({ item, formatDate }: { formatDate: (date: string) => string; item: HistoryItem; }) {
  return (
    <div className="rounded-2xl border border-slate-100 bg-white p-5 shadow-sm dark:border-slate-800 dark:bg-slate-900">
      <div className="flex items-start justify-between gap-3">
        <span className="rounded-xl bg-slate-50 px-3 py-1.5 font-mono text-xs font-bold uppercase text-slate-600 dark:bg-slate-800 dark:text-slate-300">
          {formatDate(item.date)}
        </span>
        <span className={cn("rounded-full border px-2.5 py-0.5 text-[9px] font-bold uppercase tracking-widest", getTransactionColor(item.transaction_type))}>
          {item.transaction_type}
        </span>
      </div>

      <div className="mt-3 flex items-center justify-between border-t border-slate-100 pt-3 dark:border-slate-800">
        <span className="text-[10px] font-bold uppercase tracking-wider text-slate-400">Amount</span>
        <span className={cn("font-mono text-sm font-bold", item.amount > 0 ? "text-emerald-600 dark:text-emerald-300" : "text-rose-600 dark:text-rose-300")}>
          {item.amount > 0 ? "+" : ""}{item.amount}
        </span>
      </div>

      {item.reason ? (
        <div className="mt-2 truncate rounded-lg bg-slate-50 px-3 py-1.5 text-[10px] font-medium text-slate-600 dark:bg-slate-800 dark:text-slate-400" title={item.reason}>
          {item.reason}
        </div>
      ) : null}
    </div>
  );
}

export function HistoryTable({ history, isLoading, formatDate, t }: HistoryTableProps) {
  if (isLoading) {
    return <div className="p-20 text-center"><Loader2 className="mx-auto h-10 w-10 animate-spin text-primary-600 dark:text-primary-400" /></div>;
  }

  return (
    <ResponsiveTable<HistoryItem>
      data={history}
      emptyState={{
        icon: <Info className="h-10 w-10" />,
        title: t("leave_balances.history.no_history"),
      }}
      getCardKey={(item) => item.id}
      isLoading={isLoading}
      renderCard={(item) => (
        <HistoryMobileCard formatDate={formatDate} item={item} />
      )}
      renderDesktopTable={() => (
        <div className="overflow-hidden">
          <table className="w-full border-collapse text-left">
            <thead>
              <tr>
                <th className="px-6 py-4 text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400">{t("common.date")}</th>
                <th className="px-6 py-4 text-center text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400">{t("common.type")}</th>
                <th className="px-6 py-4 text-right text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400">{t("common.total")}</th>
                <th className="px-6 py-4 text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400">{t("common.reason")}</th>
              </tr>
            </thead>
            <tbody>
              {history.map((item) => (
                <tr key={item.id} className="border-t border-slate-50 dark:border-slate-800">
                  <td className="px-6 py-5 font-mono text-xs font-bold uppercase text-slate-500 dark:text-slate-400">{formatDate(item.date)}</td>
                  <td className="px-6 py-5 text-center">
                    <span className={cn("rounded-full border px-2.5 py-1 text-[9px] font-bold uppercase tracking-widest", getTransactionColor(item.transaction_type))}>
                      {item.transaction_type}
                    </span>
                  </td>
                  <td className={cn("px-6 py-5 text-right font-mono text-sm font-bold tracking-tighter", item.amount > 0 ? "text-emerald-600 dark:text-emerald-300" : "text-rose-600 dark:text-rose-300")}>
                    {item.amount > 0 ? "+" : ""}{item.amount}
                  </td>
                  <td className="max-w-[200px] truncate px-6 py-5 text-[10px] font-bold uppercase tracking-tighter text-slate-500 dark:text-slate-400" title={item.reason ?? ""}>
                    {item.reason ?? "-"}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    />
  );
}
