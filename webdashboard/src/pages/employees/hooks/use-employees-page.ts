import { useRef } from "react";
import { useTranslation } from "react-i18next";

import { useConfirm } from "@/hooks/use-confirm";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import {
  getCurrentWorkspaceId,
} from "@/lib/api";

import { useEmployeeActions } from "./use-employee-actions";
import { useEmployeesData } from "./use-employees-data";
import { useEmployeesModals } from "./use-employees-modals";
import { useEmployeesState, SortKey, SortDir } from "./use-employees-state";

export type { SortKey, SortDir };

export function useEmployeesPage() {
  const { t } = useTranslation();
  const wsId = getCurrentWorkspaceId() ?? "";
  const { formatDate } = useDateTimeFormatter();
  const { confirm, ConfirmationDialog } = useConfirm();

  const {
    debouncedSearch,
    filterDeptIds,
    searchTerm,
    setFilterDeptIds,
    setSearchTerm,
    sortDir,
    sortKey,
    toggleDeptFilter,
    toggleSort,
  } = useEmployeesState();

  const {
    editingEmployee,
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
  } = useEmployeesModals();

  const {
    departments,
    deptHistory,
    employees,
    employmentHistory,
    fetchNextPage,
    hasNextPage,
    isFetching,
    isFetchingNextPage,
    isLoading,
    isPlaceholderData,
    totalItems,
    updateMutation,
  } = useEmployeesData({
    debouncedSearch,
    filterDeptIds,
    selectedEmployee,
    setEditingEmployee,
    setIsModalOpen,
    sortDir,
    sortKey,
    wsId,
  });

  const { handleTerminate } = useEmployeeActions({ confirm, updateMutation });

  const searchInputRef = useRef<HTMLInputElement>(null);
  const focusSearch = () => { setTimeout(() => { searchInputRef.current?.focus(); }, 100); };

  const handleDrawerClose = () => {
    setSelectedEmployee(null);
    focusSearch();
  };

  return {
    ConfirmationDialog,
    departments,
    deptHistory,
    editingEmployee,
    employees,
    employmentHistory,
    fetchNextPage,
    filterDeptIds,
    formatDate,
    handleDrawerClose,
    handleTerminate,
    hasNextPage,
    isFetching,
    isFetchingNextPage,
    isHistoryModalOpen,
    isImportDialogOpen,
    isLoading,
    isModalOpen,
    isPlaceholderData,
    isRehireModalOpen,
    isTransferModalOpen,
    prefillEnrollmentId,
    searchInputRef,
    searchTerm,
    selectedDeptId,
    selectedEmployee,
    setEditingEmployee,
    setFilterDeptIds,
    setIsHistoryModalOpen,
    setIsImportDialogOpen,
    setIsModalOpen,
    setIsRehireModalOpen,
    setIsTransferModalOpen,
    setPrefillEnrollmentId,
    setSearchTerm,
    setSelectedDeptId,
    setSelectedEmployee,
    sortDir,
    sortKey,
    t,
    toggleDeptFilter,
    toggleSort,
    totalItems,
    wsId,
  };
}
