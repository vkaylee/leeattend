import { Filter } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import type { ConditionTypeDefinition } from "@/lib/api";

import { Combobox } from "@/components/ui/combobox";
import { SearchInput } from "@/components/ui/search-input";

export interface WorkingConditionsFiltersProps {
  onClearFilters?: () => void;
  onPaymentFilterChange: (value: string) => void;
  onSearchChange: (value: string) => void;
  onTypeChange: (value: string) => void;
  paymentFilter: string;
  search: string;
  selectedType: string;
  typeDefs: ConditionTypeDefinition[];
}

export function WorkingConditionsFilters({
  search,
  onSearchChange,
  selectedType,
  onTypeChange,
  paymentFilter,
  onPaymentFilterChange,
  typeDefs,
}: WorkingConditionsFiltersProps) {
  const { t, i18n } = useTranslation();
  const currentLang = i18n.language.split("-")[0];

  const typeOptions = [
    { value: "all", label: t("working_conditions.filters.all_types", { defaultValue: "TẤT CẢ LOẠI ĐIỀU KIỆN" }) },
    ...typeDefs.map((def) => ({
      value: def.code,
      label: (def.name_i18n?.[currentLang] || def.name_i18n?.en || def.code).toUpperCase(),
    })),
  ];

  return (
    <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 p-6 rounded-[2rem] shadow-xs flex flex-col md:flex-row items-center justify-between gap-4">
      <div className="w-full md:w-80">
        <SearchInput
          aria-label={t("working_conditions.search_label", { defaultValue: "Tìm kiếm điều kiện làm việc" })}
          containerClassName="h-10"
          onChange={(e) => { onSearchChange(e.target.value); }}
          onClear={() => { onSearchChange(""); }}
          placeholder={t("working_conditions.search_placeholder", {
            defaultValue: "Tìm nhân sự, mã NV, loại chế độ...",
          })}
          size="sm"
          value={search}
        />
      </div>

      <div className="w-full md:w-auto flex flex-wrap md:flex-nowrap items-center gap-3">
        {/* Type Filter */}
        <div className="flex items-center gap-2">
          <span className="text-[10px] font-bold text-slate-400 uppercase tracking-wider flex items-center gap-1">
            <Filter size={12} /> {t("common.type", { defaultValue: "LOẠI" })}
          </span>
          <Combobox
            className="h-10 text-xs bg-slate-50 dark:bg-slate-855 border border-slate-200 dark:border-slate-800 rounded-xl text-slate-600 dark:text-slate-350 w-52"
            onChange={onTypeChange}
            options={typeOptions}
            placeholder={t("working_conditions.filters.all_types", { defaultValue: "TẤT CẢ LOẠI ĐIỀU KIỆN" })}
            value={selectedType}
          />
        </div>

        {/* Payment Regime Filter */}
        <div className="flex items-center gap-2">
          <span className="text-[10px] font-bold text-slate-400 uppercase tracking-wider flex items-center gap-1">
            <Filter size={12} /> {t("working_conditions.filters.payment", { defaultValue: "CHẾ ĐỘ LƯƠNG" })}
          </span>
          <Combobox
            className="h-10 text-xs bg-slate-50 dark:bg-slate-855 border border-slate-200 dark:border-slate-800 rounded-xl text-slate-600 dark:text-slate-350 w-48"
            onChange={onPaymentFilterChange}
            options={[
              { value: "all", label: t("working_conditions.filters.all_payment", { defaultValue: "TẤT CẢ CHẾ ĐỘ" }) },
              { value: "paid", label: t("working_conditions.paid", { defaultValue: "Có hưởng lương" }) },
              { value: "unpaid", label: t("working_conditions.unpaid", { defaultValue: "Không hưởng lương" }) },
            ]}
            placeholder={t("working_conditions.filters.all_payment", { defaultValue: "TẤT CẢ CHẾ ĐỘ" })}
            value={paymentFilter}
          />
        </div>
      </div>
    </div>
  );
}
