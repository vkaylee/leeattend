import { Filter } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { FilterPanel } from "@/components/ui/filter-panel";
import { FilterTrigger } from "@/components/ui/filter-trigger";
import FormField from "@/components/ui/form-field";
import { useCustomFieldDefinitions } from "@/hooks/use-custom-fields";
import { useMyPermissions } from "@/hooks/use-permissions";

interface CustomFieldsFilterPopoverProps {
  customFieldFilters: Record<string, string>;
  setCustomFieldFilters: (filters: Record<string, string>) => void;
}

export function CustomFieldsFilterPopover({
  customFieldFilters,
  setCustomFieldFilters,
}: CustomFieldsFilterPopoverProps) {
  const { t } = useTranslation(["custom_fields", "common"]);
  const { hasPermission } = useMyPermissions();

  // Fetch definitions for employee entity
  const { data: definitions = [] } = useCustomFieldDefinitions("employee", hasPermission("employees:read"));

  // Filter only those marked as filterable
  const filterableDefs = definitions.filter((def) => def.is_filterable);

  const [localFilters, setLocalFilters] = useState<Record<string, string>>(customFieldFilters);

  if (filterableDefs.length === 0) return null;

  const handleClear = () => {
    setLocalFilters({});
    setCustomFieldFilters({});
  };

  const handleChange = (key: string, val: string) => {
    setLocalFilters((prev) => {
      const next = { ...prev };
      if (!val.trim()) {
        // eslint-disable-next-line @typescript-eslint/no-dynamic-delete
        delete next[key];
      } else {
        next[key] = val;
      }
      return next;
    });
  };

  const activeCount = Object.keys(customFieldFilters).length;

  return (
    <FilterPanel
      className="w-80 p-6"
      container="popover"
      mode="multi"
      onApply={() => {
        setCustomFieldFilters(localFilters);
      }}
      onClear={handleClear}
      // eslint-disable-next-line @typescript-eslint/no-empty-function -- children manages its own form state
      onToggle={() => {}}
      options={[]}
      searchable
      selected={Object.keys(customFieldFilters)}
      showApplyButton
      trigger={
        <FilterTrigger
          activeCount={activeCount}
          hasActive={activeCount > 0}
          icon={<Filter size={14} />}
          label={t("filter_title")}
          variant="pill-button"
        />
      }
    >
      <div className="flex justify-between items-center pb-4 border-b border-slate-50 dark:border-slate-800">
        <span className="text-[10px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-[0.2em]">
          {t("filter_header")}
        </span>
        {activeCount > 0 && (
          <Button
            className="text-[10px] font-bold text-red-500 dark:text-red-400 hover:underline uppercase tracking-wide cursor-pointer h-auto p-0"
            onClick={handleClear}
            variant="ghost"
          >
            {t("common.clear")}
          </Button>
        )}
      </div>
      <div className="space-y-4 py-6 max-h-80 overflow-y-auto custom-scrollbar">
        {filterableDefs.map((def) => (
          <FormField
            key={def.id}
            label={def.field_label}
            name={def.field_key}
            onChange={(e) => {
              handleChange(def.field_key, e.target.value);
            }}
            placeholder={t("search_placeholder", { label: def.field_label })}
            value={localFilters[def.field_key] ?? ""}
          />
        ))}
      </div>
    </FilterPanel>
  );
}
