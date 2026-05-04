import { Loader2, Info } from "lucide-react";
import React from "react";

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

export function HistoryTable({ history, isLoading, formatDate, t }: HistoryTableProps) {
  if (isLoading) {
    return <div className="p-20 text-center"><Loader2 className="animate-spin w-10 h-10 text-primary-600 dark:text-primary-400 mx-auto" /></div>;
  }

  if (history.length === 0) {
    return (
      <div className="py-20 text-center border-2 border-dashed border-slate-100 dark:border-slate-800 rounded-[2.5rem] bg-slate-50 dark:bg-slate-800/30 flex flex-col items-center">
        <Info className="text-slate-200 dark:text-slate-700 mb-4" size={32} />
        <p className="text-[10px] font-bold text-slate-300 dark:text-slate-600 uppercase tracking-widest">{t("leave_balances.history.no_history")}</p>
      </div>
    );
  }

  return (
    <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-[2rem] overflow-hidden shadow-sm animate-in fade-in slide-in-from-bottom-4 duration-500">
      <table className="w-full text-left border-collapse">
        <thead>
          <tr className="bg-slate-50 dark:bg-slate-800/50 border-b border-slate-100 dark:border-slate-800">
            <th className="px-6 py-4 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">{t("common.date")}</th>
            <th className="px-6 py-4 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest text-center">{t("common.type")}</th>
            <th className="px-6 py-4 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest text-right">{t("common.total")}</th>
            <th className="px-6 py-4 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">{t("common.reason")}</th>
          </tr>
        </thead>
        <tbody className="divide-y divide-slate-50 dark:divide-slate-800">
          {history.map((item) => (
            <tr className="hover:bg-slate-50 dark:hover:bg-slate-800/50 transition-colors group" key={item.id}>
              <td className="px-6 py-5 text-xs font-bold text-slate-500 dark:text-slate-400 font-mono uppercase">{formatDate(item.date)}</td>
              <td className="px-6 py-5 text-center">
                <span className={cn("px-2.5 py-1 rounded-full text-[9px] font-bold uppercase tracking-widest border", getTransactionColor(item.transaction_type))}>
                  {item.transaction_type}
                </span>
              </td>
              <td className={cn("px-6 py-5 text-right font-bold text-sm font-mono tracking-tighter", item.amount > 0 ? "text-emerald-600 dark:text-emerald-300" : "text-rose-600 dark:text-rose-300")}>
                {item.amount > 0 ? "+" : ""}{item.amount}
              </td>
              <td className="px-6 py-5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-tighter max-w-[200px] truncate" title={item.reason ?? ""}>
                {item.reason ?? "-"}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
