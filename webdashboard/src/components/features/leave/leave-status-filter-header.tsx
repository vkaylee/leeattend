import { Filter } from "lucide-react";
import { useTranslation } from "react-i18next";

import type { LeaveStatusFilter } from "@/components/features/leave/use-leave-requests";

import Button from "@/components/ui/button";
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

interface LeaveStatusFilterHeaderProps {
  setStatusFilter: (value: LeaveStatusFilter) => void;
  statusFilter: LeaveStatusFilter;
}

export function LeaveStatusFilterHeader({ statusFilter, setStatusFilter }: LeaveStatusFilterHeaderProps) {
  const { t } = useTranslation();

  return (
    <th className="py-6 text-[10px] font-bold uppercase tracking-[0.25em] text-center px-8" key="status-header">
      <div className="flex justify-center">
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button
              className={cn(
                "flex items-center gap-1.5 h-auto min-w-0 transition-colors group cursor-pointer px-3 py-1.5 rounded-xl border border-transparent bg-transparent font-bold text-[10px] uppercase tracking-widest hover:bg-slate-50 dark:hover:bg-slate-800/50",
                statusFilter !== "all"
                  ? "text-primary-600 dark:text-primary-450 bg-primary-50/50 dark:bg-primary-950/30 border-primary-150/50 dark:border-primary-900/50 font-extrabold"
                  : "text-slate-500 dark:text-slate-400 hover:text-slate-850 dark:hover:text-slate-200"
              )}
              type="button"
              variant="ghost"
            >
              <span className="flex items-center gap-1.5">
                {t("common.status")}
                {statusFilter !== "all" && (
                  <span className="text-[8px] font-extrabold text-primary-500 dark:text-primary-400 normal-case bg-primary-100/50 dark:bg-primary-900/40 px-1.5 py-0.5 rounded-md">
                    {t(`common.status_${statusFilter}`).toUpperCase()}
                  </span>
                )}
              </span>
              <Filter
                className={cn(
                  "text-slate-400 group-hover:text-slate-600 dark:group-hover:text-slate-350 transition-colors",
                  statusFilter !== "all" && "text-primary-500 dark:text-primary-400 fill-primary-500/20"
                )}
                size={11}
              />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="center" className="w-52">
            <DropdownMenuLabel>{t("common.filter_by_status", "Filter by Status").toUpperCase()}</DropdownMenuLabel>
            <DropdownMenuSeparator />
            <DropdownMenuRadioGroup onValueChange={(val) => { setStatusFilter(val as LeaveStatusFilter); }} value={statusFilter}>
              <DropdownMenuRadioItem value="all">{t("common.all", "All").toUpperCase()}</DropdownMenuRadioItem>
              <DropdownMenuRadioItem value="pending">{t("common.status_pending", "Pending").toUpperCase()}</DropdownMenuRadioItem>
              <DropdownMenuRadioItem value="approved">{t("common.status_approved", "Approved").toUpperCase()}</DropdownMenuRadioItem>
              <DropdownMenuRadioItem value="pre_approved">{t("common.status_pre_approved", "Pre-approved").toUpperCase()}</DropdownMenuRadioItem>
              <DropdownMenuRadioItem value="rejected">{t("common.status_rejected", "Rejected").toUpperCase()}</DropdownMenuRadioItem>
            </DropdownMenuRadioGroup>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>
    </th>
  );
}
