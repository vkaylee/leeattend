import * as Popover from "@radix-ui/react-popover";
import { ChevronLeft, ChevronRight, Upload } from "lucide-react";
import { useTranslation } from "react-i18next";

import { format } from "date-fns";

import { CreateOtRequestDialog } from "@/components/approvals/create-ot-request-dialog";
import { MonthYearPicker } from "@/components/common/month-year-picker";
import Button from "@/components/ui/button";
import { PageHeader } from "@/components/ui/page-header";
import { useDateTimeFormatter } from "@/hooks/use-datetime";

interface OtApprovalsHeaderProps {
  activeTab: "pre" | "post" | "delegated_claim" | "behalf_assignments";
  canApprove: boolean;
  canCreateOnBehalf: boolean;
  currentMonth: Date;
  handleMonthYearChange: (month: number, year: number) => void;
  handleNextMonth: () => void;
  handlePreviousMonth: () => void;
  setBulkUploadDialogOpen: (open: boolean) => void;
}

export function OtApprovalsHeader({
  currentMonth,
  handleMonthYearChange,
  handleNextMonth,
  handlePreviousMonth,
  setBulkUploadDialogOpen,
  activeTab,
  canApprove,
  canCreateOnBehalf,
}: OtApprovalsHeaderProps) {
  const { t } = useTranslation();
  const { formatMonthLongYear } = useDateTimeFormatter();

  return (
    <PageHeader
      actions={
        <div className="flex items-center gap-4 flex-wrap">
          {activeTab === "pre" && canCreateOnBehalf ? <CreateOtRequestDialog /> : null}

          {(activeTab === "post" || activeTab === "delegated_claim") && canApprove ? (
            <Button
              className="normal-case tracking-normal h-10 px-5 rounded-xl gap-2 font-bold text-sm shadow-sm transition-all duration-300 hover:shadow-md hover:-translate-y-0.5 border border-blue-250 dark:border-blue-900 bg-blue-50/50 hover:bg-blue-100 text-blue-900 dark:bg-blue-950/30 dark:hover:bg-blue-900/50 dark:text-blue-200 flex items-center justify-center active:scale-98"
              icon={<Upload className="w-4 h-4" />}
              onClick={() => {
                setBulkUploadDialogOpen(true);
              }}
              variant="ghost"
            >
              {t("approvals.bulk_upload.btn") || "Bulk Upload"}
            </Button>
          ) : null}

          <div className="flex items-center gap-4 bg-white dark:bg-slate-900 p-2 sm:p-2.5 rounded-2xl sm:rounded-[2.5rem] border border-slate-100 dark:border-slate-800 shadow-xl shadow-slate-900/5">
            <div className="flex items-center bg-slate-100 dark:bg-slate-700/50 rounded-[1.5rem] p-1.5 shadow-inner border border-slate-200 dark:border-slate-700/50">
              <Button
                aria-label={t("common.prev_period", { defaultValue: "Kỳ trước" })}
                className="w-12 h-12 flex items-center justify-center text-slate-700 dark:text-slate-300 hover:text-primary-600 dark:hover:text-primary-400 hover:bg-white dark:hover:bg-slate-900 hover:shadow-md rounded-xl transition-all shadow-none"
                icon={<ChevronLeft size={24} />}
                onClick={handlePreviousMonth}
                size="icon"
                variant="ghost"
              />

              <Popover.Root>
                <Popover.Trigger asChild>
                  <Button
                    className="px-10 flex flex-col items-center min-w-[200px] cursor-pointer hover:bg-white dark:hover:bg-slate-900 hover:shadow-md rounded-xl transition-all py-2 group outline-none border border-transparent hover:border-slate-100 dark:border-slate-800 h-auto"
                    variant="ghost"
                  >
                    <span className="text-[9px] font-extrabold text-slate-600 dark:text-slate-400 uppercase tracking-[0.2em] leading-none mb-1.5 group-hover:text-primary-600 dark:group-hover:text-primary-400 transition-colors">
                      {t("payroll.table.period")}
                    </span>
                    <span className="text-sm font-bold text-slate-800 dark:text-slate-100 group-hover:text-primary-600 dark:group-hover:text-primary-400 transition-colors capitalize tracking-tight">
                      {formatMonthLongYear(currentMonth)}
                    </span>
                  </Button>
                </Popover.Trigger>
                <Popover.Portal>
                  <Popover.Content
                    align="center"
                    className="z-50 w-80 p-4 rounded-[2.5rem] shadow-premium-2xl border border-slate-100 dark:border-slate-800 bg-white dark:bg-slate-900 animate-in zoom-in-95 slide-in-from-top-4 origin-top"
                    sideOffset={12}
                  >
                    <MonthYearPicker
                      month={currentMonth.getMonth() + 1}
                      onChange={handleMonthYearChange}
                      year={currentMonth.getFullYear()}
                    />
                  </Popover.Content>
                </Popover.Portal>
              </Popover.Root>

              <Button
                aria-label={t("common.next_period", { defaultValue: "Kỳ tiếp theo" })}
                className="w-12 h-12 flex items-center justify-center text-slate-700 dark:text-slate-300 hover:text-primary-600 dark:hover:text-primary-400 hover:bg-white dark:hover:bg-slate-900 hover:shadow-md rounded-xl transition-all shadow-none"
                icon={<ChevronRight size={24} />}
                onClick={handleNextMonth}
                size="icon"
                variant="ghost"
              />
            </div>
          </div>
        </div>
      }
      subtitle={t("ot_approvals.subtitle")}
      title={t("ot_approvals.title")}
    />
  );
}

