import { useRef } from "react";
import { useTranslation } from "react-i18next";

import { useDateTimeFormatter } from "@/hooks/use-datetime";

import { useEmployeesData } from "./use-employees-data";
import { useEmployeesModals } from "./use-employees-modals";
import { useEmployeesState, SortKey, SortDir } from "./use-employees-state";

export type { SortKey, SortDir };

export function useEmployeesPage() {
  const { t } = useTranslation();
  
  const { formatDate } = useDateTimeFormatter();

  const state = useEmployeesState();
  const modals = useEmployeesModals();

  const data = useEmployeesData({
    debouncedSearch: state.debouncedSearch,
    filterDeptIds: state.filterDeptIds,
    filterStatus: state.filterStatus,
    selectedEmployee: modals.selectedEmployee,
    setSelectedEmployee: modals.setSelectedEmployee,
    sortDir: state.sortDir,
    sortKey: state.sortKey,
    customFieldFilters: state.customFieldFilters,
  });

  const searchInputRef = useRef<HTMLInputElement>(null);
  const focusSearch = () => { setTimeout(() => { searchInputRef.current?.focus(); }, 100); };

  const handleDrawerClose = () => {
    modals.setSelectedEmployee(null);
    focusSearch();
  };

  return {
    formatDate,
    handleDrawerClose,
    searchInputRef,
    t,
    ...state,
    ...modals,
    ...data,
  };
}
