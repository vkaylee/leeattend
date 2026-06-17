import { Filter } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuLabel,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { cn } from "@/lib/utils";

import type { AttendanceStatusFilter } from "../../hooks/use-attendance-adjustments-page";

interface AdjustmentsTableHeaderProps {
  statusFilter: AttendanceStatusFilter;
  setStatusFilter: (filter: AttendanceStatusFilter) => void;
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
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <div role="button" className={cn(
                  "flex items-center gap-1.5 focus:outline-none transition-colors group cursor-pointer px-3 py-1.5 rounded-xl border border-transparent",
                  statusFilter !== "all"
                    ? "text-primary-600 dark:text-primary-450 bg-primary-50/50 dark:bg-primary-950/30 border-primary-150/50 dark:border-primary-900/50 font-extrabold"
                    : "text-slate-500 dark:text-slate-400 hover:text-slate-850 dark:hover:text-slate-200"
                )}>
                  <span className="flex items-center gap-1.5">
                    {t("common.status")}
                    {statusFilter !== "all" && (
                      <span className="text-[8px] font-extrabold text-primary-500 dark:text-primary-400 normal-case bg-primary-100/50 dark:bg-primary-900/40 px-1.5 py-0.5 rounded-md">
                        {t(`common.status_${statusFilter}`).toUpperCase()}
                      </span>
                    )}
                  </span>
                  <Filter
                    size={11}
                    className={cn(
                      "text-slate-400 group-hover:text-slate-600 dark:group-hover:text-slate-350 transition-colors",
                      statusFilter !== "all" && "text-primary-500 dark:text-primary-400 fill-primary-500/20"
                    )}
                  />
                </div>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="center" className="w-52">
                <DropdownMenuLabel>{t("common.filter_by_status", "Filter by Status").toUpperCase()}</DropdownMenuLabel>
                <DropdownMenuSeparator />
                <DropdownMenuRadioGroup value={statusFilter} onValueChange={(val) => setStatusFilter(val as AttendanceStatusFilter)}>
                  <DropdownMenuRadioItem value="all">{t("common.all", "All").toUpperCase()}</DropdownMenuRadioItem>
                  <DropdownMenuRadioItem value="pending">{t("common.status_pending", "Pending").toUpperCase()}</DropdownMenuRadioItem>
                  <DropdownMenuRadioItem value="approved">{t("common.status_approved", "Approved").toUpperCase()}</DropdownMenuRadioItem>
                  <DropdownMenuRadioItem value="rejected">{t("common.status_rejected", "Rejected").toUpperCase()}</DropdownMenuRadioItem>
                </DropdownMenuRadioGroup>
              </DropdownMenuContent>
            </DropdownMenu>
          </div>
        </th>
        <th className="px-10 py-6 border-b border-slate-100 dark:border-slate-800"></th>
      </tr>
    </thead>
  );
}
