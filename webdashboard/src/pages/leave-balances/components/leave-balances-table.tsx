import React from "react";
import { useTranslation } from "react-i18next";

import { WorkspaceLeaveBalance } from "@/lib/api/types";

import { DesktopBalanceRow } from "./desktop-balance-row";
import { MobileBalanceCard } from "./mobile-balance-card";

interface LeaveBalancesTableProps {
  balances: WorkspaceLeaveBalance[];
  isLoading: boolean;
  onViewDetails: (balance: WorkspaceLeaveBalance) => void;
  onViewHistory: (employee: { id: string, name: string, leaveTypeId?: string, leaveTypeName?: string }) => void;
}

export const LeaveBalancesTable: React.FC<LeaveBalancesTableProps> = ({
  balances,
  isLoading,
  onViewDetails,
  onViewHistory,
}) => {
  const { t } = useTranslation();

  return (
    <>
      {/* Desktop Table View (≥1024px) */}
      <div className="hidden lg:block overflow-x-auto">
        <table className="w-full text-left border-collapse">
          <thead>
            <tr className="bg-slate-50 dark:bg-slate-800/50 border-b border-slate-100 dark:border-slate-800">
              <th className="px-6 py-5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em]">{t("common.employee")}</th>
              <th className="px-5 py-5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em]">{t("common.type")}</th>
              <th className="px-5 py-5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em] text-right">{t("leave_balances.table.remaining")}</th>
              <th className="px-5 py-5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em]">{t("leave_balances.table.usage")}</th>
              <th className="px-6 py-5 text-right text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em]">{t("common.actions")}</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-slate-50 dark:divide-slate-800">
            {isLoading && !balances.length ? (
              <tr><td className="px-6 py-20 text-center text-slate-500 dark:text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50 animate-pulse" colSpan={5}>{t("common.loading")}</td></tr>
            ) : balances.length === 0 ? (
              <tr><td className="px-6 py-32 text-center text-slate-500 dark:text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50" colSpan={5}>{t("attendance_logs.messages.no_data")}</td></tr>
            ) : balances.map((balance, index) => (
              <DesktopBalanceRow
                balance={balance}
                key={`${balance.employee_id}-${balance.leave_type_id}-${index}`}
                onViewDetails={() => { onViewDetails(balance); }}
                onViewHistory={() => { 
                  onViewHistory({ 
                    id: balance.employee_id, 
                    name: balance.employee_name, 
                    leaveTypeId: balance.leave_type_id, 
                    leaveTypeName: balance.leave_type_name 
                  }); 
                }}
              />
            ))}
          </tbody>
        </table>
      </div>

      {/* Tablet/Mobile Card View (<1024px) */}
      <div className="lg:hidden divide-y divide-slate-100 dark:divide-slate-800">
        {isLoading && !balances.length ? (
          <div className="px-5 py-20 text-center text-slate-500 dark:text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50 animate-pulse">{t("common.loading")}</div>
        ) : balances.length === 0 ? (
          <div className="px-5 py-32 text-center text-slate-500 dark:text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50">{t("attendance_logs.messages.no_data")}</div>
        ) : balances.map((balance, index) => (
          <MobileBalanceCard
            balance={balance}
            key={`${balance.employee_id}-${balance.leave_type_id}-${index}`}
            onViewDetails={() => { onViewDetails(balance); }}
            onViewHistory={() => { 
              onViewHistory({ 
                id: balance.employee_id, 
                name: balance.employee_name, 
                leaveTypeId: balance.leave_type_id, 
                leaveTypeName: balance.leave_type_name 
              }); 
            }}
          />
        ))}
      </div>
    </>
  );
};
