import { useState, useEffect, useCallback } from "react";

import type { Employee } from "@/lib/api";

import type { EmployeeModalState } from "../types/employee-modals";

export const useEmployeesModals = () => {
  const [searchParams] = useState(() => new URLSearchParams(typeof window !== "undefined" ? window.location.search : ""));

  const [selectedEmployee, setSelectedEmployee] = useState<Employee | null>(null);
  const [activeModal, setActiveModal] = useState<EmployeeModalState>(() =>
    searchParams.get("enrollment_id") ? { type: "create" } : null
  );
  const [prefillEnrollmentId, setPrefillEnrollmentId] = useState<string>(() => searchParams.get("enrollment_id") ?? "");
  const [selectedDeptId, setSelectedDeptId] = useState<string>("");

  const openModal = useCallback((modal: EmployeeModalState) => {
    setActiveModal(modal);
  }, []);

  const closeModal = useCallback(() => {
    setActiveModal(null);
  }, []);

  useEffect(() => {
    const enrollmentId = searchParams.get("enrollment_id");
    if (enrollmentId && typeof window !== "undefined") {
      const url = new URL(window.location.href);
      url.searchParams.delete("enrollment_id");
      window.history.replaceState({}, "", url.toString());
    }
  }, [searchParams]);

  // Backward compatibility getters & setters
  const isModalOpen = activeModal?.type === "create" || activeModal?.type === "edit";
  const editingEmployee = activeModal?.type === "edit" ? activeModal.employee : null;
  const isImportDialogOpen = activeModal?.type === "import";
  const isTerminationImportOpen = activeModal?.type === "import-termination";
  const isTransferModalOpen = activeModal?.type === "transfer";
  const isTerminateModalOpen = activeModal?.type === "terminate";
  const isRehireModalOpen = activeModal?.type === "rehire";

  const setIsModalOpen = useCallback((open: boolean) => {
    if (open) {
      setActiveModal(prev => (prev?.type === "edit" ? prev : { type: "create" }));
    } else {
      setActiveModal(null);
    }
  }, []);

  const setEditingEmployee = useCallback((emp: Employee | null) => {
    if (emp) {
      setActiveModal({ type: "edit", employee: emp });
    } else {
      setActiveModal(prev => (prev?.type === "edit" ? null : prev));
    }
  }, []);

  const setIsImportDialogOpen = useCallback((open: boolean) => {
    setActiveModal(open ? { type: "import" } : null);
  }, []);

  const setIsTerminationImportOpen = useCallback((open: boolean) => {
    setActiveModal(open ? { type: "import-termination" } : null);
  }, []);

  const setIsTransferModalOpen = useCallback((open: boolean) => {
    if (open) {
      if (selectedEmployee) {
        setActiveModal({ type: "transfer", employee: selectedEmployee });
      }
    } else {
      setActiveModal(null);
    }
  }, [selectedEmployee]);

  const setIsTerminateModalOpen = useCallback((open: boolean) => {
    if (open) {
      if (selectedEmployee) {
        setActiveModal({ type: "terminate", employee: selectedEmployee });
      }
    } else {
      setActiveModal(null);
    }
  }, [selectedEmployee]);

  const setIsRehireModalOpen = useCallback((open: boolean) => {
    if (open) {
      if (selectedEmployee) {
        setActiveModal({ type: "rehire", employee: selectedEmployee });
      }
    } else {
      setActiveModal(null);
    }
  }, [selectedEmployee]);

  return {
    activeModal,
    openModal,
    closeModal,
    selectedEmployee,
    setSelectedEmployee,
    prefillEnrollmentId,
    setPrefillEnrollmentId,
    selectedDeptId,
    setSelectedDeptId,
    // Backward-compatible interface
    isModalOpen,
    setIsModalOpen,
    isImportDialogOpen,
    setIsImportDialogOpen,
    editingEmployee,
    setEditingEmployee,
    isTransferModalOpen,
    setIsTransferModalOpen,
    isHistoryModalOpen: false,
    setIsHistoryModalOpen: () => undefined,
    isRehireModalOpen,
    setIsRehireModalOpen,
    isTerminateModalOpen,
    setIsTerminateModalOpen,
    isTerminationImportOpen,
    setIsTerminationImportOpen,
  };
};
