import React from "react";
import { useTranslation } from "react-i18next";

import AttendanceImportDialog from "@/components/features/attendance/attendance-import-dialog";
import EmployeeFormModal from "@/components/features/employees/employee-form-modal";
import RehireEmployeeModal from "@/components/features/employees/rehire-employee-modal";
import TransferEmployeeModal from "@/components/features/employees/transfer-employee-modal";

import type { Employee, Department } from "../../../lib/api/types";

interface EmployeeModalsProps { departments: Department[] | undefined;
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
setSelectedEmployee: (emp: Employee | null) => void;  }

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
  } = props;

  return (
    <>
      {/* Main Employee Form Modal */}
      <EmployeeFormModal editingEmployee={editingEmployee}
      isOpen={isModalOpen}
      onClose={() => { setIsModalOpen(false); }}
      onSuccess={() => { setPrefillEnrollmentId(""); }}
      prefillEnrollmentId={prefillEnrollmentId}  />

      {/* Transfer Modal */}
      <TransferEmployeeModal departments={departments}
      employeeId={selectedEmployee?.id ?? null}
      isOpen={isTransferModalOpen}
      onClose={() => { setIsTransferModalOpen(false); }}
      selectedDeptId={selectedDeptId}
      setSelectedDeptId={setSelectedDeptId}  />



      {/* Rehire Modal */}
      <RehireEmployeeModal employeeId={selectedEmployee?.id ?? null}
      isOpen={isRehireModalOpen}
      onClose={() => { setIsRehireModalOpen(false); }}
      onSuccess={(updatedEmp) => { setSelectedEmployee(updatedEmp); }}  />

      <AttendanceImportDialog
        endpoint="/employees/upload"
        expectedColumns={[
          {
            key: "employee_code",
            label: "Mã nhân viên",
            required: true,
            description: "Mã số định danh duy nhất của nhân viên trong hệ thống (Employee ID)."
          },
          {
            key: "full_name",
            label: "Họ và tên",
            required: true,
            description: "Họ và tên đầy đủ của nhân viên."
          },
          {
            key: "hire_date",
            label: "Ngày vào làm",
            required: false,
            format: "YYYY-MM-DD hoặc DD/MM/YYYY",
            description: "Ngày bắt đầu làm việc chính thức (mặc định lấy ngày hôm nay nếu để trống)."
          },
          {
            key: "department_name",
            label: "Tên phòng ban",
            required: false,
            description: "Phòng ban làm việc của nhân viên (sẽ tự động tạo phòng ban mới nếu chưa tồn tại)."
          }
        ]}
        isOpen={isImportDialogOpen}
        onClose={() => { setIsImportDialogOpen(false); }}
        targetType="employee"
        title={t("employees.bulk_import_title")}
      />
    </>
  );
}
