import * as Popover from "@radix-ui/react-popover";
import { ChevronLeft, ChevronRight, FileText, Upload } from "lucide-react";
import { useTranslation } from "react-i18next";

import { format } from "date-fns";

import { CreateOtRequestDialog } from "@/components/approvals/create-ot-request-dialog";
import { MonthYearPicker } from "@/components/common/month-year-picker";
import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";

interface OtApprovalsHeaderProps {
  currentMonth: Date;
  handleMonthYearChange: (month: number, year: number) => void;
  handleNextMonth: () => void;
  handlePreviousMonth: () => void;
  setBulkUploadDialogOpen: (open: boolean) => void;
  setPaperOtDialogOpen: (open: boolean) => void;
}

export function OtApprovalsHeader({
  currentMonth,
  handleMonthYearChange,
  handleNextMonth,
  handlePreviousMonth,
  setBulkUploadDialogOpen,
  setPaperOtDialogOpen,
}: OtApprovalsHeaderProps) {
  const { t } = useTranslation();

  return (
    <div className="flex flex-col md:flex-row md:items-center justify-between gap-8">
      <div className="space-y-1">
        <Typography className="text-4xl font-bold tracking-tighter text-slate-700 dark:text-slate-200 uppercase leading-none" variant="h1">
          {t("ot_approvals.title")}
        </Typography>
        <Typography className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] opacity-60" variant="p">
          {t("ot_approvals.subtitle")}
        </Typography>
      </div>

      <div className="flex items-center gap-4">
        <Button
          className="h-auto px-6 py-3 rounded-[1.5rem] bg-amber-50 dark:bg-amber-950 text-white text-xs font-bold uppercase tracking-wider shadow-lg hover:bg-amber-600 flex items-center"
          icon={<FileText className="w-4 h-4" />}
          onClick={() => {
            setPaperOtDialogOpen(true);
          }}
          variant="primary"
        >
          {t("approvals.paper_ot.btn_enter") || "Enter Paper Form"}
        </Button>

        <CreateOtRequestDialog />

        <Button
          className="h-auto px-6 py-3 rounded-[1.5rem] bg-blue-50 dark:bg-blue-950 text-white text-xs font-bold uppercase tracking-wider shadow-lg hover:bg-blue-600 flex items-center"
          icon={<Upload className="w-4 h-4" />}
          onClick={() => {
            setBulkUploadDialogOpen(true);
          }}
          variant="primary"
        >
          {t("approvals.bulk_upload.btn") || "Bulk Upload"}
        </Button>

        <div className="flex items-center gap-4 bg-white dark:bg-slate-900 p-2.5 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 shadow-xl shadow-slate-900/5">
          <div className="flex items-center bg-slate-100 dark:bg-slate-700/50 rounded-[1.5rem] p-1.5 shadow-inner border border-slate-200 dark:border-slate-700/50">
            <Button
              className="w-12 h-12 flex items-center justify-center text-slate-500 dark:text-slate-400 hover:text-primary-600 dark:text-primary-400 hover:bg-white dark:hover:bg-slate-900 hover:shadow-md rounded-xl transition-all shadow-none h-auto"
              icon={<ChevronLeft size={24} />}
              onClick={handlePreviousMonth}
              variant="ghost"
            />

            <Popover.Root>
              <Popover.Trigger asChild>
                <Button
                  className="px-10 flex flex-col items-center min-w-[200px] cursor-pointer hover:bg-white dark:hover:bg-slate-900 hover:shadow-md rounded-xl transition-all py-2 group outline-none border border-transparent hover:border-slate-100 dark:border-slate-800 h-auto"
                  variant="ghost"
                >
                  <span className="text-[8px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.3em] leading-none mb-1.5 group-hover:text-primary-500 dark:group-hover:text-primary-400 transition-colors">
                    {t("payroll.table.period").toUpperCase()}
                  </span>
                  <span className="text-sm font-bold text-slate-700 dark:text-slate-200 group-hover:text-primary-600 dark:group-hover:text-primary-400 transition-colors uppercase tracking-tight">
                    {format(currentMonth, "MMMM yyyy")}
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
              className="w-12 h-12 flex items-center justify-center text-slate-500 dark:text-slate-400 hover:text-primary-600 dark:text-primary-400 hover:bg-white dark:hover:bg-slate-900 hover:shadow-md rounded-xl transition-all shadow-none h-auto"
              icon={<ChevronRight size={24} />}
              onClick={handleNextMonth}
              variant="ghost"
            />
          </div>
        </div>
      </div>
    </div>
  );
}

