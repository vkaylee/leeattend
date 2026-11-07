import { Building2, UserX } from "lucide-react";

import FloatingRecordBadge from "@/components/common/floating-record-badge";
import { FilterChips } from "@/components/ui/filter-chips";
import { ContractExpirationAlerts } from "./employees/components/contract-expiration-alerts";
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

  // Build filter chips for the generic FilterChips component
  const statusChip = page.filterStatus !== "active" ? [{
    key: `status-${page.filterStatus}`,
    label: page.filterStatus === "terminated"
      ? t("employees.table.terminated", "Đã nghỉ việc")
      : t("common.all", "Tất cả"),
    category: "status",
    icon: <UserX size={10} />,
  }] : [];

  const deptChips = (page.departments ?? [])
    .filter(d => page.filterDeptIds.includes(d.id))
    .map(d => ({
      key: `dept-${d.id}`,
      label: d.name,
      category: "department",
      icon: <Building2 size={10} />,
    }));

  const cfChips = Object.entries(page.customFieldFilters).map(([k, v]) => ({
    key: `cf-${k}`,
    label: `${k}: ${v}`,
    category: "custom",
  }));

  const activeChips = [...statusChip, ...deptChips, ...cfChips];

  return (
    <div className="space-y-6 pb-20 animate-in fade-in duration-700">
      <EmployeesPageHeader
        setEditingEmployee={page.setEditingEmployee}
        setIsImportDialogOpen={page.setIsImportDialogOpen}
        setIsModalOpen={page.setIsModalOpen}
        setIsTerminationImportOpen={page.setIsTerminationImportOpen}
      />

      <ContractExpirationAlerts employees={employees} />

      <FilterChips
        chips={activeChips}
        data-testid="active-filter-chips"
        label={t("employees.filtering_by", "Đang lọc")}
        onClearAll={() => {
          page.setFilterStatus("active");
          page.setFilterDeptIds([]);
          page.setCustomFieldFilters({});
        }}
        onRemove={(key) => {
          if (key.startsWith("status-")) page.setFilterStatus("active");
          else if (key.startsWith("dept-")) page.toggleDeptFilter(key.replace("dept-", ""));
          else if (key.startsWith("cf-")) page.updateCustomFieldFilter(key.replace("cf-", ""), "");
        }}
      />

      <EmployeeTable
        customFieldFilters={page.customFieldFilters}
        departments={page.departments}
        employees={page.employees}
        fetchNextPage={page.fetchNextPage}
        filterDeptIds={page.filterDeptIds}
        filterStatus={page.filterStatus}
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
        setFilterStatus={page.setFilterStatus}
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
