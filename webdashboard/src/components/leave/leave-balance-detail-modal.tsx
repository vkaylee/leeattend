import { useQuery } from "@tanstack/react-query";
import { History as HistoryIcon } from "lucide-react";
import React, { useState } from "react";
import { createPortal } from "react-dom";
import { useTranslation } from "react-i18next";

import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";
import { Typography } from "@/components/ui/typography";
import { LeaveBalance, fetchLeaveHistory } from "@/lib/api";

import { LeaveBalanceBreakdown } from "./leave-balance-detail-components/leave-balance-breakdown";
import { LeaveBalanceHeader } from "./leave-balance-detail-components/leave-balance-header";
import { LeaveBalanceHero } from "./leave-balance-detail-components/leave-balance-hero";
import { HistoryList } from "./leave-balance-detail-components/leave-balance-history";
import { createQueryKey } from "@/lib/query-keys";


interface LeaveBalanceDetailModalProps {
  balance: LeaveBalance;
  isOpen: boolean;
  onClose: () => void;
}

export function LeaveBalanceDetailModal({ balance, isOpen, onClose }: LeaveBalanceDetailModalProps) {
  const { t } = useTranslation();
  const [activeTab, setActiveTab] = useState("all");

  const { data: history = [], isLoading } = useQuery({
    queryKey: createQueryKey("leave-history",balance.employee_id, balance.leave_type_id),
    queryFn: () => fetchLeaveHistory(balance.employee_id, balance.leave_type_id),
    enabled: isOpen ,
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

      <div className="relative bg-white dark:bg-slate-900 rounded-3xl shadow-2xl max-w-2xl w-full max-h-[90dvh] overflow-hidden animate-in zoom-in-95 slide-in-from-bottom-8 duration-500 flex flex-col">
        <LeaveBalanceHeader balance={balance} onClose={onClose} />

        <div className="min-h-0 flex-1 overflow-y-auto p-6 md:p-8 space-y-8">
          <LeaveBalanceHero balance={balance} usagePercent={usagePercent} />

          <LeaveBalanceBreakdown balance={balance} />

          {/* History Section */}
          <div className="space-y-4">
            <div className="flex items-center gap-3 pb-3 border-b border-slate-100 dark:border-slate-800">
              <HistoryIcon className="w-5 h-5 text-slate-600 dark:text-slate-300" />
              <Typography className="text-sm font-bold text-slate-600 dark:text-slate-300 uppercase tracking-[0.15em]" variant="h3">
                {t("leave_balances.detail.history", "Transaction History")}
              </Typography>
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
