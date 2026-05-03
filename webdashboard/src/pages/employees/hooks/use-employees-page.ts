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

  const state = useEmployeesState();
  const modals = useEmployeesModals();

  const data = useEmployeesData({
    debouncedSearch: state.debouncedSearch,
    filterDeptIds: state.filterDeptIds,
    selectedEmployee: modals.selectedEmployee,
    setEditingEmployee: modals.setEditingEmployee,
    setIsModalOpen: modals.setIsModalOpen,
    sortDir: state.sortDir,
    sortKey: state.sortKey,
    wsId,
  });

  const { handleTerminate } = useEmployeeActions({ confirm, updateMutation: data.updateMutation });

  const searchInputRef = useRef<HTMLInputElement>(null);
  const focusSearch = () => { setTimeout(() => { searchInputRef.current?.focus(); }, 100); };

  const handleDrawerClose = () => {
    modals.setSelectedEmployee(null);
    focusSearch();
  };

  return {
    ConfirmationDialog,
    formatDate,
    handleDrawerClose,
    handleTerminate,
    searchInputRef,
    t,
    wsId,
    ...state,
    ...modals,
    ...data,
  };
}
