import { History, Calendar, ChevronDown, FileText } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import { ErrorBoundary } from "@/components/common/error-boundary";
import ResponsiveTable from "@/components/ui/responsive-table";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { useMyPermissions } from "@/hooks/use-permissions";
import { Employee, Workspace, Payslip, AttendanceAdjustmentRequest } from "@/lib/api/types";
import { cn } from "@/lib/utils";

import { PayslipCard } from "./payslip-card";
import PayslipModal from "./payslip-modal";

// ── Skeleton ───────────────────────────────────────────────────────

function PayslipCardSkeleton() {
  return (
    <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-[2rem] shadow-sm p-8 flex items-center justify-between animate-pulse">
      <div className="flex items-center gap-8">
        <div className="w-14 h-14 rounded-2xl bg-slate-200 dark:bg-slate-800" />
        <div className="space-y-3">
          <div className="h-4 w-32 bg-slate-200 dark:bg-slate-800 rounded" />
          <div className="flex items-center gap-3">
            <div className="h-5 w-16 bg-slate-200 dark:bg-slate-800 rounded-full" />
            <div className="h-3 w-24 bg-slate-200 dark:bg-slate-800 rounded" />
          </div>
        </div>
      </div>
      <div className="flex items-center gap-10 pr-2">
        <div className="text-right space-y-2">
          <div className="h-2 w-16 bg-slate-200 dark:bg-slate-800 rounded ml-auto" />
          <div className="h-6 w-24 bg-slate-200 dark:bg-slate-800 rounded" />
        </div>
        <div className="w-10 h-10 rounded-xl bg-slate-200 dark:bg-slate-800" />
      </div>
    </div>
  );
}

// ── Mobile Card ────────────────────────────────────────────────────

interface PayrollMobileCardProps {
  canReadSensitive: boolean;
  formatMonthYear: (date: string) => string;
  isExpanded: boolean;
  setExpandedId: (id: string | null) => void;
  setIsModalOpen: (open: boolean) => void;
  setSelectedPayslip: (slip: Payslip) => void;
  slip: Payslip;
  t: (key: string) => string;
}

function PayrollMobileCard({
  slip,
  isExpanded,
  setExpandedId,
  canReadSensitive,
  formatMonthYear,
  setSelectedPayslip,
  setIsModalOpen,
  t,
}: PayrollMobileCardProps) {
  return (
    <div
      className={cn(
        "rounded-2xl border bg-white shadow-sm transition-all dark:bg-slate-900",
        isExpanded
          ? "border-primary-300 ring-2 ring-primary-500/20 dark:border-primary-700"
          : "border-slate-100 dark:border-slate-800",
      )}
    >
      {/* Header tap target */}
      <div
        className="flex items-center gap-4 p-5 active:bg-slate-50 dark:active:bg-slate-800/50"
        data-testid={`toggle-card-${slip.id}`}
        onClick={() => { setExpandedId(isExpanded ? null : slip.id); }}
        onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); setExpandedId(isExpanded ? null : slip.id); } }}
        role="button"
        tabIndex={0}
      >
        <div className={cn(
          "flex h-12 w-12 shrink-0 items-center justify-center rounded-xl border font-bold uppercase transition-all",
          isExpanded
            ? "border-primary-500 bg-primary-600 text-white"
            : "border-slate-100 bg-slate-50 text-slate-500 dark:border-slate-700 dark:bg-slate-800 dark:text-slate-400",
        )}>
          <Calendar size={20} />
        </div>
        <div className="min-w-0 flex-1">
          <div className="truncate text-sm font-bold uppercase tracking-tight text-slate-800 dark:text-slate-100">
            {t("common.month")} {formatMonthYear(slip.period_start)}
          </div>
          <div className="mt-1 flex items-center gap-2">
            <span className={cn(
              "rounded-full border px-2 py-0.5 text-[9px] font-bold uppercase tracking-widest",
              slip.status === 'paid'
                ? "border-emerald-100 bg-emerald-50 text-emerald-600 dark:border-emerald-800 dark:bg-emerald-950 dark:text-emerald-300"
                : "border-blue-100 bg-blue-50 text-blue-600 dark:border-blue-800 dark:bg-blue-950 dark:text-blue-300",
            )}>
              {slip.status === 'paid' ? t("common.payroll.status.paid") : t("common.payroll.status.finalized")}
            </span>
            <span className="font-mono text-[10px] font-bold text-slate-400 dark:text-slate-500">
              {slip.standard_units}U · {slip.overtime_units}OT
            </span>
          </div>
        </div>
        <div className="flex items-center gap-3">
          <div className="text-right">
            <div className="text-[9px] font-bold uppercase tracking-widest text-slate-400 dark:text-slate-500">
              {t("common.payroll.table.net_pay")}
            </div>
            <div className="font-mono text-base font-bold text-primary-600 dark:text-primary-400">
              {!canReadSensitive ? "***" : new Intl.NumberFormat('vi-VN').format(parseFloat(slip.net_pay_amount))}
            </div>
          </div>
          <ChevronDown className={cn(
            "h-5 w-5 shrink-0 text-slate-300 transition-transform dark:text-slate-600",
            isExpanded && "rotate-180",
          )} />
        </div>
      </div>

      {/* Expanded detail (mobile) */}
      {isExpanded ? (
        <div className="animate-in fade-in slide-in-from-top-2 border-t border-slate-100 px-5 pb-5 pt-4 duration-300 dark:border-slate-800">
          {/* Quick stats */}
          <div className="grid grid-cols-2 gap-3">
            <div className="rounded-xl bg-slate-50 p-3 dark:bg-slate-800/60">
              <div className="text-[9px] font-bold uppercase tracking-wider text-slate-400">{t("common.payroll.table.base_pay")}</div>
              <div className="mt-1 font-mono text-sm font-bold text-slate-700 dark:text-slate-200">
                {!canReadSensitive ? "***" : new Intl.NumberFormat('vi-VN').format(parseFloat(slip.base_pay_amount || "0"))}
              </div>
            </div>
            <div className="rounded-xl bg-emerald-50 p-3 dark:bg-emerald-950/30">
              <div className="text-[9px] font-bold uppercase tracking-wider text-emerald-500">{t("common.payroll.table.overtime")}</div>
              <div className="mt-1 font-mono text-sm font-bold text-emerald-600 dark:text-emerald-400">
                {!canReadSensitive ? "***" : `+${new Intl.NumberFormat('vi-VN').format(parseFloat(slip.overtime_pay_amount || "0"))}`}
              </div>
            </div>
            <div className="rounded-xl bg-blue-50 p-3 dark:bg-blue-950/30">
              <div className="text-[9px] font-bold uppercase tracking-wider text-blue-500">{t("common.payroll.table.bonus")}</div>
              <div className="mt-1 font-mono text-sm font-bold text-blue-600 dark:text-blue-400">
                {!canReadSensitive ? "***" : `+${new Intl.NumberFormat('vi-VN').format(parseFloat(String(slip.incentives?.reduce((acc, inc) => acc + parseFloat(inc.amount), 0) ?? 0)))}`}
              </div>
            </div>
            <div className="rounded-xl bg-rose-50 p-3 dark:bg-rose-950/30">
              <div className="text-[9px] font-bold uppercase tracking-wider text-rose-500">{t("payroll.document.deductions")}</div>
              <div className="mt-1 font-mono text-sm font-bold text-rose-600 dark:text-rose-400">
                {!canReadSensitive ? "***" : `-${new Intl.NumberFormat('vi-VN').format(parseFloat(slip.deductions_amount || "0"))}`}
              </div>
            </div>
          </div>

          {/* Action button */}
          <button
            className="mt-4 flex w-full items-center justify-center gap-2 rounded-xl border border-slate-200 bg-slate-50 py-3 text-[10px] font-bold uppercase tracking-widest text-slate-600 transition-colors active:bg-slate-100 dark:border-slate-700 dark:bg-slate-800 dark:text-slate-400 dark:active:bg-slate-700"
            data-testid={`mobile-view-payslip-${slip.id}`}
            onClick={(e) => {
              e.stopPropagation();
              setSelectedPayslip(slip);
              setIsModalOpen(true);
            }}
            type="button"
          >
            <FileText size={14} />
            {t("payroll.document.official_payslip")}
          </button>
        </div>
      ) : null}
    </div>
  );
}

// ── Main Component ─────────────────────────────────────────────────

interface PayrollTableProps {
  adjustments: AttendanceAdjustmentRequest[];
  employee: Employee | null;
  isLoading: boolean;
  onResolveAdjustments: () => void;
  payslips: Payslip[];
  workspace: Workspace | null;
}

export default function PayrollTable({
  employee,
  workspace,
  payslips,
  isLoading,
  adjustments,
  onResolveAdjustments
}: PayrollTableProps) {
  const [selectedPayslip, setSelectedPayslip] = useState<Payslip | null>(null);
  const [isModalOpen, setIsModalOpen] = useState(false);
  const { t } = useTranslation();
  const { formatMonthYear } = useDateTimeFormatter();
  const [expandedId, setExpandedId] = useState<string | null>(null);
  const { hasPermission } = useMyPermissions();
  const canReadSensitive = hasPermission("payroll:sensitive:read");

  return (
    <ErrorBoundary>
      <ResponsiveTable<Payslip>
        data={payslips}
        emptyState={{
          icon: <History className="h-10 w-10" />,
          title: t("common.payroll.table.no_payslips"),
        }}
        getCardKey={(slip) => slip.id}
        isLoading={isLoading}
        renderCard={(slip) => (
          <PayrollMobileCard
            canReadSensitive={canReadSensitive}
            formatMonthYear={formatMonthYear}
            isExpanded={expandedId === slip.id}
            setExpandedId={setExpandedId}
            setIsModalOpen={setIsModalOpen}
            setSelectedPayslip={setSelectedPayslip}
            slip={slip}
            t={t}
          />
        )}
        renderDesktopTable={() => (
          <div className="flex flex-col gap-6 animate-in fade-in duration-700">
            {payslips.map((slip) => {
              const isExpanded = expandedId === slip.id;
              const slipAdjustments = adjustments.filter(adj =>
                adj.employment_id === employee?.active_employment_id &&
                adj.requested_date &&
                new Date(adj.requested_date) >= new Date(slip.period_start) &&
                new Date(adj.requested_date) <= new Date(slip.period_end)
              );

              return (
                <PayslipCard
                  canReadSensitive={canReadSensitive}
                  formatMonthYear={formatMonthYear}
                  isExpanded={isExpanded}
                  key={slip.id}
                  onResolveAdjustments={onResolveAdjustments}
                  setExpandedId={setExpandedId}
                  setIsModalOpen={setIsModalOpen}
                  setSelectedPayslip={setSelectedPayslip}
                  slip={slip}
                  slipAdjustments={slipAdjustments}
                  t={t}
                />
              );
            })}
          </div>
        )}
        renderSkeletonCard={() => <PayslipCardSkeleton />}
      />

      {employee && selectedPayslip ? (
        <PayslipModal
          employee={employee}
          isOpen={isModalOpen}
          onClose={() => {
            setIsModalOpen(false);
            setSelectedPayslip(null);
          }}
          payslip={selectedPayslip}
          workspace={workspace}
        />
      ) : null}
    </ErrorBoundary>
  );
}
