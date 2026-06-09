import React from "react";

import AdjustmentDrawer from "@/components/payroll/adjustment-drawer";
import BatchCalculationModal from "@/components/payroll/batch-calculation-modal";
import PayrollDetailDrawer from "@/components/payroll/payroll-detail-drawer";
import { Employee } from "@/lib/api";
import { AttendanceAdjustmentRequest, Payslip } from "@/lib/api/types";

interface PayrollModalsProps {
  adjustments: AttendanceAdjustmentRequest[];
  employee: Employee | null;
  employeeNames: Record<string, string>;
  isAdjustmentDrawerOpen: boolean;
  isBatchCalcModalOpen: boolean;
  isDetailDrawerOpen: boolean;
  isLoadingPayslips: boolean;
  onAdjustmentResolved: () => void;
  onCloseAdjustmentDrawer: () => void;
  onCloseBatchCalcModal: () => void;
  onCloseDetailDrawer: () => void;
  onOpenAdjustmentDrawer: () => void;
  payslips: Payslip[];
  selectedIds: Set<string>;
  workspaceId: string;
}

export const PayrollModals: React.FC<PayrollModalsProps> = ({
  adjustments,
  employee,
  employeeNames,
  isAdjustmentDrawerOpen,
  isBatchCalcModalOpen,
  isDetailDrawerOpen,
  isLoadingPayslips,
  onAdjustmentResolved,
  onCloseAdjustmentDrawer,
  onCloseBatchCalcModal,
  onCloseDetailDrawer,
  onOpenAdjustmentDrawer,
  payslips,
  selectedIds,
  workspaceId,
}) => {
  return (
    <>
      <PayrollDetailDrawer 
        adjustments={adjustments}
        employee={employee}
        isLoadingPayslips={isLoadingPayslips}
        isOpen={isDetailDrawerOpen}
        onClose={onCloseDetailDrawer}
        onResolveAdjustments={onOpenAdjustmentDrawer}
        payslips={payslips}
        workspaceId={workspaceId}
      />

      {!!employee && (
        <AdjustmentDrawer
          employeeId={employee.id}
          employeeName={employee.full_name}
          employmentId={employee.active_employment_id ?? ""}
          isOpen={isAdjustmentDrawerOpen}
          onAdjustmentResolved={onAdjustmentResolved}
          onClose={onCloseAdjustmentDrawer}
        />
      )}

      <BatchCalculationModal
        employeeIds={Array.from(selectedIds)}
        employeeNames={employeeNames}
        isOpen={isBatchCalcModalOpen}
        onClose={onCloseBatchCalcModal}
        workspaceId={workspaceId}
      />
    </>
  );
};
