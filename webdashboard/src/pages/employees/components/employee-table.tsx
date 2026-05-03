import React from "react";
import { useTranslation } from "react-i18next";

import NavigableList from "@/components/common/navigable-list";
import { cn } from "@/lib/utils";

import type { Employee, Department } from "../../../lib/api/types";
import type { SortKey, SortDir } from "../hooks/use-employees-page";

import { useEmployeeColumns } from "../hooks/use-employee-columns";

interface EmployeeTableProps {
  departments: Department[] | undefined;
  employees: Employee[];
  fetchNextPage: () => void;
  filterDeptIds: string[];
  hasNextPage: boolean | undefined;
  isFetching: boolean;
  isFetchingNextPage: boolean;
  isLoading: boolean;
  isPlaceholderData: boolean;
  searchInputRef: React.RefObject<HTMLInputElement | null>;
  searchTerm: string;
  selectedEmployee: Employee | null;
  setFilterDeptIds: (ids: string[]) => void;
  setSearchTerm: (s: string) => void;
  setSelectedEmployee: (emp: Employee) => void;
  sortDir: SortDir;
  sortKey: SortKey | null;
  toggleDeptFilter: (id: string) => void;
  toggleSort: (key: SortKey) => void;
  totalItems: number;
}

export function EmployeeTable(props: EmployeeTableProps) {
  const { t } = useTranslation();
  const {
    departments,
    employees,
    fetchNextPage,
    filterDeptIds,
    hasNextPage,
    isFetching,
    isFetchingNextPage,
    isLoading,
    isPlaceholderData,
    searchInputRef,
    searchTerm,
    selectedEmployee,
    setFilterDeptIds,
    setSearchTerm,
    setSelectedEmployee,
    sortDir,
    sortKey,
    toggleDeptFilter,
    toggleSort,
  } = props;

  const employeeColumns = useEmployeeColumns({
    departments,
    filterDeptIds,
    selectedEmployee,
    setFilterDeptIds,
    sortDir,
    sortKey,
    toggleDeptFilter,
    toggleSort,
  });

  return (
    <div className={cn("transition-opacity duration-300", isPlaceholderData && isFetching && "opacity-50")}>
      <NavigableList<Employee>
        className="animate-in slide-in-from-bottom-8 duration-700"
        columns={employeeColumns}
        emptyMessage={t("employees.table.no_data")}
        fetchNextPage={fetchNextPage}
        hasNextPage={hasNextPage}
        isFetchingNextPage={isFetchingNextPage}
        isLoading={isLoading}
        itemKey={(emp) => emp.id}
        items={employees}
        maxHeightClass=""
        onSearchChange={setSearchTerm}
        onSelect={setSelectedEmployee}
        resetDeps={[searchTerm, filterDeptIds, sortKey, sortDir]}
        rowIdPrefix="employee-row"
        searchInputRef={searchInputRef}
        searchPlaceholder={t("employees.search_placeholder")}
        searchTestId="employee-search"
        searchValue={searchTerm}
        selectedKey={selectedEmployee?.id}
        showChevron
        testIdPrefix="employees-row"
      />
    </div>
  );
}
