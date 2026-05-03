import React from "react";
import { useTranslation } from "react-i18next";

import AttendanceImportDialog from "@/components/features/attendance/attendance-import-dialog";
import EmployeeFormModal from "@/components/features/employees/employee-form-modal";
import EmploymentHistoryModal from "@/components/features/employees/employment-history-modal";
import RehireEmployeeModal from "@/components/features/employees/rehire-employee-modal";
import TransferEmployeeModal from "@/components/features/employees/transfer-employee-modal";

import type { Employee, Department, EmploymentPeriod } from "../../../lib/api/types";

interface EmployeeModalsProps {
  departments: Department[] | undefined;
  editingEmployee: Employee | null;
  employmentHistory: EmploymentPeriod[] | undefined;
  formatDate: (date: string) => string;
  isHistoryModalOpen: boolean;
  isImportDialogOpen: boolean;
  isModalOpen: boolean;
  isRehireModalOpen: boolean;
  isTransferModalOpen: boolean;
  prefillEnrollmentId: string;
  selectedDeptId: string;
  selectedEmployee: Employee | null;
  setIsHistoryModalOpen: (open: boolean) => void;
  setIsImportDialogOpen: (open: boolean) => void;
  setIsModalOpen: (open: boolean) => void;
  setIsRehireModalOpen: (open: boolean) => void;
  setIsTransferModalOpen: (open: boolean) => void;
  setPrefillEnrollmentId: (id: string) => void;
  setSelectedDeptId: (id: string) => void;
  setSelectedEmployee: (emp: Employee | null) => void;
  wsId: string;
}

export function EmployeeModals(props: EmployeeModalsProps) {
  const { t } = useTranslation();
  const {
    departments,
    editingEmployee,
    employmentHistory,
    formatDate,
    isHistoryModalOpen,
    isImportDialogOpen,
    isModalOpen,
    isRehireModalOpen,
    isTransferModalOpen,
    prefillEnrollmentId,
    selectedDeptId,
    selectedEmployee,
    setIsHistoryModalOpen,
    setIsImportDialogOpen,
    setIsModalOpen,
    setIsRehireModalOpen,
    setIsTransferModalOpen,
    setPrefillEnrollmentId,
    setSelectedDeptId,
    setSelectedEmployee,
    wsId,
  } = props;

  return (
    <>
      {/* Main Employee Form Modal */}
      <EmployeeFormModal
        editingEmployee={editingEmployee}
        isOpen={isModalOpen}
        onClose={() => { setIsModalOpen(false); }}
        onSuccess={() => { setPrefillEnrollmentId(""); }}
        prefillEnrollmentId={prefillEnrollmentId}
        wsId={wsId}
      />

      {/* Transfer Modal */}
      <TransferEmployeeModal
        departments={departments}
        employeeId={selectedEmployee?.id ?? null}
        isOpen={isTransferModalOpen}
        onClose={() => { setIsTransferModalOpen(false); }}
        selectedDeptId={selectedDeptId}
        setSelectedDeptId={setSelectedDeptId}
        wsId={wsId}
      />

      {/* History Modal */}
      <EmploymentHistoryModal
        employmentHistory={employmentHistory}
        formatDate={formatDate}
        isOpen={isHistoryModalOpen}
        onClose={() => { setIsHistoryModalOpen(false); }}
      />

      {/* Rehire Modal */}
      <RehireEmployeeModal
        employeeId={selectedEmployee?.id ?? null}
        isOpen={isRehireModalOpen}
        onClose={() => { setIsRehireModalOpen(false); }}
        onSuccess={(updatedEmp) => { setSelectedEmployee(updatedEmp); }}
        wsId={wsId}
      />

      <AttendanceImportDialog
        customRequirements={[
          t("employees.import_req.col_a"),
          t("employees.import_req.col_b"),
          t("employees.import_req.col_c"),
          t("employees.import_req.col_d"),
        ]}
        endpoint="/employees/upload"
        isOpen={isImportDialogOpen}
        onClose={() => { setIsImportDialogOpen(false); }}
        targetType="employee"
        title={t("employees.bulk_import_title")}
      />
    </>
  );
}
