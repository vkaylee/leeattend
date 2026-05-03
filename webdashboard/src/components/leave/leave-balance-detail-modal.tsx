import { useQuery } from "@tanstack/react-query";
import { History as HistoryIcon, Clock, ArrowUpRight, ArrowDownRight, Calendar } from "lucide-react";
import React, { useState } from "react";
import { createPortal } from "react-dom";
import { useTranslation } from "react-i18next";

import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";
import { WorkspaceLeaveBalance, fetchLeaveHistory, getCurrentWorkspaceId, LeaveHistoryItem } from "@/lib/api";
import { cn } from "@/lib/utils";

import { LeaveBalanceHeader } from "./leave-balance-detail-components/leave-balance-header";
import { LeaveBalanceHero } from "./leave-balance-detail-components/leave-balance-hero";
import { LeaveBalanceBreakdown } from "./leave-balance-detail-components/leave-balance-breakdown";

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
    queryFn: () => wsId ? fetchLeaveHistory(wsId, balance.employee_id, balance.leave_type_id) : Promise.resolve([]),
    enabled: isOpen && !!wsId,
  });

  if (!isOpen) return null;

  const totalEntitled = balance.entitlement + balance.accrued_amount + balance.carried_over;
  const usagePercent = Math.round((balance.used / (totalEntitled || 1)) * 100);

  const accruals = history.filter(h => h.transaction_type === "accrual" || h.transaction_type === "adjustment" || h.transaction_type === "carry_over");
  const usage = history.filter(h => h.transaction_type === "usage");

  return createPortal(
    <div className="fixed inset-0 z-[9999] flex items-center justify-center p-4 animate-in fade-in duration-300">
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

      <div className="relative bg-white dark:bg-slate-900 rounded-3xl shadow-2xl max-w-2xl w-full max-h-[90vh] overflow-y-auto animate-in zoom-in-95 slide-in-from-bottom-8 duration-500 flex flex-col">
        <LeaveBalanceHeader balance={balance} onClose={onClose} />

        <div className="p-6 md:p-8 space-y-8 overflow-y-auto">
          <LeaveBalanceHero balance={balance} usagePercent={usagePercent} />
          
          <LeaveBalanceBreakdown balance={balance} />

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
