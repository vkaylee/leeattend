import { Filter } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { Combobox } from "@/components/ui/combobox";
import { SearchInput } from "@/components/ui/search-input";

export interface LifecycleFiltersProps {
  onClearFilters?: () => void;
  onSearchChange: (value: string) => void;
  onStatusChange: (value: string) => void;
  onTypeChange: (value: string) => void;
  search: string;
  selectedStatus: string;
  selectedType: string;
}

export function LifecycleFilters({
  search,
  onSearchChange,
  selectedType,
  onTypeChange,
  selectedStatus,
  onStatusChange,
}: LifecycleFiltersProps) {
  const { t } = useTranslation();

  const typeOptions = [
    { value: "all", label: t("lifecycle.filters.all_types", { defaultValue: "TẤT CẢ QUY TRÌNH" }) },
    { value: "onboarding", label: t("lifecycle.types.onboarding", { defaultValue: "Onboarding (Tiếp nhận)" }) },
    { value: "probation_end", label: t("lifecycle.types.probation", { defaultValue: "Thử việc (Đánh giá)" }) },
    { value: "offboarding", label: t("lifecycle.types.offboarding", { defaultValue: "Offboarding (Nghỉ việc)" }) },
    { value: "transfer", label: t("lifecycle.types.transfer", { defaultValue: "Điều chuyển (Luân chuyển)" }) },
  ];

  const statusOptions = [
    { value: "all", label: t("lifecycle.filters.all_statuses", { defaultValue: "TẤT CẢ TRẠNG THÁI" }) },
    { value: "in_progress", label: t("lifecycle.status.in_progress", { defaultValue: "Đang tiến hành" }) },
    { value: "completed", label: t("lifecycle.status.completed", { defaultValue: "Đã hoàn thành" }) },
  ];

  return (
    <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 p-6 rounded-[2rem] shadow-xs flex flex-col md:flex-row items-center justify-between gap-4">
      <div className="w-full md:w-80">
        <SearchInput
          aria-label={t("lifecycle.search_label", { defaultValue: "Tìm kiếm quy trình nhân sự" })}
          containerClassName="h-10"
          onChange={(e) => { onSearchChange(e.target.value); }}
          onClear={() => { onSearchChange(""); }}
          placeholder={t("lifecycle.search_placeholder", {
            defaultValue: "Tìm nhân sự, mã NV, phòng ban, mẫu checklist...",
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
            placeholder={t("lifecycle.filters.all_types", { defaultValue: "TẤT CẢ QUY TRÌNH" })}
            value={selectedType}
          />
        </div>

        {/* Status Filter */}
        <div className="flex items-center gap-2">
          <span className="text-[10px] font-bold text-slate-400 uppercase tracking-wider flex items-center gap-1">
            <Filter size={12} /> {t("common.status", { defaultValue: "TRẠNG THÁI" })}
          </span>
          <Combobox
            className="h-10 text-xs bg-slate-50 dark:bg-slate-855 border border-slate-200 dark:border-slate-800 rounded-xl text-slate-600 dark:text-slate-350 w-44"
            onChange={onStatusChange}
            options={statusOptions}
            placeholder={t("lifecycle.filters.all_statuses", { defaultValue: "TẤT CẢ TRẠNG THÁI" })}
            value={selectedStatus}
          />
        </div>
      </div>
    </div>
  );
}
