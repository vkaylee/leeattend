import { useState, useCallback } from "react";

import type { Employee } from "@/lib/api";

import type { EmployeeModalState } from "../types/employee-modals";

export const useEmployeesModals = () => {
  const [selectedEmployee, setSelectedEmployee] = useState<Employee | null>(null);
  const [activeModal, setActiveModal] = useState<EmployeeModalState>(null);
  const [selectedDeptId, setSelectedDeptId] = useState<string>("");

  const openModal = useCallback((modal: EmployeeModalState) => {
    setActiveModal(modal);
  }, []);

  const closeModal = useCallback(() => {
    setActiveModal(null);
  }, []);

  return {
    activeModal,
    openModal,
    closeModal,
    selectedEmployee,
    setSelectedEmployee,
    selectedDeptId,
    setSelectedDeptId,
  };
};
