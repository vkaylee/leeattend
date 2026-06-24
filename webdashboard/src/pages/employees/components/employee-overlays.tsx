import React from "react";
import { useTranslation } from "react-i18next";

import AttendanceImportDialog from "@/components/features/attendance/attendance-import-dialog";
import EmployeeDetailDrawer from "@/components/features/employees/employee-detail-drawer";
import EmployeeFormModal from "@/components/features/employees/employee-form-modal";
import RehireEmployeeModal from "@/components/features/employees/rehire-employee-modal";
import TransferEmployeeModal from "@/components/features/employees/transfer-employee-modal";
import TerminateEmployeeModal from "@/components/features/employees/terminate-employee-modal";
import EmployeeTerminationImportDialog from "@/components/features/employees/employee-termination-import-dialog";

import type { Employee, Department, EmployeeDepartmentHistory } from "../../../lib/api/types";

interface EmployeeOverlaysProps {
  departments: Department[] | undefined;
  deptHistory: EmployeeDepartmentHistory[] | undefined;
  editingEmployee: Employee | null;

  formatDate: (date: string) => string;
  handleDrawerClose: () => void;

  isImportDialogOpen: boolean;
  isTerminationImportOpen: boolean;
  isModalOpen: boolean;
  isRehireModalOpen: boolean;
  isTransferModalOpen: boolean;
  isTerminateModalOpen: boolean;
  prefillEnrollmentId: string;
  selectedDeptId: string;
  selectedEmployee: Employee | null;
  setEditingEmployee: (emp: Employee | null) => void;

  setIsImportDialogOpen: (open: boolean) => void;
  setIsTerminationImportOpen: (open: boolean) => void;
  setIsModalOpen: (open: boolean) => void;
  setIsRehireModalOpen: (open: boolean) => void;
  setIsTransferModalOpen: (open: boolean) => void;
  setIsTerminateModalOpen: (open: boolean) => void;
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

    formatDate,
    handleDrawerClose,

    isImportDialogOpen,
    isTerminationImportOpen,
    isModalOpen,
    isRehireModalOpen,
    isTransferModalOpen,
    isTerminateModalOpen,
    prefillEnrollmentId,
    selectedDeptId,
    selectedEmployee,
    setEditingEmployee,

    setIsImportDialogOpen,
    setIsTerminationImportOpen,
    setIsModalOpen,
    setIsRehireModalOpen,
    setIsTransferModalOpen,
    setIsTerminateModalOpen,
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

        formatDate={formatDate}
        isOpen={!!selectedEmployee}
        key={selectedEmployee?.id ?? "closed"}
        onClose={handleDrawerClose}
        onEdit={(emp) => {
          setEditingEmployee(emp);
          setIsModalOpen(true);
        }}
        onRehire={() => { setIsRehireModalOpen(true); }}
        onTerminate={(_emp) => { setIsTerminateModalOpen(true); }}
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

      <TerminateEmployeeModal
        activeEmploymentId={selectedEmployee?.active_employment_id ?? null}
        employeeId={selectedEmployee?.id ?? null}
        isOpen={isTerminateModalOpen}
        onClose={() => { setIsTerminateModalOpen(false); }}
        onSuccess={() => {
          if (selectedEmployee) {
            setSelectedEmployee({
              ...selectedEmployee,
              active_employment_id: null,
            });
          }
        }}
        wsId={wsId}
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
          "Cột A (employee_code): Mã nhân viên (Bắt buộc)",
          "Cột B (full_name): Họ và tên (Bắt buộc)",
          "Cột C (hire_date): Ngày vào làm (VD: 2023-01-15)",
          "Cột D (department_name): Tên phòng ban",
        ]}
        endpoint="/employees/upload"
        isOpen={isImportDialogOpen}
        onClose={() => { setIsImportDialogOpen(false); }}
        targetType="employee"
        title={t("employees.bulk_import_title")}
      />

      <EmployeeTerminationImportDialog
        isOpen={isTerminationImportOpen}
        onClose={() => { setIsTerminationImportOpen(false); }}
        wsId={wsId}
      />
    </>
  );
}
