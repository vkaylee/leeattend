import React, { Suspense, lazy } from "react";
import { useTranslation } from "react-i18next";
import type { Employee, Department, EmployeeDepartmentHistory } from "../../../lib/api/types";
import type { EmployeeModalState } from "../types/employee-modals";
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
  activeModal?: EmployeeModalState;
  closeModal?: () => void;
  departments: Department[] | undefined;
  deptHistory: EmployeeDepartmentHistory[] | undefined;
  editingEmployee?: Employee | null;
  formatDate: (date: string) => string;
  handleDrawerClose: () => void;
  isImportDialogOpen?: boolean;
  isModalOpen?: boolean;
  isRehireModalOpen?: boolean;
  isTerminateModalOpen?: boolean;
  isTerminationImportOpen?: boolean;
  isTransferModalOpen?: boolean;
  openModal?: (modal: EmployeeModalState) => void;
  prefillEnrollmentId?: string;
  selectedDeptId?: string;
  selectedEmployee: Employee | null;
  setEditingEmployee?: (emp: Employee | null) => void;
  setIsImportDialogOpen?: (open: boolean) => void;
  setIsModalOpen?: (open: boolean) => void;
  setIsRehireModalOpen?: (open: boolean) => void;
  setIsTerminateModalOpen?: (open: boolean) => void;
  setIsTerminationImportOpen?: (open: boolean) => void;
  setIsTransferModalOpen?: (open: boolean) => void;
  setPrefillEnrollmentId?: (id: string) => void;
  setSelectedDeptId?: (id: string) => void;
  setSelectedEmployee: (emp: Employee | null) => void;
  wsId: string;
}
  // eslint-disable-next-line complexity
export function EmployeeOverlays(props: EmployeeOverlaysProps) {
  const { t } = useTranslation();
  const {
    activeModal,
    closeModal,
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
    openModal,
    prefillEnrollmentId = "",
    selectedDeptId = "",
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

  const modalOpen = activeModal ? activeModal.type === "create" || activeModal.type === "edit" : Boolean(isModalOpen);
  const currentEditingEmployee = activeModal?.type === "edit" ? activeModal.employee : editingEmployee;
  const currentTransferEmployee = activeModal?.type === "transfer" ? activeModal.employee : selectedEmployee;
  const currentTerminateEmployee = activeModal?.type === "terminate" ? activeModal.employee : selectedEmployee;
  const currentRehireEmployee = activeModal?.type === "rehire" ? activeModal.employee : selectedEmployee;
  const importOpen = activeModal ? activeModal.type === "import" : Boolean(isImportDialogOpen);
  const transferOpen = activeModal ? activeModal.type === "transfer" : Boolean(isTransferModalOpen);
  const terminateOpen = activeModal ? activeModal.type === "terminate" : Boolean(isTerminateModalOpen);
  const rehireOpen = activeModal ? activeModal.type === "rehire" : Boolean(isRehireModalOpen);
  const terminationImportOpen = activeModal ? activeModal.type === "import-termination" : Boolean(isTerminationImportOpen);

  const handleClose = () => {
    closeModal?.();
    setIsModalOpen?.(false);
    setIsImportDialogOpen?.(false);
    setIsTransferModalOpen?.(false);
    setIsTerminateModalOpen?.(false);
    setIsRehireModalOpen?.(false);
    setIsTerminationImportOpen?.(false);
  };
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
            if (openModal) openModal({ type: "edit", employee: emp });
            else {
              setEditingEmployee?.(emp);
              setIsModalOpen?.(true);
            }
          }}
          onEmployeeUpdate={setSelectedEmployee}
          onRehire={() => {
            if (openModal && selectedEmployee) openModal({ type: "rehire", employee: selectedEmployee });
            else setIsRehireModalOpen?.(true);
          }}
          onTerminate={(_emp) => {
            if (openModal && selectedEmployee) openModal({ type: "terminate", employee: selectedEmployee });
            else setIsTerminateModalOpen?.(true);
          }}
          onTransfer={(deptId) => {
            setSelectedDeptId?.(deptId);
            if (openModal && selectedEmployee) openModal({ type: "transfer", employee: selectedEmployee, departmentId: deptId });
            else setIsTransferModalOpen?.(true);
          }}
        />
      </Suspense>

      <Suspense fallback={<OverlayFallback />}>
        <EmployeeFormModal
          editingEmployee={currentEditingEmployee}
          isOpen={modalOpen}
          onClose={handleClose}
          onSuccess={() => { setPrefillEnrollmentId?.(""); }}
          prefillEnrollmentId={prefillEnrollmentId}
          wsId={wsId}
        />
      </Suspense>

      <Suspense fallback={<OverlayFallback />}>
        <TransferEmployeeModal
          departments={departments}
          employeeId={currentTransferEmployee?.id ?? null}
          isOpen={transferOpen}
          onClose={handleClose}
          selectedDeptId={selectedDeptId}
          setSelectedDeptId={setSelectedDeptId}
          wsId={wsId}
        />
      </Suspense>

      <Suspense fallback={<OverlayFallback />}>
        <TerminateEmployeeModal
          activeEmploymentId={currentTerminateEmployee?.active_employment_id ?? null}
          employeeId={currentTerminateEmployee?.id ?? null}
          isOpen={terminateOpen}
          onClose={handleClose}
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
          employeeId={currentRehireEmployee?.id ?? null}
          isOpen={rehireOpen}
          onClose={handleClose}
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
          isOpen={importOpen}
          onClose={handleClose}
          targetType="employee"
          title={t("employees.bulk_import_title")}
        />
      </Suspense>

      <Suspense fallback={<OverlayFallback />}>
        <EmployeeTerminationImportDialog
          isOpen={terminationImportOpen}
          onClose={handleClose}
          wsId={wsId}
        />
      </Suspense>
    </>
  );
}
