import { Filter } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { Combobox } from "@/components/ui/combobox";
import { SearchInput } from "@/components/ui/search-input";

export interface AssignmentsFiltersProps {
  onClearFilters?: () => void;
  onSearchChange: (value: string) => void;
  onStatusChange: (value: string) => void;
  search: string;
  selectedStatus: string;
}

export function AssignmentsFilters({
  search,
  onSearchChange,
  selectedStatus,
  onStatusChange,
}: AssignmentsFiltersProps) {
  const { t } = useTranslation();

  return (
    <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 p-6 rounded-[2rem] shadow-xs flex flex-col md:flex-row items-center justify-between gap-4">
      <div className="w-full md:w-80">
        <SearchInput
          aria-label={t("assignments.search_label", { defaultValue: "Tìm kiếm phái cử" })}
          containerClassName="h-10"
          onChange={(e) => { onSearchChange(e.target.value); }}
          onClear={() => { onSearchChange(""); }}
          placeholder={t("assignments.search_placeholder", { defaultValue: "Tìm nhân sự, mã NV, ghi chú..." })}
          size="sm"
          value={search}
        />
      </div>
      <div className="w-full md:w-auto flex flex-wrap md:flex-nowrap items-center gap-3">
        <div className="flex items-center gap-2">
          <span className="text-[10px] font-bold text-slate-400 uppercase tracking-wider flex items-center gap-1">
            <Filter size={12} /> {t("common.status", { defaultValue: "TRẠNG THÁI" })}
          </span>
          <Combobox
            className="h-10 text-xs bg-slate-50 dark:bg-slate-855 border border-slate-200 dark:border-slate-800 rounded-xl text-slate-600 dark:text-slate-350 w-48"
            onChange={onStatusChange}
            options={[
              { value: "all", label: t("assignments.status.all", { defaultValue: "TẤT CẢ TRẠNG THÁI" }) },
              { value: "active", label: t("assignments.status.active", { defaultValue: "ĐANG HIỆU LỰC" }) },
              { value: "scheduled", label: t("assignments.status.scheduled", { defaultValue: "ĐÃ LÊN LỊCH" }) },
              { value: "completed", label: t("assignments.status.completed", { defaultValue: "ĐÃ HOÀN THÀNH" }) },
              { value: "cancelled", label: t("assignments.status.cancelled", { defaultValue: "ĐÃ HỦY BỎ" }) },
            ]}
            placeholder={t("assignments.status.all", { defaultValue: "TẤT CẢ TRẠNG THÁI" })}
            value={selectedStatus}
          />
        </div>
      </div>
    </div>
  );
}
