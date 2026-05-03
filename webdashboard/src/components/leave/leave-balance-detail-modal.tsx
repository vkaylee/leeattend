import { useQuery } from "@tanstack/react-query";
import { X, TrendingUp, Calendar, Award, ArrowUpRight, ArrowDownRight, History as HistoryIcon, Clock } from "lucide-react";
import { useState } from "react";
import { createPortal } from "react-dom";
import { useTranslation } from "react-i18next";

import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";
import { WorkspaceLeaveBalance, fetchLeaveHistory, getCurrentWorkspaceId, LeaveHistoryItem } from "@/lib/api";
import { cn } from "@/lib/utils";

interface LeaveBalanceDetailModalProps {
  balance: WorkspaceLeaveBalance;
  isOpen: boolean;
  onClose: () => void;
}

export function LeaveBalanceDetailModal({ balance, isOpen, onClose }: LeaveBalanceDetailModalProps) {
  const { t } = useTranslation();
  const wsId = getCurrentWorkspaceId();
  const [activeTab, setActiveTab] = useState("all");

  const { data: history = [], isLoading } = useQuery({
    queryKey: ["leave-history", wsId, balance.employee_id, balance.leave_type_id],
    queryFn: () => fetchLeaveHistory(wsId!, balance.employee_id, balance.leave_type_id),
    enabled: isOpen && !!wsId,
  });

  if (!isOpen) return null;

  const totalEntitled = balance.entitlement + balance.accrued_amount + balance.carried_over;
  const usagePercent = Math.round((balance.used / (totalEntitled || 1)) * 100);

  const accruals = history.filter(h => h.transaction_type === "accrual" || h.transaction_type === "adjustment" || h.transaction_type === "carry_over");
  const usage = history.filter(h => h.transaction_type === "usage");

  return createPortal(
 
 
    <div className="fixed inset-0 z-[9999] flex items-center justify-center p-4 animate-in fade-in duration-300">
      {/* Backdrop */}
      <div 
        className="absolute inset-0 bg-slate-900/20 backdrop-blur-md"
        onClick={onClose}
        onKeyDown={(e) => {
          if (e.key === 'Escape' || e.key === 'Enter' || e.key === ' ') {
 
 
            onClose();
          }
        }}
        role="presentation"
      />

      {/* Modal Content */}
      <div className="relative bg-white dark:bg-slate-900 rounded-3xl shadow-2xl max-w-2xl w-full max-h-[90vh] overflow-y-auto animate-in zoom-in-95 slide-in-from-bottom-8 duration-500 flex flex-col">
        {/* Header */}
        <div className="sticky top-0 bg-white dark:bg-slate-900 border-b border-slate-100 dark:border-slate-800 px-6 md:px-8 py-5 md:py-6 rounded-t-3xl z-10 flex-shrink-0">
          <div className="flex items-start justify-between gap-4">
            <div className="flex items-center gap-4 flex-1 min-w-0">
              <div className="w-14 h-14 bg-primary-50 dark:bg-primary-950 rounded-2xl flex items-center justify-center text-primary-600 dark:text-primary-400 font-bold text-base shadow-inner border border-primary-100 dark:border-primary-800/50 uppercase flex-shrink-0">
                {balance.employee_name.split(' ').map(n => n[0]).join('').slice(0, 2)}
              </div>
              <div className="flex-1 min-w-0">
                <h2 className="text-xl md:text-2xl font-bold text-slate-800 dark:text-slate-100 tracking-tight uppercase leading-tight truncate">
                  {balance.employee_name}
                </h2>
                <div className="flex items-center gap-2 mt-2">
                  <span className="inline-block px-3 py-1.5 bg-slate-50 dark:bg-slate-800 text-slate-600 dark:text-slate-300 rounded-xl text-xs font-bold uppercase tracking-wider border border-slate-200 dark:border-slate-700 shadow-sm">
                    {balance.leave_type_name}
                  </span>
                </div>
              </div>
            </div>
            <button
              aria-label={t("common.close")}
              className="p-2.5 text-slate-500 dark:text-slate-400 hover:text-slate-600 dark:hover:text-slate-300 dark:text-slate-600 hover:bg-slate-50 dark:hover:bg-slate-800 rounded-xl transition-all active:scale-90 flex-shrink-0 min-w-[44px] min-h-[44px]"
              onClick={onClose}
            >
              <X size={24} />
            </button>
          </div>
        </div>

        {/* Body */}
        <div className="p-6 md:p-8 space-y-8 overflow-y-auto">
          {/* Hero Metric: Remaining Balance */}
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

            {/* Usage Bar */}
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

          {/* Breakdown Section */}
          <div className="space-y-4">
            <div className="flex items-center gap-3 pb-3 border-b border-slate-100 dark:border-slate-800">
              <TrendingUp className="w-5 h-5 text-slate-600 dark:text-slate-300" />
              <h3 className="text-sm font-bold text-slate-600 dark:text-slate-300 uppercase tracking-[0.15em]">
                {t("leave_balances.detail.breakdown", "Balance Breakdown")}
              </h3>
            </div>

            <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
              {/* Base Entitlement */}
              <div className="bg-slate-50 dark:bg-slate-800 rounded-2xl p-4 border border-slate-100 dark:border-slate-800">
                <div className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-wider mb-1">
                  {t("leave_balances.table.base")}
                </div>
                <div className="text-2xl font-bold font-mono text-slate-600 dark:text-slate-300">
                  {balance.entitlement}
                </div>
              </div>

              {/* Accrued Amount */}
              <div className="bg-primary-50 dark:bg-primary-950 rounded-2xl p-4 border border-primary-100 dark:border-primary-800">
                <div className="text-[10px] font-bold text-primary-600 dark:text-primary-400 uppercase tracking-wider mb-1">
                  {t("leave_balances.table.accrued")}
                </div>
                <div className="text-2xl font-bold font-mono text-primary-600 dark:text-primary-400">
                  +{balance.accrued_amount}
                </div>
              </div>

              {/* Carried Over */}
              <div className="bg-amber-50 dark:bg-amber-950 rounded-2xl p-4 border border-amber-100 dark:border-amber-800">
                <div className="text-[10px] font-bold text-amber-600 dark:text-amber-300 uppercase tracking-wider mb-1">
                  {t("leave_balances.table.carried_over")}
                </div>
                <div className="text-2xl font-bold font-mono text-amber-600 dark:text-amber-300">
                  +{balance.carried_over}
                </div>
              </div>

              {/* Used */}
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

          {/* History Section */}
          <div className="space-y-4">
            <div className="flex items-center gap-3 pb-3 border-b border-slate-100 dark:border-slate-800">
              <HistoryIcon className="w-5 h-5 text-slate-600 dark:text-slate-300" />
              <h3 className="text-sm font-bold text-slate-600 dark:text-slate-300 uppercase tracking-[0.15em]">
                {t("leave_balances.detail.history", "Transaction History")}
              </h3>
            </div>

            <Tabs onValueChange={setActiveTab} value={activeTab}>
              <TabsList className="bg-slate-50 dark:bg-slate-800 p-1 rounded-xl">
                <TabsTrigger value="all">{t("leave_balances.detail.tabs.all", "Tất cả")}</TabsTrigger>
                <TabsTrigger value="accrual">{t("leave_balances.detail.tabs.accruals", "Tích lũy")}</TabsTrigger>
                <TabsTrigger value="usage">{t("leave_balances.detail.tabs.usage", "Sử dụng")}</TabsTrigger>
              </TabsList>

              <TabsContent value="all">
                <HistoryList isLoading={isLoading} items={history} />
              </TabsContent>
              <TabsContent value="accrual">
                <HistoryList isLoading={isLoading} items={accruals} />
              </TabsContent>
              <TabsContent value="usage">
                <HistoryList isLoading={isLoading} items={usage} />
              </TabsContent>
            </Tabs>
          </div>
        </div>
      </div>
    </div>,
    document.body
  );
}

function HistoryList({ items, isLoading }: { items: LeaveHistoryItem[], isLoading: boolean }) {
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

function HistoryItemRow({ item }: { item: LeaveHistoryItem }) {
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
            {item.reason ?? t(`leave_balances.transaction_type.${item.transaction_type ?? ""}`)}
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
          {t(`leave_balances.transaction_type.${item.transaction_type ?? ""}`)}
        </div>
      </div>
    </div>
  );
}
