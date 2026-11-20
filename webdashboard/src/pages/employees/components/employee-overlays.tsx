import React, { Suspense, lazy } from "react";
import { useTranslation } from "react-i18next";

import type { Employee, Department } from "../../../lib/api/types";
import type { EmployeeModalState } from "../types/employee-modals";
// Lazy-loaded overlays — only loaded when user triggers the action.
// This keeps the employees.lazy chunk under the 300 kB budget.
const EmployeeFormModal = lazy(() => import("@/components/features/employees/employee-form-modal"));
const AttendanceImportDialog = lazy(() => import("@/components/features/attendance/attendance-import-dialog"));
const EmployeeTerminationImportDialog = lazy(() => import("@/components/features/employees/employee-termination-import-dialog"));
const TransferEmployeeModal = lazy(() => import("@/components/features/employees/transfer-employee-modal"));
const TerminateEmployeeModal = lazy(() => import("@/components/features/employees/terminate-employee-modal"));
const RehireEmployeeModal = lazy(() => import("@/components/features/employees/rehire-employee-modal"));

const OverlayFallback = () => null;

interface EmployeeOverlaysProps {
  activeModal: EmployeeModalState;
  closeModal: () => void;
  departments: Department[] | undefined;
  activeEmploymentId?: string | null;
  selectedDeptId: string;
  setSelectedDeptId: (id: string) => void;
  onEmployeeRefresh?: () => void;
}
export function EmployeeOverlays(props: EmployeeOverlaysProps) {
  const { t } = useTranslation();
  const {
    activeModal,
    closeModal,
    departments,
    activeEmploymentId,
    selectedDeptId = "",
    setSelectedDeptId,
    onEmployeeRefresh,
  } = props;

  const currentEditingEmployee = activeModal?.type === "edit" ? activeModal.employee : null;
  const currentTransferEmployee = activeModal?.type === "transfer" ? activeModal.employee : null;
  const currentTerminateEmployee = activeModal?.type === "terminate" ? activeModal.employee : null;
  const currentRehireEmployee = activeModal?.type === "rehire" ? activeModal.employee : null;
  const importOpen = activeModal?.type === "import";
  const transferOpen = activeModal?.type === "transfer";
  const terminateOpen = activeModal?.type === "terminate";
  const rehireOpen = activeModal?.type === "rehire";
  const terminationImportOpen = activeModal?.type === "import-termination";

  const handleClose = () => {
    setSelectedDeptId("");
    closeModal();
  };
  return (
    <>
      <Suspense fallback={<OverlayFallback />}>
        {currentEditingEmployee ? (
          <EmployeeFormModal
            editingEmployee={currentEditingEmployee}
            isOpen
            onClose={handleClose}
            onSuccess={onEmployeeRefresh}
          />
        ) : null}
      </Suspense>

      <Suspense fallback={<OverlayFallback />}>
        <TransferEmployeeModal departments={departments}
        employeeId={currentTransferEmployee?.id ?? null}
        isOpen={transferOpen}
        onClose={handleClose}
        selectedDeptId={selectedDeptId}
        setSelectedDeptId={setSelectedDeptId}  />
      </Suspense>

      <Suspense fallback={<OverlayFallback />}>
        <TerminateEmployeeModal activeEmploymentId={activeEmploymentId ?? currentTerminateEmployee?.active_employment_id ?? null}
        employeeId={currentTerminateEmployee?.id ?? null}
        isOpen={terminateOpen}
        onClose={handleClose}
        onSuccess={onEmployeeRefresh}  />
      </Suspense>

      <Suspense fallback={<OverlayFallback />}>
        <RehireEmployeeModal employeeId={currentRehireEmployee?.id ?? null}
        isOpen={rehireOpen}
        onClose={handleClose}
        onSuccess={() => { onEmployeeRefresh?.(); }}  />
      </Suspense>

      <Suspense fallback={<OverlayFallback />}>
        <AttendanceImportDialog
          customRequirements={[
            "Cột A (employee_code): Mã nhân viên (Bắt buộc)",
            "Cột B (full_name): Họ và tên (Bắt buộc)",
            "Cột C (hire_date): Ngày vào làm (VD: 2023-01-15)",
            "Cột D (department_name): Tên phòng ban",
          ]}
          endpoint="/employees/upload"
          isOpen={importOpen}
          onClose={handleClose}
          targetType="employee"
          title={t("employees.bulk_import_title")}
        />
      </Suspense>

      <Suspense fallback={<OverlayFallback />}>
        <EmployeeTerminationImportDialog isOpen={terminationImportOpen}
        onClose={handleClose}  />
      </Suspense>
    </>
  );
}
