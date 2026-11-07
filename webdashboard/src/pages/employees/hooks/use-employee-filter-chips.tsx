import { Building2, UserX } from "lucide-react";
import React, { useMemo } from "react";

import type { FilterChipItem } from "@/components/ui/filter-chips";
import type { Department } from "@/lib/api/types";

interface UseEmployeeFilterChipsProps {
  customFieldFilters: Record<string, string>;
  departments: Department[] | undefined;
  filterDeptIds: string[];
  filterStatus: string;
  setCustomFieldFilters: (filters: Record<string, string>) => void;
  setFilterDeptIds: (ids: string[]) => void;
  setFilterStatus: (status: string) => void;
  t: (key: string, defaultValue?: string) => string;
  toggleDeptFilter: (id: string) => void;
  updateCustomFieldFilter: (key: string, val: string) => void;
}

export function useEmployeeFilterChips({
  departments,
  filterDeptIds,
  filterStatus,
  customFieldFilters,
  setFilterStatus,
  setFilterDeptIds,
  setCustomFieldFilters,
  toggleDeptFilter,
  updateCustomFieldFilter,
  t,
}: UseEmployeeFilterChipsProps) {
  const activeChips = useMemo<FilterChipItem[]>(() => {
    const statusChip: FilterChipItem[] = filterStatus !== "active" ? [{
      key: `status-${filterStatus}`,
      label: filterStatus === "terminated"
        ? t("employees.table.terminated", "Đã nghỉ việc")
        : t("common.all", "Tất cả"),
      category: "status",
      icon: <UserX size={10} />,
    }] : [];

    const deptChips: FilterChipItem[] = (departments ?? [])
      .filter(d => filterDeptIds.includes(d.id))
      .map(d => ({
        key: `dept-${d.id}`,
        label: d.name,
        category: "department",
        icon: <Building2 size={10} />,
      }));

    const cfChips: FilterChipItem[] = Object.entries(customFieldFilters).map(([k, v]) => ({
      key: `cf-${k}`,
      label: `${k}: ${v}`,
      category: "custom",
    }));

    return [...statusChip, ...deptChips, ...cfChips];
  }, [departments, filterDeptIds, filterStatus, customFieldFilters, t]);

  const handleClearAll = () => {
    setFilterStatus("active");
    setFilterDeptIds([]);
    setCustomFieldFilters({});
  };

  const handleRemove = (key: string) => {
    if (key.startsWith("status-")) {
      setFilterStatus("active");
    } else if (key.startsWith("dept-")) {
      toggleDeptFilter(key.replace("dept-", ""));
    } else if (key.startsWith("cf-")) {
      updateCustomFieldFilter(key.replace("cf-", ""), "");
    }
  };

  return { activeChips, handleClearAll, handleRemove };
}
