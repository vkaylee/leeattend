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
  } = page;

  return (
    <div className="space-y-6 pb-20 animate-in fade-in duration-700">
      <EmployeesPageHeader
        setEditingEmployee={page.setEditingEmployee}
        setIsImportDialogOpen={page.setIsImportDialogOpen}
        setIsModalOpen={page.setIsModalOpen}
        setIsTerminationImportOpen={page.setIsTerminationImportOpen}
      />

      <EmployeeFilterChips
        customFieldFilters={page.customFieldFilters}
        departments={page.departments}
        filterDeptIds={page.filterDeptIds}
        filterStatus={page.filterStatus}
        setCustomFieldFilters={page.setCustomFieldFilters}
        setFilterDeptIds={page.setFilterDeptIds}
        setFilterStatus={page.setFilterStatus}
        toggleDeptFilter={page.toggleDeptFilter}
        updateCustomFieldFilter={page.updateCustomFieldFilter}
      />

      <EmployeeTable
        customFieldFilters={page.customFieldFilters}
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
        setCustomFieldFilters={page.setCustomFieldFilters}
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
    </div>
  );
}
