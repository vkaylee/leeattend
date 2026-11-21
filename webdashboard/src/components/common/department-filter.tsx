import type { TFunction } from "i18next";

import type { ControlSize } from "@/components/ui/control-styles";

import { FilterPanel } from "@/components/ui/filter-panel";
import { FilterTrigger } from "@/components/ui/filter-trigger";
import type { Department } from "@/lib/api/types";

interface DepartmentFilterProps {
  departments: Department[];
  effectiveDeptIds: string[];
  handleDeptChange: (ids: string[]) => void;
  size: ControlSize;
  t: TFunction;
}

export function DepartmentFilter({
  departments,
  effectiveDeptIds,
  handleDeptChange,
  size,
  t,
}: DepartmentFilterProps) {
  const options = departments.map((department) => ({
    value: department.id,
    label: department.name,
  }));

  return (
    <FilterPanel
      className="rounded-xl"
      container="popover"
      mode="multi"
      onClear={() => { handleDeptChange([]); }}
      onToggle={(id) => {
        handleDeptChange(
          effectiveDeptIds.includes(id)
            ? effectiveDeptIds.filter((item) => item !== id)
            : [...effectiveDeptIds, id],
        );
      }}
      options={options}
      searchable
      searchPlaceholder={t("common.search")}
      selected={effectiveDeptIds}
      trigger={
        <FilterTrigger
          activeCount={effectiveDeptIds.length}
          data-testid="employee-dept-filter"
          hasActive={effectiveDeptIds.length > 0}
          label={t("common.filter_by_department", "Filter by department")}
          size={size}
          variant="pill-button"
        />
      }
    />
  );
}
