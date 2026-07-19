import * as Popover from "@radix-ui/react-popover";
import { Filter, Check } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import FormField from "@/components/ui/form-field";
import { useCustomFieldDefinitions } from "@/hooks/use-custom-fields";
import { cn } from "@/lib/utils";

interface CustomFieldsFilterPopoverProps {
  customFieldFilters: Record<string, string>;
  setCustomFieldFilters: (filters: Record<string, string>) => void;
}

export function CustomFieldsFilterPopover({
  customFieldFilters,
  setCustomFieldFilters,
}: CustomFieldsFilterPopoverProps) {
  const { t } = useTranslation(["custom_fields", "common"]);

  // Fetch definitions for employee entity
  const { data: definitions = [] } = useCustomFieldDefinitions("employee");

  // Filter only those marked as filterable
  const filterableDefs = definitions.filter((def) => def.is_filterable);

  const [localFilters, setLocalFilters] = useState<Record<string, string>>({});
  const [isOpen, setIsOpen] = useState(false);

  const handleOpenChange = (open: boolean) => {
    setIsOpen(open);
    if (open) {
      setLocalFilters(customFieldFilters);
    }
  };

  if (filterableDefs.length === 0) return null;

  const handleApply = () => {
    setCustomFieldFilters(localFilters);
    setIsOpen(false);
  };

  const handleClear = () => {
    setLocalFilters({});
    setCustomFieldFilters({});
    setIsOpen(false);
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
    <Popover.Root onOpenChange={handleOpenChange} open={isOpen}>
      <Popover.Trigger asChild>
        <Button
          className={cn(
            "relative flex items-center gap-2 rounded-2xl h-[56px] px-6 transition-all font-bold text-xs uppercase tracking-widest",
            activeCount > 0
              ? "bg-primary-50 dark:bg-primary-950 text-primary-600 dark:text-primary-400 border border-primary-200 dark:border-primary-800"
              : "bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 text-slate-600 dark:text-slate-400 hover:bg-slate-50 dark:hover:bg-slate-800"
          )}
          variant="outline"
        >
          <Filter size={14} />
          <span>{t("filter_title")}</span>
          {activeCount > 0 && (
            <span className="min-w-[18px] h-[18px] px-1 flex items-center justify-center bg-primary-600 text-white text-[9px] font-bold rounded-full leading-none">
              {activeCount}
            </span>
          )}
        </Button>
      </Popover.Trigger>
      <Popover.Portal>
        <Popover.Content
          align="end"
          className="z-[10000] w-80 overflow-hidden rounded-[2rem] border border-slate-100 dark:border-slate-800 bg-white dark:bg-slate-900 p-6 shadow-2xl dark:shadow-black/40 animate-in fade-in zoom-in-95 duration-200"
          sideOffset={8}
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
                onChange={(e) => { handleChange(def.field_key, e.target.value); }}
                placeholder={t("search_placeholder", { label: def.field_label })}
                value={localFilters[def.field_key] ?? ""}
              />
            ))}
          </div>

          <div className="pt-4 border-t border-slate-50 dark:border-slate-800 flex justify-end gap-2">
            <Button onClick={() => { setIsOpen(false); }} size="sm" variant="ghost">
              {t("common.cancel")}
            </Button>
            <Button
              icon={<Check size={12} />}
              onClick={handleApply}
              size="sm"
              variant="primary"
            >
              {t("apply")}
            </Button>
          </div>
        </Popover.Content>
      </Popover.Portal>
    </Popover.Root>
  );
}
