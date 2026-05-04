import { Clock, ArrowUpRight, ArrowDownRight, Calendar } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { LeaveHistoryItem } from "@/lib/api";
import { cn } from "@/lib/utils";

export function HistoryItemRow({ item }: { item: LeaveHistoryItem }) {
  const { t } = useTranslation();
  const isPositive = item.amount > 0;

  return (
    <div className="flex items-center gap-4 p-4 bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-2xl hover:border-primary-100 dark:border-primary-800 hover:bg-slate-50 dark:hover:bg-slate-800/50 transition-all group">
      <div className={cn(
        "w-10 h-10 rounded-xl flex items-center justify-center flex-shrink-0 shadow-sm transition-transform group-hover:scale-110",
        isPositive ? "bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300" : "bg-rose-50 dark:bg-rose-950 text-rose-600 dark:text-rose-300"
      )}>
        {isPositive ? <ArrowUpRight size={18} /> : <ArrowDownRight size={18} />}
      </div>

      <div className="flex-1 min-w-0">
        <div className="flex items-center justify-between mb-0.5">
          <span className="font-bold text-slate-800 dark:text-slate-100 text-sm uppercase tracking-tight truncate">
            {item.reason ?? t(`leave_balances.transaction_type.${item.transaction_type}`)}
          </span>
          <span className={cn(
            "font-mono font-bold text-sm",
            isPositive ? "text-emerald-600 dark:text-emerald-300" : "text-rose-600 dark:text-rose-300"
          )}>
            {isPositive ? "+" : ""}{item.amount}
          </span>
        </div>
        <div className="flex items-center gap-2 text-[10px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-wider">
          <Calendar size={10} />
          {new Date(item.date).toLocaleDateString()}
          <span className="w-1 h-1 bg-slate-200 dark:bg-slate-600 rounded-full" />
          {t(`leave_balances.transaction_type.${item.transaction_type}`)}
        </div>
      </div>
    </div>
  );
}

export function HistoryList({ items, isLoading }: { items: LeaveHistoryItem[], isLoading: boolean }) {
  const { t } = useTranslation();

  if (isLoading) {
    return (
      <div className="space-y-4">
        {[1, 2, 3].map(index => (
          <div className="h-20 bg-slate-50 dark:bg-slate-800 animate-pulse rounded-2xl" key={index} />
        ))}
      </div>
    );
  }

  if (items.length === 0) {
    return (
      <div className="py-12 text-center">
        <Clock className="w-12 h-12 text-slate-200 dark:text-slate-700 mx-auto mb-3" />
        <p className="text-sm font-bold text-slate-400 dark:text-slate-500 uppercase tracking-widest italic">
          {t("common.no_data", "No history found")}
        </p>
      </div>
    );
  }

  return (
    <div className="space-y-3">
      {items.map((item) => (
        <HistoryItemRow item={item} key={item.id} />
      ))}
    </div>
  );
}
