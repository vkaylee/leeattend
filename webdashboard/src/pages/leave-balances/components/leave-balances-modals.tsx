import React from "react";

import { LeaveBalanceDetailModal } from "@/components/leave/leave-balance-detail-modal";
import { LeaveHistoryModal } from "@/components/leave/leave-history-modal";
import { WorkspaceLeaveBalance } from "@/lib/api/types";

interface LeaveBalancesModalsProps {
  selectedBalance: WorkspaceLeaveBalance | null;
  selectedEmployee: { id: string, name: string, leaveTypeId?: string, leaveTypeName?: string } | null;
  setSelectedBalance: (balance: WorkspaceLeaveBalance | null) => void;
  setSelectedEmployee: (employee: { id: string, name: string, leaveTypeId?: string, leaveTypeName?: string } | null) => void;
  wsId: string;
}

export const LeaveBalancesModals: React.FC<LeaveBalancesModalsProps> = ({
  selectedBalance,
  selectedEmployee,
  setSelectedBalance,
  setSelectedEmployee,
  wsId,
}) => {
  return (
    <>
      {Boolean(selectedEmployee && wsId) && (
        <LeaveHistoryModal
          employeeId={selectedEmployee?.id ?? ""}
          employeeName={selectedEmployee?.name ?? ""}
          isOpen={Boolean(selectedEmployee)}
          leaveTypeId={selectedEmployee?.leaveTypeId}
          leaveTypeName={selectedEmployee?.leaveTypeName}
          onClose={() => { setSelectedEmployee(null); }}
          wsId={wsId}
        />
      )}

      {!!selectedBalance && (
        <LeaveBalanceDetailModal 
          balance={selectedBalance} 
          isOpen={!!selectedBalance} 
          onClose={() => { setSelectedBalance(null); }} 
        />
      )}
    </>
  );
};
