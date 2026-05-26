import FloatingRecordBadge from "@/components/common/floating-record-badge";

import { EmployeeFilterChips } from "./employees/components/employee-filter-chips";
import { EmployeeOverlays } from "./employees/components/employee-overlays";
import { EmployeeTable } from "./employees/components/employee-table";
import { EmployeesPageHeader } from "./employees/components/employees-page-header";
import { useEmployeesPage } from "./employees/hooks/use-employees-page";

export default function EmployeesPage() {
  const page = useEmployeesPage();
  const {
    isLoading,
    employees,
    totalItems,
    t,
    ConfirmationDialog,
  } = page;

  return (
    <div className="space-y-10 pb-20 animate-in fade-in duration-700">
      <EmployeesPageHeader
        setEditingEmployee={page.setEditingEmployee}
        setIsImportDialogOpen={page.setIsImportDialogOpen}
        setIsModalOpen={page.setIsModalOpen}
      />

      <EmployeeFilterChips
        departments={page.departments}
        filterDeptIds={page.filterDeptIds}
        setFilterDeptIds={page.setFilterDeptIds}
        toggleDeptFilter={page.toggleDeptFilter}
      />

      <EmployeeTable
        departments={page.departments}
        employees={page.employees}
        fetchNextPage={page.fetchNextPage}
        filterDeptIds={page.filterDeptIds}
        hasNextPage={page.hasNextPage}
        isFetching={page.isFetching}
        isFetchingNextPage={page.isFetchingNextPage}
        isLoading={isLoading}
        isPlaceholderData={page.isPlaceholderData}
        searchInputRef={page.searchInputRef}
        searchTerm={page.searchTerm}
        selectedEmployee={page.selectedEmployee}
        setFilterDeptIds={page.setFilterDeptIds}
        setSearchTerm={page.setSearchTerm}
        setSelectedEmployee={page.setSelectedEmployee}
        sortDir={page.sortDir}
        sortKey={page.sortKey}
        toggleDeptFilter={page.toggleDeptFilter}
        toggleSort={page.toggleSort}
        totalItems={page.totalItems}
      />

      <EmployeeOverlays {...page} />

      <FloatingRecordBadge label={t("common.pagination.records")} loaded={employees.length} total={totalItems} />
      
      <ConfirmationDialog />
    </div>
  );
}
