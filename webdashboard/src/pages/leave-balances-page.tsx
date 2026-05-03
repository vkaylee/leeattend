import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Search, Filter, History, PlayCircle, Eye, ChevronLeft, ChevronRight, Calendar } from "lucide-react";
import { useState, useCallback } from "react";
import { useTranslation } from "react-i18next";

import { LeaveBalanceDetailModal } from "@/components/leave/leave-balance-detail-modal";
import { LeaveHistoryModal } from "@/components/leave/leave-history-modal";
import { Combobox } from "@/components/ui/combobox";
import { useToast } from "@/components/ui/toast";
import { useConfirm } from "@/hooks/use-confirm";
import { useDebounce } from "@/hooks/use-debounce";
import { fetchWorkspaceLeaveBalances, getCurrentWorkspaceId, triggerAccrualRun, fetchJobStatus } from "@/lib/api";
import { WorkspaceLeaveBalance } from "@/lib/api/types";
import { cn } from "@/lib/utils";


export default function LeaveBalancesPage() {
  const { t } = useTranslation();
  const toast = useToast();
  const queryClient = useQueryClient();
  const wsId = getCurrentWorkspaceId();
  const [searchTerm, setSearchTerm] = useState("");
  const [page, setPage] = useState(1);
  const [selectedYear, setSelectedYear] = useState(new Date().getFullYear());
  const { confirm, ConfirmationDialog } = useConfirm();
  const limit = 20;
  const debouncedSearch = useDebounce(searchTerm, 500);

  // Generate year options: current year ± 2
  const currentYear = new Date().getFullYear();
  const yearOptions = Array.from({ length: 5 }, (_, index) => currentYear - 2 + index);

  const [selectedEmployee, setSelectedEmployee] = useState<{ id: string, name: string, leaveTypeId?: string, leaveTypeName?: string } | null>(null);
  const [selectedBalance, setSelectedBalance] = useState<WorkspaceLeaveBalance | null>(null);

  const { data: paginatedData, isLoading } = useQuery({
    queryKey: ["workspace-leave-balances", wsId, page, debouncedSearch, selectedYear],
    queryFn: () => {
      if (!wsId) throw new Error("Workspace ID is required");
      return fetchWorkspaceLeaveBalances(wsId, { page, limit, q: debouncedSearch, year: selectedYear });
    },
    enabled: !!wsId,
    placeholderData: (previousData) => previousData
  });

  const pollJobStatus = useCallback(async (jobId: string) => {
    const POLL_INTERVAL = 2000; // 2 seconds
    const MAX_POLLS = 60; // 2 min max

    for (let index = 0; index < MAX_POLLS; index++) {
      await new Promise(resolve => setTimeout(resolve, POLL_INTERVAL));
      try {
        if (!wsId) return;
        const status = await fetchJobStatus(wsId, jobId);
        if (status.status === 'completed') {
          void queryClient.invalidateQueries({ queryKey: ["workspace-leave-balances"] });
          toast.success(t("leave_balances.accrual_completed"));
          return;
        }
        if (status.status === 'failed') {
          toast.error(status.error_message ?? t("common.error"));
          return;
        }
        // Still processing — continue polling
      } catch {
        // Network error — stop polling
        toast.error(t("common.error"));
        return;
      }
    }
    // Timed out
    toast.error(t("leave_balances.accrual_timeout"));
  }, [wsId, queryClient, toast, t]);

  const accrualMutation = useMutation({
    mutationFn: () => {
      if (!wsId) throw new Error("Workspace ID is required");
      return triggerAccrualRun(wsId, { date: new Date().toISOString().split('T')[0] });
    },
    onSuccess: (data) => {
      if (data.status === 'queued' && data.job_id) {
        toast.success(t("leave_balances.accrual_queued"));
        void pollJobStatus(data.job_id);
      } else {
        void queryClient.invalidateQueries({ queryKey: ["workspace-leave-balances"] });
        toast.success(data.message || t("common.success"));
      }
    },
    onError: () => { toast.error(t("common.error")); }
  });

  const balances = paginatedData?.data ?? [];
  const totalItems = paginatedData?.total ?? 0;
  const totalPages = Math.ceil(totalItems / limit);

  return (
    <div className="space-y-10 animate-in fade-in duration-700 pb-20">
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-6 px-2">
        <div className="space-y-1">
          <h1 className="text-3xl md:text-5xl font-bold tracking-tighter text-slate-800 dark:text-slate-100 uppercase leading-none">{t("leave_balances.title")}</h1>
          <p className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] opacity-60">{t("leave_balances.subtitle")}</p>
        </div>
        <button
          className="bg-primary-600 hover:bg-primary-700 text-white font-bold px-6 md:px-8 py-3.5 md:py-3 rounded-2xl flex items-center justify-center gap-3 shadow-xl shadow-primary-100 transition-all active:scale-95 text-xs uppercase tracking-widest disabled:opacity-50 min-h-[44px] w-full md:w-auto"
          disabled={accrualMutation.isPending || selectedYear !== currentYear}
          onClick={async () => {
            const ok = await confirm({
              title: t("leave_balances.run_accrual"),
              message: t("leave_balances.confirm_accrual"),
              variant: "warning"
            });
            if (ok) accrualMutation.mutate();
          }}
          title={selectedYear !== currentYear ? t("leave_balances.accrual_current_year_only") : undefined}
        >
          <PlayCircle size={18} /> <span className="whitespace-nowrap">{accrualMutation.isPending ? t("common.processing").toUpperCase() : t("leave_balances.run_accrual").toUpperCase()}</span>
        </button>
      </div>

      <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-3xl md:rounded-[3rem] overflow-hidden shadow-sm animate-in slide-in-from-bottom-8 duration-700">
        <div className="p-5 md:p-8 border-b border-slate-50 dark:border-slate-800 flex flex-col md:flex-row md:items-center justify-between gap-5 md:gap-6 bg-slate-50 dark:bg-slate-800/30">
          <div className="flex items-center gap-3">
            <Filter className="w-5 h-5 text-primary-600 dark:text-primary-400" />
            <h2 className="text-lg md:text-xl font-bold text-slate-800 dark:text-slate-100 tracking-tight uppercase">{t("leave_balances.summary_table")}</h2>
            {totalItems > 0 && (
              <span className="px-2 py-0.5 bg-slate-200 dark:bg-slate-600 text-slate-600 dark:text-slate-300 rounded-full text-xs font-bold">
                {totalItems}
              </span>
            )}
            {/* Year Selector */}
            <div className="flex items-center gap-2 ml-2">
              <Calendar className="w-4 h-4 text-slate-400 dark:text-slate-500" />
              <Combobox
                className="w-24 bg-white dark:bg-slate-900 border-slate-200 dark:border-slate-700 rounded-xl text-xs font-bold text-slate-700 dark:text-slate-300 uppercase tracking-wider focus:ring-2 focus:ring-primary-500/20 py-[5px]"
                onChange={(value) => {
                  if (value) {
                    setSelectedYear(Number(value));
                    setPage(1);
                  }
                }}
                options={yearOptions.map(y => ({ value: y.toString(), label: y.toString() }))}
                placeholder="Select Year"
                value={selectedYear.toString()}
              />
            </div>
          </div>

          <div className="relative group w-full md:w-80">
            <Search className="absolute left-4 md:left-5 top-1/2 -translate-y-1/2 text-slate-300 dark:text-slate-600 group-focus-within:text-primary-500 dark:group-focus-within:text-primary-400 transition-colors" size={18} />
            <input className="w-full pl-12 md:pl-14 pr-5 md:pr-6 py-3.5 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-bold text-sm tracking-tight shadow-sm min-h-[44px]" onChange={(e) => {
                setSearchTerm(e.target.value);
                setPage(1); // Reset to page 1 on search
              }} placeholder={t("common.search_placeholder")} type="text" value={searchTerm}
            />
          </div>
        </div>

        {/* Desktop Table View (≥1024px) - Simplified 5 Columns */}
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
              ) : (balances).map((balance, index) => (
                <DesktopBalanceRow
                  balance={balance}
                  key={`${balance.employee_id}-${balance.leave_type_id}-${index}`}
                  onViewDetails={() => { setSelectedBalance(balance); }}
                  onViewHistory={() => { setSelectedEmployee({ id: balance.employee_id, name: balance.employee_name, leaveTypeId: balance.leave_type_id, leaveTypeName: balance.leave_type_name }); }}
                />
              ))}
            </tbody>
          </table>
        </div>

        {/* Tablet/Mobile Card View (<1024px) - Simplified */}
        <div className="lg:hidden divide-y divide-slate-100 dark:divide-slate-800">
          {isLoading && !balances.length ? (
            <div className="px-5 py-20 text-center text-slate-500 dark:text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50 animate-pulse">{t("common.loading")}</div>
          ) : balances.length === 0 ? (
            <div className="px-5 py-32 text-center text-slate-500 dark:text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50">{t("attendance_logs.messages.no_data")}</div>
          ) : (balances).map((balance, index) => (
            <MobileBalanceCard
              balance={balance}
              key={`${balance.employee_id}-${balance.leave_type_id}-${index}`}
              onViewDetails={() => { setSelectedBalance(balance); }}
              onViewHistory={() => { setSelectedEmployee({ id: balance.employee_id, name: balance.employee_name, leaveTypeId: balance.leave_type_id, leaveTypeName: balance.leave_type_name }); }}
            />
          ))}
        </div>

        {/* Pagination Controls */}
        {totalItems > 0 && (
          <div className="p-5 md:p-8 border-t border-slate-50 dark:border-slate-800 bg-slate-50 dark:bg-slate-800/30 flex flex-col md:flex-row items-center justify-between gap-4">
            <div className="text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-wider">
              {t("common.showing")} <span className="text-slate-600 dark:text-slate-300">{(page - 1) * limit + 1}</span> - <span className="text-slate-600 dark:text-slate-300">{Math.min(page * limit, totalItems)}</span> {t("common.of")} <span className="text-slate-600 dark:text-slate-300">{totalItems}</span>
            </div>

            <div className="flex items-center gap-2">
              <button
                className="p-2 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-xl hover:bg-slate-50 dark:hover:bg-slate-800 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                disabled={page === 1}
                onClick={() => { setPage(p => Math.max(1, p - 1)); }}
              >
                <ChevronLeft className="w-4 h-4 text-slate-600 dark:text-slate-300" />
              </button>

              <div className="flex items-center gap-1">
                {Array.from({ length: Math.min(5, totalPages) }, (_, index) => {
                  // Simple logic to show window around current page
                  let p = index + 1;
                  if (totalPages > 5) {
                    if (page > 3) p = page - 2 + index;
                    if (p > totalPages) p = totalPages - (4 - index);
                  }

                  return (
                    <button
                      className={cn(
                        "w-8 h-8 flex items-center justify-center rounded-xl text-xs font-bold transition-all",
                        page === p
                          ? "bg-primary-600 text-white shadow-md shadow-primary-100"
                          : "bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 text-slate-600 dark:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-800"
                      )}
                      key={p}
                      onClick={() => { setPage(p); }}
                    >
                      {p}
                    </button>
                  );
                })}
              </div>

              <button
                className="p-2 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-xl hover:bg-slate-50 dark:hover:bg-slate-800 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                disabled={page === totalPages}
                onClick={() => { setPage(p => Math.min(totalPages, p + 1)); }}
              >
                <ChevronRight className="w-4 h-4 text-slate-600 dark:text-slate-300" />
              </button>
            </div>
          </div>
        )}
      </div>

      {Boolean(selectedEmployee && wsId) && (
        <LeaveHistoryModal
          employeeId={selectedEmployee?.id ?? ""}
          employeeName={selectedEmployee?.name ?? ""}
          isOpen={Boolean(selectedEmployee)}
          leaveTypeId={selectedEmployee?.leaveTypeId}
          leaveTypeName={selectedEmployee?.leaveTypeName}
          onClose={() => { setSelectedEmployee(null); }}
          wsId={wsId ?? ""}
        />
      )}

      {!!selectedBalance && <LeaveBalanceDetailModal balance={selectedBalance} isOpen={!!selectedBalance} onClose={() => { setSelectedBalance(null); }} />}

      <ConfirmationDialog />
    </div>
  );
}

// Desktop table row component (≥1024px) - Simplified to essential info only
function DesktopBalanceRow({ balance, onViewDetails, onViewHistory }: {
  balance: WorkspaceLeaveBalance;
  onViewDetails: () => void;
  onViewHistory: () => void;
}) {
  const { t } = useTranslation();
  const totalEntitled = balance.entitlement + balance.accrued_amount + balance.carried_over;
  const usagePercent = Math.round((balance.used / (totalEntitled || 1)) * 100);

  return (
    <tr className="hover:bg-slate-50 dark:hover:bg-slate-800/50 transition-all group cursor-pointer" onClick={onViewDetails}>
      {/* Employee Name with Avatar */}
      <td className="px-6 py-6">
        <div className="flex items-center gap-4">
          <div className="w-12 h-12 bg-primary-50 dark:bg-primary-950 rounded-xl flex items-center justify-center text-primary-600 dark:text-primary-400 font-bold text-sm shadow-inner border border-primary-100 dark:border-primary-800/50 uppercase group-hover:scale-110 transition-transform flex-shrink-0">
            {balance.employee_name.split(' ').map(n => n[0]).join('').slice(0, 2)}
          </div>
          <div className="font-bold text-slate-800 dark:text-slate-100 text-base tracking-tight uppercase leading-tight">
            {balance.employee_name}
          </div>
        </div>
      </td>

      {/* Leave Type Badge */}
      <td className="px-5 py-6">
        <span className="px-3 py-1.5 bg-slate-50 dark:bg-slate-800 text-slate-600 dark:text-slate-300 rounded-xl text-xs font-bold uppercase tracking-wider border border-slate-200 dark:border-slate-700 shadow-sm inline-block">
          {balance.leave_type_name}
        </span>
      </td>

      {/* Remaining Balance - Hero Metric */}
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

      {/* Usage Progress Bar */}
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

      {/* Action Buttons */}
      <td className="px-6 py-6 text-right" onClick={(e) => { e.stopPropagation(); }}>
        <div className="flex items-center justify-end gap-2 opacity-60 group-hover:opacity-100 transition-all">
          <button
            className="p-2.5 bg-primary-50 dark:bg-primary-950 text-primary-600 dark:text-primary-400 hover:bg-primary-100 dark:bg-primary-900 rounded-xl border border-primary-200 dark:border-primary-800 transition-all active:scale-90 min-w-[44px] min-h-[44px]"
            onClick={onViewDetails}
            title={t("leave_balances.view_details")}
          >
            <Eye className="h-5 w-5" />
          </button>
          <button
            className="p-2.5 bg-slate-50 dark:bg-slate-800 text-slate-500 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-slate-700 rounded-xl border border-slate-200 dark:border-slate-700 transition-all active:scale-90 min-w-[44px] min-h-[44px]"
            onClick={onViewHistory}
            title={t("leave_balances.table.history_tooltip")}
          >
            <History className="h-5 w-5" />
          </button>
        </div>
      </td>
    </tr>
  );
}

// Mobile card component (<1024px) - Simplified, essential info only
function MobileBalanceCard({ balance, onViewDetails, onViewHistory }: {
  balance: WorkspaceLeaveBalance;
  onViewDetails: () => void;
  onViewHistory: () => void;
}) {
  const { t } = useTranslation();
  const totalEntitled = balance.entitlement + balance.accrued_amount + balance.carried_over;
  const usagePercent = Math.round((balance.used / (totalEntitled || 1)) * 100);

  return (
    <div className="p-5 space-y-5 bg-white dark:bg-slate-900 hover:bg-slate-50 dark:hover:bg-slate-800/30 transition-all">
      {/* Header: Employee Avatar + Name + Leave Type */}
      <div className="flex items-center gap-3">
        <div className="w-14 h-14 bg-primary-50 dark:bg-primary-950 rounded-2xl flex items-center justify-center text-primary-600 dark:text-primary-400 font-bold text-base shadow-inner border border-primary-100 dark:border-primary-800/50 uppercase flex-shrink-0">
          {balance.employee_name.split(' ').map(n => n[0]).join('').slice(0, 2)}
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

      {/* Hero Metric: Remaining Balance */}
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

      {/* Usage Progress Bar */}
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

      {/* Action Buttons */}
      <div className="flex items-center gap-3 pt-3 border-t border-slate-100 dark:border-slate-800">
        <button
          className="flex-1 flex items-center justify-center gap-2 px-4 py-3.5 bg-primary-600 hover:bg-primary-700 text-white font-bold uppercase tracking-wider text-xs rounded-xl transition-all active:scale-95 shadow-lg shadow-primary-200 min-h-[44px]"
          onClick={onViewDetails}
        >
          <Eye className="h-5 w-5" />
          <span>{t("leave_balances.view_details")}</span>
        </button>
        <button
          className="p-3.5 bg-slate-100 dark:bg-slate-700 hover:bg-slate-200 dark:bg-slate-600 text-slate-600 dark:text-slate-300 rounded-xl transition-all active:scale-95 min-w-[44px] min-h-[44px] flex-shrink-0"
          onClick={onViewHistory}
          title={t("leave_balances.table.history_tooltip")}
        >
          <History className="h-5 w-5" />
        </button>
      </div>
    </div>
  );
}