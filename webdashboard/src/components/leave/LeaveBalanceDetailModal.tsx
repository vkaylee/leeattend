import { X, Info, TrendingUp, Calendar, Award, FileText } from "lucide-react";
import { createPortal } from "react-dom";
import { useTranslation } from "react-i18next";
import { cn } from "@/lib/utils";
import { WorkspaceLeaveBalance } from "@/lib/api";

interface LeaveBalanceDetailModalProps {
  balance: WorkspaceLeaveBalance;
  isOpen: boolean;
  onClose: () => void;
}

export function LeaveBalanceDetailModal({ balance, isOpen, onClose }: LeaveBalanceDetailModalProps) {
  const { t } = useTranslation();

  if (!isOpen) return null;

  const totalEntitled = balance.entitlement + balance.accrued_amount + balance.carried_over;
  const usagePercent = Math.round((balance.used / (totalEntitled || 1)) * 100);

  return createPortal(
    <div
      className="fixed inset-0 z-[9999] flex items-center justify-center p-4 animate-in fade-in duration-300"
      onClick={onClose}
    >
      {/* Backdrop */}
      <div className="absolute inset-0 bg-slate-900/20 backdrop-blur-md" />

      {/* Modal Content */}
      <div
        className="relative bg-white rounded-3xl shadow-2xl max-w-2xl w-full max-h-[90vh] overflow-y-auto animate-in zoom-in-95 slide-in-from-bottom-8 duration-500"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="sticky top-0 bg-white border-b border-slate-100 px-6 md:px-8 py-5 md:py-6 rounded-t-3xl z-10">
          <div className="flex items-start justify-between gap-4">
            <div className="flex items-center gap-4 flex-1 min-w-0">
              <div className="w-14 h-14 bg-primary-50 rounded-2xl flex items-center justify-center text-primary-600 font-bold text-base shadow-inner border border-primary-100/50 uppercase flex-shrink-0">
                {balance.employee_name.split(' ').map(n => n[0]).join('').slice(0, 2)}
              </div>
              <div className="flex-1 min-w-0">
                <h2 className="text-xl md:text-2xl font-bold text-slate-800 tracking-tight uppercase leading-tight truncate">
                  {balance.employee_name}
                </h2>
                <div className="flex items-center gap-2 mt-2">
                  <span className="inline-block px-3 py-1.5 bg-slate-50 text-slate-600 rounded-xl text-xs font-bold uppercase tracking-wider border border-slate-200 shadow-sm">
                    {balance.leave_type_name}
                  </span>
                </div>
              </div>
            </div>
            <button
              onClick={onClose}
              className="p-2.5 text-slate-500 hover:text-slate-600 hover:bg-slate-50 rounded-xl transition-all active:scale-90 flex-shrink-0 min-w-[44px] min-h-[44px]"
              aria-label={t("common.close")}
            >
              <X size={24} />
            </button>
          </div>
        </div>

        {/* Body */}
        <div className="p-6 md:p-8 space-y-8">
          {/* Hero Metric: Remaining Balance */}
          <div className="bg-gradient-to-br from-primary-50 via-primary-50/50 to-transparent rounded-3xl p-8 border border-primary-100/50 shadow-inner">
            <div className="flex items-center gap-3 mb-3">
              <Award className="w-6 h-6 text-primary-600" />
              <div className="text-xs font-bold text-primary-600 uppercase tracking-[0.2em]">
                {t("leave_balances.table.remaining")}
              </div>
            </div>
            <div className={cn(
              "text-6xl md:text-7xl font-bold font-mono tracking-tighter",
              balance.remaining > 0 ? "text-emerald-600" : "text-slate-500"
            )}>
              {balance.remaining}
              <span className="text-2xl md:text-3xl ml-2 text-slate-500 font-bold uppercase tracking-tight">
                {t("common.days_short", "days")}
              </span>
            </div>

            {/* Usage Bar */}
            <div className="mt-6 space-y-3">
              <div className="flex items-center justify-between text-xs font-bold text-slate-600 uppercase tracking-wider">
                <span>{t("leave_balances.table.usage")}</span>
                <span className="font-mono">{usagePercent}% {t("leave_balances.detail.used", "used")}</span>
              </div>
              <div className="w-full h-4 bg-white rounded-full overflow-hidden shadow-inner border border-primary-100">
                <div
                  className={cn(
                    "h-full rounded-full transition-all duration-1000 shadow-lg",
                    usagePercent > 80 ? "bg-rose-500 shadow-rose-200" :
                    usagePercent > 50 ? "bg-amber-500 shadow-amber-200" :
                    "bg-primary-600 shadow-primary-200"
                  )}
                  style={{ width: `${Math.min(100, usagePercent)}%` }}
                />
              </div>
            </div>
          </div>

          {/* Breakdown Section */}
          <div className="space-y-4">
            <div className="flex items-center gap-3 pb-3 border-b border-slate-100">
              <TrendingUp className="w-5 h-5 text-slate-600" />
              <h3 className="text-sm font-bold text-slate-600 uppercase tracking-[0.15em]">
                {t("leave_balances.detail.breakdown", "Balance Breakdown")}
              </h3>
            </div>

            <div className="grid grid-cols-2 gap-4">
              {/* Base Entitlement */}
              <div className="bg-slate-50 rounded-2xl p-5 border border-slate-100">
                <div className="flex items-center gap-2 mb-2">
                  <Calendar className="w-4 h-4 text-slate-500" />
                  <div className="text-[10px] font-bold text-slate-500 uppercase tracking-wider">
                    {t("leave_balances.table.base")}
                  </div>
                </div>
                <div className="text-3xl font-bold font-mono text-slate-600">
                  {balance.entitlement}
                </div>
              </div>

              {/* Accrued Amount */}
              <div className="bg-primary-50 rounded-2xl p-5 border border-primary-100 relative group">
                <div className="flex items-center gap-2 mb-2">
                  <TrendingUp className="w-4 h-4 text-primary-400" />
                  <div className="text-[10px] font-bold text-primary-600 uppercase tracking-wider flex items-center gap-1">
                    {t("leave_balances.table.accrued")}
                    {balance.accrued_amount > 0 && (
                      <Info className="h-3.5 w-3.5 text-primary-300 cursor-help" />
                    )}
                  </div>
                </div>
                <div className="text-3xl font-bold font-mono text-primary-600">
                  +{balance.accrued_amount}
                </div>

                {/* Accrual Info Tooltip */}
                {balance.accrued_amount > 0 && (
                  <div className="absolute top-full left-0 mt-2 w-full p-4 bg-primary-600 text-white text-[10px] font-medium leading-relaxed rounded-2xl opacity-0 group-hover:opacity-100 transition-all duration-300 pointer-events-none z-20 shadow-2xl border border-white/10">
                    <div className="flex items-center gap-2 mb-2 text-primary-400">
                      <Info size={12} />
                      <span className="uppercase tracking-wider font-bold">
                        {t("leave_balances.detail.accrual_info_title", "Accrual Info")}
                      </span>
                    </div>
                    {t("leave_balances.table.accrued_info")}
                  </div>
                )}
              </div>

              {/* Carried Over */}
              <div className="bg-amber-50 rounded-2xl p-5 border border-amber-100">
                <div className="flex items-center gap-2 mb-2">
                  <FileText className="w-4 h-4 text-amber-400" />
                  <div className="text-[10px] font-bold text-amber-600 uppercase tracking-wider">
                    {t("leave_balances.table.carried_over")}
                  </div>
                </div>
                <div className="text-3xl font-bold font-mono text-amber-600">
                  +{balance.carried_over}
                </div>
              </div>

              {/* Used */}
              <div className="bg-rose-50 rounded-2xl p-5 border border-rose-100">
                <div className="flex items-center gap-2 mb-2">
                  <Calendar className="w-4 h-4 text-rose-400" />
                  <div className="text-[10px] font-bold text-rose-600 uppercase tracking-wider">
                    {t("leave_balances.table.used")}
                  </div>
                </div>
                <div className="text-3xl font-bold font-mono text-rose-600">
                  -{balance.used}
                </div>
              </div>
            </div>

            {/* Calculation Formula */}
            <div className="bg-slate-50 rounded-2xl p-5 border border-slate-200 mt-6">
              <div className="text-xs font-bold text-slate-500 uppercase tracking-wider mb-3">
                {t("leave_balances.detail.calculation", "Calculation")}
              </div>
              <div className="flex flex-wrap items-center gap-2 text-sm font-bold font-mono text-slate-600">
                <span className="px-3 py-1.5 bg-white rounded-lg border border-slate-200">
                  {balance.entitlement}
                </span>
                <span className="text-slate-500">+</span>
                <span className="px-3 py-1.5 bg-white rounded-lg border border-slate-200 text-primary-600">
                  {balance.accrued_amount}
                </span>
                <span className="text-slate-500">+</span>
                <span className="px-3 py-1.5 bg-white rounded-lg border border-slate-200 text-amber-600">
                  {balance.carried_over}
                </span>
                <span className="text-slate-500">-</span>
                <span className="px-3 py-1.5 bg-white rounded-lg border border-slate-200 text-rose-600">
                  {balance.used}
                </span>
                <span className="text-slate-500">=</span>
                <span className={cn(
                  "px-4 py-1.5 rounded-lg border-2 font-bold text-base",
                  balance.remaining > 0
                    ? "bg-emerald-50 border-emerald-200 text-emerald-600"
                    : "bg-slate-100 border-slate-200 text-slate-500"
                )}>
                  {balance.remaining}
                </span>
              </div>
            </div>
          </div>

          {/* Policy Information */}
          {balance.policy_name && (
            <div className="space-y-4">
              <div className="flex items-center gap-3 pb-3 border-b border-slate-100">
                <FileText className="w-5 h-5 text-slate-600" />
                <h3 className="text-sm font-bold text-slate-600 uppercase tracking-[0.15em]">
                  {t("leave_balances.detail.policy_info", "Policy Information")}
                </h3>
              </div>

              <div className="bg-slate-50 rounded-2xl p-5 border border-slate-100">
                <div className="flex items-center justify-between gap-4 flex-wrap">
                  <div>
                    <div className="text-[10px] font-bold text-slate-500 uppercase tracking-wider mb-2">
                      {t("leave_balances.table.policy")}
                    </div>
                    <div className="text-lg font-bold text-slate-800 uppercase tracking-tight">
                      {balance.policy_name}
                    </div>
                  </div>

                  <span className={cn(
                    "px-4 py-2 rounded-xl text-xs font-bold uppercase tracking-wider border shadow-sm",
                    balance.assignment_type === 'employee' ? "bg-primary-50 text-primary-600 border-primary-200" :
                    balance.assignment_type === 'department' ? "bg-amber-50 text-amber-600 border-amber-200" :
                    "bg-slate-50 text-slate-500 border-slate-200"
                  )}>
                    {t(`leave_balances.assignment_types.${balance.assignment_type}`)}
                  </span>
                </div>
              </div>
            </div>
          )}
        </div>

        {/* Footer Actions (Optional - for future enhancements) */}
        <div className="sticky bottom-0 bg-slate-50 border-t border-slate-100 px-6 md:px-8 py-4 rounded-b-3xl">
          <div className="flex items-center justify-end gap-3">
            <button
              onClick={onClose}
              className="px-6 py-3 bg-white text-slate-600 font-bold uppercase tracking-wider text-xs rounded-xl border border-slate-200 hover:bg-slate-100 transition-all active:scale-95 min-h-[44px]"
            >
              {t("common.close")}
            </button>
          </div>
        </div>
      </div>
    </div>,
    document.body
  );
}
