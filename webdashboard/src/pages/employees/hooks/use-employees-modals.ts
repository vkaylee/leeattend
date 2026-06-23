import { useState, useEffect } from "react";

import type { Employee } from "@/lib/api";

export const useEmployeesModals = () => {
  const [searchParams] = useState(() => new URLSearchParams(typeof window !== "undefined" ? window.location.search : ""));
  
  const [isModalOpen, setIsModalOpen] = useState(() => !!searchParams.get("enrollment_id"));
  const [isImportDialogOpen, setIsImportDialogOpen] = useState(false);
  const [editingEmployee, setEditingEmployee] = useState<Employee | null>(null);
  const [selectedEmployee, setSelectedEmployee] = useState<Employee | null>(null);
  const [prefillEnrollmentId, setPrefillEnrollmentId] = useState<string>(() => searchParams.get("enrollment_id") ?? "");
  
  const [isTransferModalOpen, setIsTransferModalOpen] = useState(false);
  const [isHistoryModalOpen, setIsHistoryModalOpen] = useState(false);
  const [isRehireModalOpen, setIsRehireModalOpen] = useState(false);
  const [isTerminationImportOpen, setIsTerminationImportOpen] = useState(false);
  const [selectedDeptId, setSelectedDeptId] = useState<string>("");

  useEffect(() => {
    const enrollmentId = searchParams.get("enrollment_id");
    if (enrollmentId && typeof window !== "undefined") {
      const url = new URL(window.location.href);
      url.searchParams.delete("enrollment_id");
      window.history.replaceState({}, "", url.toString());
    }
  }, [searchParams]);

  return {
    isModalOpen,
    setIsModalOpen,
    isImportDialogOpen,
    setIsImportDialogOpen,
    editingEmployee,
    setEditingEmployee,
    selectedEmployee,
    setSelectedEmployee,
    prefillEnrollmentId,
    setPrefillEnrollmentId,
    isTransferModalOpen,
    setIsTransferModalOpen,
    isHistoryModalOpen,
    setIsHistoryModalOpen,
    isRehireModalOpen,
    setIsRehireModalOpen,
    isTerminationImportOpen,
    setIsTerminationImportOpen,
    selectedDeptId,
    setSelectedDeptId,
  };
};
