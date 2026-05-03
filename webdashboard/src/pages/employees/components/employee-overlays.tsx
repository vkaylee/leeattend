import React from "react";
import { useTranslation } from "react-i18next";

import AttendanceImportDialog from "@/components/features/attendance/attendance-import-dialog";
import EmployeeDetailDrawer from "@/components/features/employees/employee-detail-drawer";
import EmployeeFormModal from "@/components/features/employees/employee-form-modal";
import EmploymentHistoryModal from "@/components/features/employees/employment-history-modal";
import RehireEmployeeModal from "@/components/features/employees/rehire-employee-modal";
import TransferEmployeeModal from "@/components/features/employees/transfer-employee-modal";

import type { Employee, Department, EmploymentPeriod, EmployeeDepartmentHistory } from "../../../lib/api/types";

interface EmployeeOverlaysProps {
  departments: Department[] | undefined;
  deptHistory: EmployeeDepartmentHistory[] | undefined;
  editingEmployee: Employee | null;
  employmentHistory: EmploymentPeriod[] | undefined;
  formatDate: (date: string) => string;
  handleDrawerClose: () => void;
  handleTerminate: (emp: Employee) => Promise<void>;
  isHistoryModalOpen: boolean;
  isImportDialogOpen: boolean;
  isModalOpen: boolean;
  isRehireModalOpen: boolean;
  isTransferModalOpen: boolean;
  prefillEnrollmentId: string;
  selectedDeptId: string;
  selectedEmployee: Employee | null;
  setEditingEmployee: (emp: Employee | null) => void;
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

export function EmployeeOverlays(props: EmployeeOverlaysProps) {
  const { t } = useTranslation();
  const {
    departments,
    deptHistory,
    editingEmployee,
    employmentHistory,
    formatDate,
    handleDrawerClose,
    handleTerminate,
    isHistoryModalOpen,
    isImportDialogOpen,
    isModalOpen,
    isRehireModalOpen,
    isTransferModalOpen,
    prefillEnrollmentId,
    selectedDeptId,
    selectedEmployee,
    setEditingEmployee,
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
      <EmployeeDetailDrawer
        departments={departments}
        deptHistory={deptHistory}
        employee={selectedEmployee}
        employmentHistory={employmentHistory}
        formatDate={formatDate}
        isOpen={!!selectedEmployee}
        key={selectedEmployee?.id ?? "closed"}
        onClose={handleDrawerClose}
        onEdit={(emp) => {
          setEditingEmployee(emp);
          setIsModalOpen(true);
        }}
        onRehire={() => { setIsRehireModalOpen(true); }}
        onTerminate={handleTerminate}
        onTransfer={(deptId) => {
          setSelectedDeptId(deptId);
          setIsTransferModalOpen(true);
        }}
        wsId={wsId}
      />

      <EmployeeFormModal
        editingEmployee={editingEmployee}
        isOpen={isModalOpen}
        onClose={() => { setIsModalOpen(false); }}
        onSuccess={() => { setPrefillEnrollmentId(""); }}
        prefillEnrollmentId={prefillEnrollmentId}
        wsId={wsId}
      />

      <TransferEmployeeModal
        departments={departments}
        employeeId={selectedEmployee?.id ?? null}
        isOpen={isTransferModalOpen}
        onClose={() => { setIsTransferModalOpen(false); }}
        selectedDeptId={selectedDeptId}
        setSelectedDeptId={setSelectedDeptId}
        wsId={wsId}
      />

      <EmploymentHistoryModal
        employmentHistory={employmentHistory}
        formatDate={formatDate}
        isOpen={isHistoryModalOpen}
        onClose={() => { setIsHistoryModalOpen(false); }}
      />

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
