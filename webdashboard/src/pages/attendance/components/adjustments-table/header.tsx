 
import { Filter } from "lucide-react";
import { useTranslation } from "react-i18next";

import { FilterPanel } from "@/components/ui/filter-panel";
import { FilterTrigger } from "@/components/ui/filter-trigger";

import type { AttendanceStatusFilter } from "../../hooks/use-attendance-adjustments-page";

interface AdjustmentsTableHeaderProps {
  setStatusFilter: (filter: AttendanceStatusFilter) => void;
  statusFilter: AttendanceStatusFilter;
}

export function AdjustmentsTableHeader({ statusFilter, setStatusFilter }: AdjustmentsTableHeaderProps) {
  const { t } = useTranslation();

  return (
    <thead>
      <tr className="bg-slate-50 dark:bg-slate-800/50 border-b border-slate-100 dark:border-slate-800">
        <th className="px-10 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em]">
          {t("common.employee")}
        </th>
        <th className="px-8 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] text-center">
          {t("common.time")}
        </th>
        <th className="px-8 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em]">
          {t("common.notes")}
        </th>
        <th className="px-8 py-6 text-[10px] font-bold uppercase tracking-[0.25em] text-center">
          <div className="flex justify-center">
            <FilterPanel
              container="dropdown"
              data-testid="adjustments-status-filter"
              mode="single"
              onClear={() => { setStatusFilter("all"); }}
              onToggle={(val) => { setStatusFilter(val as AttendanceStatusFilter); }}
              options={[
                { value: "all", label: t("common.all") },
                { value: "pending", label: t("adjustments.status.pending") },
                { value: "approved", label: t("adjustments.status.approved") },
                { value: "rejected", label: t("adjustments.status.rejected") },
              ]}
              selected={statusFilter !== "all" ? [statusFilter] : []}
              trigger={
                <FilterTrigger
                  hasActive={statusFilter !== "all"}
                  icon={<Filter size={14} />}
                  label={t("common.filter_by_status")}
                  variant="icon-button"
                />
              }
            />
          </div>
        </th>
        <th className="px-10 py-6 border-b border-slate-100 dark:border-slate-800"></th>
      </tr>
    </thead>
  );
}
