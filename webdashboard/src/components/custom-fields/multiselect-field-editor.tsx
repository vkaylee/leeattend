import React from "react";
import { useTranslation } from "react-i18next";

import type { CustomFieldDefinition } from "@/lib/api/types";

import Button from "@/components/ui/button";
import { FilterChips } from "@/components/ui/filter-chips";
import { FilterPanel } from "@/components/ui/filter-panel";
import { Typography } from "@/components/ui/typography";

interface MultiselectFieldEditorProps {
  definition: CustomFieldDefinition;
  error?: string;
  onChange: (value: unknown) => void;
  options: { value: string; label: string }[];
  value: unknown;
}

export const MultiselectFieldEditor: React.FC<MultiselectFieldEditorProps> = ({
  definition,
  value,
  onChange,
  options,
  error,
}) => {
  const { t } = useTranslation("custom_fields");

  const getSelectedValues = (): string[] => {
    if (Array.isArray(value)) {
      return value.map(String);
    }
    if (typeof value === "string") {
      try {
        const parsed: unknown = JSON.parse(value);
        if (Array.isArray(parsed)) return parsed.map(String);
      } catch { /* JSON.parse on string value — fall through to single-value fallback */
        return value ? [value] : [];
      }
    }
    return [];
  };

  const selected = getSelectedValues();

  const handleToggle = (optValue: string) => {
    const next = selected.includes(optValue)
      ? selected.filter((v) => v !== optValue)
      : [...selected, optValue];
    onChange(next);
  };

  const fieldId = `cf-${definition.field_key}`;

  return (
    <div className="flex flex-col gap-3 px-1">
      <label
        className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1"
        htmlFor={fieldId}
      >
        {definition.field_label}
        {definition.is_required ? <span className="text-rose-500 ml-0.5">*</span> : null}
      </label>

      <FilterPanel
        container="popover"
        emptyMessage={t("common.no_options", "Không có tùy chọn")}
        mode="multi"
        onClear={() => { onChange([]); }}
        onToggle={handleToggle}
        options={options.map((opt) => ({ value: opt.value, label: opt.label }))}
        selected={selected}
        trigger={
          <Button
            className="w-full justify-start min-h-[var(--ui-height-md)] h-auto px-3 py-2"
            data-testid="multiselect-trigger"
            id={fieldId}
            type="button"
            variant="outline"
          >
            {selected.length === 0 ? (
              <span className="text-slate-400 text-sm">{t("select_options")}</span>
            ) : (
              <FilterChips
                chips={selected.map((v) => {
                  const opt = options.find((o) => o.value === v);
                  return { key: v, label: opt?.label ?? v };
                })}
                onClearAll={() => { onChange([]); }}
                onRemove={(key) => { handleToggle(key); }}
                showClearAll={false}
              />
            )}
          </Button>
        }
      />

      {!!error && (
        <Typography className="text-xs text-rose-500 font-medium px-1 mt-1" variant="p">
          {error}
        </Typography>
      )}
    </div>
  );
};
