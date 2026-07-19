import React, { Suspense, lazy } from "react";
import { useTranslation } from "react-i18next";

import type { Employee, Department, EmployeeDepartmentHistory } from "../../../lib/api/types";

// Lazy-loaded overlays — only loaded when user triggers the action.
// This keeps the employees.lazy chunk under the 300 kB budget.
const EmployeeDetailDrawer = lazy(() => import("@/components/features/employees/employee-detail-drawer"));
const EmployeeFormModal = lazy(() => import("@/components/features/employees/employee-form-modal"));
const AttendanceImportDialog = lazy(() => import("@/components/features/attendance/attendance-import-dialog"));
const EmployeeTerminationImportDialog = lazy(() => import("@/components/features/employees/employee-termination-import-dialog"));
const TransferEmployeeModal = lazy(() => import("@/components/features/employees/transfer-employee-modal"));
const TerminateEmployeeModal = lazy(() => import("@/components/features/employees/terminate-employee-modal"));
const RehireEmployeeModal = lazy(() => import("@/components/features/employees/rehire-employee-modal"));

const OverlayFallback = () => null;

interface EmployeeOverlaysProps {
  departments: Department[] | undefined;
  deptHistory: EmployeeDepartmentHistory[] | undefined;
  editingEmployee: Employee | null;

  formatDate: (date: string) => string;
  handleDrawerClose: () => void;

  isImportDialogOpen: boolean;
  isModalOpen: boolean;
  isRehireModalOpen: boolean;
  isTerminateModalOpen: boolean;
  isTerminationImportOpen: boolean;
  isTransferModalOpen: boolean;
  prefillEnrollmentId: string;
  selectedDeptId: string;
  selectedEmployee: Employee | null;
  setEditingEmployee: (emp: Employee | null) => void;

  setIsImportDialogOpen: (open: boolean) => void;
  setIsModalOpen: (open: boolean) => void;
  setIsRehireModalOpen: (open: boolean) => void;
  setIsTerminateModalOpen: (open: boolean) => void;
  setIsTerminationImportOpen: (open: boolean) => void;
  setIsTransferModalOpen: (open: boolean) => void;
  setPrefillEnrollmentId: (id: string) => void;
  setSelectedDeptId: (id: string) => void;
  setSelectedEmployee: (emp: Employee | null) => void;
  wsId: string;
}

  // eslint-disable-next-line complexity
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
      <Suspense fallback={<OverlayFallback />}>
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
          onEmployeeUpdate={setSelectedEmployee}
          onRehire={() => { setIsRehireModalOpen(true); }}
          onTerminate={(_emp) => { setIsTerminateModalOpen(true); }}
          onTransfer={(deptId) => {
            setSelectedDeptId(deptId);
            setIsTransferModalOpen(true);
          }}
          wsId={wsId}
        />
      </Suspense>

      <Suspense fallback={<OverlayFallback />}>
        <EmployeeFormModal
          editingEmployee={editingEmployee}
          isOpen={isModalOpen}
          onClose={() => { setIsModalOpen(false); }}
          onSuccess={() => { setPrefillEnrollmentId(""); }}
          prefillEnrollmentId={prefillEnrollmentId}
          wsId={wsId}
        />
      </Suspense>

      <Suspense fallback={<OverlayFallback />}>
        <TransferEmployeeModal
          departments={departments}
          employeeId={selectedEmployee?.id ?? null}
          isOpen={isTransferModalOpen}
          onClose={() => { setIsTransferModalOpen(false); }}
          selectedDeptId={selectedDeptId}
          setSelectedDeptId={setSelectedDeptId}
          wsId={wsId}
        />
      </Suspense>

      <Suspense fallback={<OverlayFallback />}>
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
      </Suspense>

      <Suspense fallback={<OverlayFallback />}>
        <RehireEmployeeModal
          employeeId={selectedEmployee?.id ?? null}
          isOpen={isRehireModalOpen}
          onClose={() => { setIsRehireModalOpen(false); }}
          onSuccess={(updatedEmp) => { setSelectedEmployee(updatedEmp); }}
          wsId={wsId}
        />
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
          isOpen={isImportDialogOpen}
          onClose={() => { setIsImportDialogOpen(false); }}
          targetType="employee"
          title={t("employees.bulk_import_title")}
        />
      </Suspense>

      <Suspense fallback={<OverlayFallback />}>
        <EmployeeTerminationImportDialog
          isOpen={isTerminationImportOpen}
          onClose={() => { setIsTerminationImportOpen(false); }}
          wsId={wsId}
        />
      </Suspense>
    </>
  );
}
