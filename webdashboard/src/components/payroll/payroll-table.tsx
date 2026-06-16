import { History } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import { ErrorBoundary } from "@/components/common/error-boundary";
import { Typography } from "@/components/ui/typography";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { useMyPermissions } from "@/hooks/use-permissions";
import { Employee, Workspace, Payslip, AttendanceAdjustmentRequest } from "@/lib/api/types";

import { PayslipCard } from "./payslip-card";
import PayslipModal from "./payslip-modal";

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

  if (isLoading) {
    return (
      <div className="flex flex-col gap-6">
        <PayslipCardSkeleton />
        <PayslipCardSkeleton />
        <PayslipCardSkeleton />
      </div>
    );
  }

  if (payslips.length === 0) {
    return (
      <div className="py-24 text-center border-2 border-dashed border-slate-100 dark:border-slate-800 rounded-[2.5rem] bg-slate-50 dark:bg-slate-800/30 flex flex-col items-center">
        <div className="w-20 h-20 bg-white dark:bg-slate-900 rounded-full flex items-center justify-center shadow-sm dark:shadow-black/10 mb-6 border border-slate-100 dark:border-slate-800">
          <History className="text-slate-200 dark:text-slate-600" size={32} />
        </div>
        <Typography className="text-[10px] font-bold text-slate-300 dark:text-slate-600 uppercase tracking-widest" variant="p">{t("common.payroll.table.no_payslips")}</Typography>
      </div>
    );
  }

  return (
    <ErrorBoundary>
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
      </div>
    </ErrorBoundary>
  );
}