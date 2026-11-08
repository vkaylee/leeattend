import React from "react";

import { LeaveBalanceDetailModal } from "@/components/leave/leave-balance-detail-modal";
import { LeaveHistoryModal } from "@/components/leave/leave-history-modal";
import { OrganizationLeaveBalance } from "@/lib/api/types";

interface LeaveBalancesModalsProps {
  selectedBalance: OrganizationLeaveBalance | null;
  selectedEmployee: { id: string, name: string, leaveTypeId?: string, leaveTypeName?: string } | null;
  setSelectedBalance: (balance: OrganizationLeaveBalance | null) => void;
  setSelectedEmployee: (employee: { id: string, name: string, leaveTypeId?: string, leaveTypeName?: string } | null) => void;
}

export const LeaveBalancesModals: React.FC<LeaveBalancesModalsProps> = ({
  selectedBalance,
  selectedEmployee,
  setSelectedBalance,
  setSelectedEmployee,
}) => {
  return (
    <>
      {Boolean(selectedEmployee) && (
        <LeaveHistoryModal employeeId={selectedEmployee?.id ?? ""}
        employeeName={selectedEmployee?.name ?? ""}
        isOpen={Boolean(selectedEmployee)}
        leaveTypeId={selectedEmployee?.leaveTypeId}
        leaveTypeName={selectedEmployee?.leaveTypeName}
        onClose={() => { setSelectedEmployee(null); }}  />
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
