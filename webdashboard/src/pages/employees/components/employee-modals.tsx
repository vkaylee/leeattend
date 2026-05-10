import React from "react";
import { useTranslation } from "react-i18next";

import AttendanceImportDialog from "@/components/features/attendance/attendance-import-dialog";
import EmployeeFormModal from "@/components/features/employees/employee-form-modal";
import RehireEmployeeModal from "@/components/features/employees/rehire-employee-modal";
import TransferEmployeeModal from "@/components/features/employees/transfer-employee-modal";

import type { Employee, Department } from "../../../lib/api/types";

interface EmployeeModalsProps {
  departments: Department[] | undefined;
  editingEmployee: Employee | null;

  isImportDialogOpen: boolean;
  isModalOpen: boolean;
  isRehireModalOpen: boolean;
  isTransferModalOpen: boolean;
  prefillEnrollmentId: string;
  selectedDeptId: string;
  selectedEmployee: Employee | null;
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

    isImportDialogOpen,
    isModalOpen,
    isRehireModalOpen,
    isTransferModalOpen,
    prefillEnrollmentId,
    selectedDeptId,
    selectedEmployee,
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
    </>
  );
}
