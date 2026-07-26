import { Filter } from "lucide-react";
import { useTranslation } from "react-i18next";

import type { LeaveStatusFilter } from "@/components/features/leave/use-leave-requests";

import { FilterPanel } from "@/components/ui/filter-panel";
import { FilterTrigger } from "@/components/ui/filter-trigger";

interface LeaveStatusFilterHeaderProps {
  setStatusFilter: (value: LeaveStatusFilter) => void;
  statusFilter: LeaveStatusFilter;
}

export function LeaveStatusFilterHeader({ statusFilter, setStatusFilter }: LeaveStatusFilterHeaderProps) {
  const { t } = useTranslation();

  return (
    <th className="py-6 text-[10px] font-bold uppercase tracking-[0.25em] text-center px-8" key="status-header">
      <div className="flex justify-center">
        <FilterPanel
          container="dropdown"
          data-testid="leave-status-filter"
          mode="single"
          onClear={() => { setStatusFilter("all"); }}
          onToggle={(val) => { setStatusFilter(val as LeaveStatusFilter); }}
          options={[
            { value: "all", label: t("common.all") },
            { value: "pending", label: t("common.status_pending") },
            { value: "approved", label: t("common.status_approved") },
            { value: "pre_approved", label: t("common.status_pre_approved") },
            { value: "rejected", label: t("common.status_rejected") },
          ]}
          selected={statusFilter !== "all" ? [statusFilter] : []}
          trigger={
            <FilterTrigger
              data-testid="leave-status-filter-btn"
              hasActive={statusFilter !== "all"}
              icon={<Filter size={14} />}
              label={t("common.filter_by_status")}
              variant="icon-button"
            />
          }
        />
      </div>
    </th>
  );
}
