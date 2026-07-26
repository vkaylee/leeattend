/* eslint-disable @typescript-eslint/no-non-null-assertion */
import { useQuery } from "@tanstack/react-query";
import React, { useMemo } from "react";
import { useTranslation } from "react-i18next";

import { FilterPanel } from "@/components/ui/filter-panel";
import { FilterTrigger } from "@/components/ui/filter-trigger";
import { fetchShifts, getCurrentWorkspaceId  } from "@/lib/api";

/**
 * Multi-select shift-type filter built on FilterPanel (the existing `Combobox`
 * is single-select only). Options come from the shift catalogue via `fetchShifts`,
 * de-duplicated by name. Selected names are sent to the backend as `shift_type`.
 */
export function ShiftTypePicker({
  selected,
  onChange,
}: {
  selected: string[];
  onChange: (names: string[]) => void;
}) {
  const { t } = useTranslation();
  const wsId = getCurrentWorkspaceId();

  const { data: shifts = [] } = useQuery({
    queryKey: ["shifts-catalogue", wsId],
    queryFn: () => fetchShifts(wsId!),
    enabled: !!wsId,
    staleTime: 5 * 60 * 1000,
  });

  // De-duplicated by name, sorted, active shifts first.
  const options = useMemo(() => {
    const byName = new Map<string, { name: string; active: boolean }>();
    for (const s of shifts) {
      const existing = byName.get(s.name);
      if (existing?.active) continue;
      byName.set(s.name, { name: s.name, active: s.is_active });
    }
    return Array.from(byName.values()).sort((a, b) => {
      if (a.active !== b.active) return a.active ? -1 : 1;
      return a.name.localeCompare(b.name);
    });
  }, [shifts]);

  const toggle = (name: string) => {
    onChange(
      selected.includes(name)
        ? selected.filter((n) => n !== name)
        : [...selected, name],
    );
  };

  return (
    <FilterPanel
      container="popover"
      emptyMessage={t("unrecorded_shifts.filter_shift_type_empty", {
        defaultValue: "No shift types",
      })}
      mode="multi"
      onClear={() => { onChange([]); }}
      onToggle={toggle}
      options={options.map(opt => ({ value: opt.name, label: opt.name }))}
      searchable
      selected={selected}
      trigger={
        <FilterTrigger
          activeCount={selected.length}
          aria-haspopup="listbox"
          chevron
          hasActive={selected.length > 0}
          id="unrecorded-shift-type-picker"
          label={t("unrecorded_shifts.filter_shift_type")}
          variant="pill-button"
        />
      }
    />
  );
}
