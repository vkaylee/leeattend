import { Search, X } from "lucide-react";
import React, { useState, useCallback, useMemo } from "react";
import { useTranslation } from "react-i18next";

import { Command } from "cmdk";

import Button from "@/components/ui/button";

import { MultiOptionRow, SingleOptionRow } from "./filter-field-renderer";
import type { OptionListProps } from "./types";

// ── Option List ──────────────────────────────────────────────────────────

export function OptionList({
  options,
  selected,
  mode,
  searchable,
  searchPlaceholder,
  emptyMessage,
  showClearButton,
  onToggle,
  onClear,
  testId,
}: OptionListProps) {
  const { t } = useTranslation();
  const [search, setSearch] = useState("");

  const filteredOptions = useMemo(() => {
    if (!searchable || !search.trim()) return options;
    const q = search.toLowerCase();
    return options.filter((opt) => opt.label.toLowerCase().includes(q));
  }, [options, search, searchable]);

  const handleToggle = useCallback(
    (value: string) => {
      onToggle(value);
    },
    [onToggle],
  );

  return (
    <>
      {searchable ? (
        <Command>
          <div className="flex items-center border-b border-slate-50 dark:border-slate-800 px-3 pb-3 mb-1">
            <Search className="mr-3 h-4 w-4 shrink-0 text-slate-300 dark:text-slate-500" />
            <Command.Input
              className="flex h-10 w-full rounded-md bg-transparent text-sm text-primary-700 dark:text-primary-300 outline-none placeholder:text-slate-300 dark:placeholder:text-slate-600 font-bold"
              onValueChange={setSearch}
              placeholder={searchPlaceholder ?? t("common.combobox.search")}
              value={search}
            />
          </div>
        </Command>
      ) : null}

      {showClearButton && selected.length > 0 ? (
        <Button
          className="h-auto w-full px-3 py-2 flex items-center gap-3 rounded-xl text-[11px] font-bold uppercase tracking-wide text-red-500 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-950/30 transition-colors mb-1 border-none shadow-none justify-start"
          data-testid={testId ? `${testId}-clear` : undefined}
          onClick={onClear}
          type="button"
          variant="ghost"
        >
          <X className="shrink-0" size={12} />
          {t("common.clear_filter", "Xóa bộ lọc")}
        </Button>
      ) : null}

      <div className="max-h-60 overflow-y-auto custom-scrollbar">
        {filteredOptions.length === 0 ? (
          <div className="px-3 py-6 text-center text-[10px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-widest opacity-60">
            {emptyMessage ?? t("common.no_options", "Không có tùy chọn")}
          </div>
        ) : (
          filteredOptions.map((opt) => {
            const checked = selected.includes(opt.value);
            if (mode === "multi") {
              return (
                <MultiOptionRow
                  checked={checked}
                  key={opt.value}
                  onToggle={() => { handleToggle(opt.value); }}
                  option={opt}
                  testId={testId}
                />
              );
            }
            return (
              <SingleOptionRow
                checked={checked}
                key={opt.value}
                onSelect={() => { handleToggle(opt.value); }}
                option={opt}
                testId={testId}
              />
            );
          })
        )}
      </div>
    </>
  );
}
