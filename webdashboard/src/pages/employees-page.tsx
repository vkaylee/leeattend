import { useNavigate } from "@tanstack/react-router";

import type { Employee } from "@/lib/api/types";

import FloatingRecordBadge from "@/components/common/floating-record-badge";
import { FilterChips } from "@/components/ui/filter-chips";

import { ContractExpirationAlerts } from "./employees/components/contract-expiration-alerts";
import { EmployeeBulkActionBar } from "./employees/components/employee-bulk-action-bar";
import { EmployeeOverlays } from "./employees/components/employee-overlays";
import { EmployeeQuickTabs } from "./employees/components/employee-quick-tabs";
import { EmployeeTable } from "./employees/components/employee-table";
import { EmployeesPageHeader } from "./employees/components/employees-page-header";
import { useEmployeeBulkSelection } from "./employees/hooks/use-employee-bulk-selection";
import { useEmployeeFilterChips } from "./employees/hooks/use-employee-filter-chips";
import { useEmployeesPage } from "./employees/hooks/use-employees-page";
export default function EmployeesPage() {
  const page = useEmployeesPage();
  const {
    isLoading,
    employees,
    totalItems,
    t,
  } = page;
  const navigate = useNavigate();

  const handleSelectEmployee = (emp: Employee) => {
    void navigate({
      to: "/employees/$employeeId",
      params: { employeeId: emp.id },
    });
  };

  const bulk = useEmployeeBulkSelection(employees);
  const { activeChips, handleClearAll, handleRemove } = useEmployeeFilterChips({
    customFieldFilters: page.customFieldFilters,
    departments: page.departments,
    filterDeptIds: page.filterDeptIds,
    filterStatus: page.filterStatus,
    setCustomFieldFilters: page.setCustomFieldFilters,
    setFilterDeptIds: page.setFilterDeptIds,
    setFilterStatus: page.setFilterStatus,
    t: (key, defaultValue) => t(key, { defaultValue }),
    toggleDeptFilter: page.toggleDeptFilter,
    updateCustomFieldFilter: page.updateCustomFieldFilter,
  });
  return (
    <div className="space-y-6 pb-20 animate-in fade-in duration-700">
      <EmployeesPageHeader
        openModal={page.openModal}
      />

      <ContractExpirationAlerts employees={employees} />

      <FilterChips
        chips={activeChips}
        data-testid="active-filter-chips"
        label={t("employees.filtering_by", "Đang lọc")}
        onClearAll={handleClearAll}
        onRemove={handleRemove}
      />

      <EmployeeQuickTabs
        activeCount={page.employees.filter((e) => Boolean(e.active_employment_id)).length}
        filterStatus={page.filterStatus}
        setFilterStatus={page.setFilterStatus}
        totalCount={totalItems}
      />
      <EmployeeTable
        customFieldFilters={page.customFieldFilters}
        departments={page.departments}
        employees={page.employees}
        fetchNextPage={page.fetchNextPage}
        filterDeptIds={page.filterDeptIds}
        filterStatus={page.filterStatus}
        formatDate={page.formatDate}
        hasNextPage={page.hasNextPage}
        isAllSelected={bulk.isAllSelected}
        isFetching={page.isFetching}
        isFetchingNextPage={page.isFetchingNextPage}
        isLoading={isLoading}
        isPlaceholderData={page.isPlaceholderData}
        onOpenModal={page.openModal}
        onToggleSelectAll={bulk.toggleSelectAll}
        onToggleSelectRow={bulk.toggleSelect}
        searchInputRef={page.searchInputRef}
        searchTerm={page.searchTerm}
        selectedRowIds={bulk.selectedIds}
        setCustomFieldFilters={page.setCustomFieldFilters}
        setFilterDeptIds={page.setFilterDeptIds}
        setFilterStatus={page.setFilterStatus}
        setSearchTerm={page.setSearchTerm}
        setSelectedEmployee={handleSelectEmployee}
        sortDir={page.sortDir}
        sortKey={page.sortKey}
        toggleDeptFilter={page.toggleDeptFilter}
        toggleSort={page.toggleSort}
        totalItems={page.totalItems}
      />

      <EmployeeOverlays {...page} />

      <EmployeeBulkActionBar
        employees={employees}
        onClearSelection={bulk.clearSelection}
        selectedIds={bulk.selectedIds}
      />

      <FloatingRecordBadge label={t("common.pagination.records")} loaded={employees.length} total={totalItems} />
    </div>
  );
}
