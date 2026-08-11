import { Calendar } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import { addMonths, endOfMonth, format, startOfMonth, subMonths } from "date-fns";

import { ClaimOtDialog } from "@/components/ess/claim-ot-dialog";
import { PageContainer } from "@/components/ui/page-container";
import { Typography } from "@/components/ui/typography";
import { useMyAttendance, useMyAttendanceSummaries } from "@/hooks/use-ess";
import { AttendanceDailySummary } from "@/lib/api/types";

import { AttendanceHeader } from "./components/attendance-header";
import { AttendanceTable } from "./components/attendance-table";

export default function MyAttendancePage() {
  const [currentMonth, setCurrentMonth] = useState(new Date());

  const startDate = format(startOfMonth(currentMonth), "yyyy-MM-dd");
  const endDate = format(endOfMonth(currentMonth), "yyyy-MM-dd");

  const { entries, isLoading } = useMyAttendance({
    end_date: endDate,
    start_date: startDate,
  });

  const { summaries } = useMyAttendanceSummaries(startDate, endDate);

  const [selectedSummary, setSelectedSummary] = useState<AttendanceDailySummary | null>(null);
  const [claimOpen, setClaimOpen] = useState(false);

  const getSummary = (date: string) => summaries.find((s) => s.date === date);

  const handleMonthYearChange = (month: number, year: number) => {
    setCurrentMonth(new Date(year, month - 1, 1));
  };
  const handlePreviousMonth = () => {
    setCurrentMonth(subMonths(currentMonth, 1));
  };
  const handleNextMonth = () => {
    setCurrentMonth(addMonths(currentMonth, 1));
  };

  return (
    <PageContainer>
      <AttendanceHeader
        currentMonth={currentMonth}
        handleMonthYearChange={handleMonthYearChange}
        handleNextMonth={handleNextMonth}
        handlePreviousMonth={handlePreviousMonth}
      />

      <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-3xl sm:rounded-[3rem] overflow-hidden shadow-sm animate-in slide-in-from-bottom-8 duration-700">
        {isLoading ? (
          <LoadingState />
        ) : entries.length === 0 ? (
          <EmptyState />
        ) : (
          <AttendanceTable
            entries={entries}
            getSummary={getSummary}
            setClaimOpen={setClaimOpen}
            setSelectedSummary={(s) => {
              setSelectedSummary(s);
            }}
          />
        )}
      </div>

      <ClaimOtDialog onOpenChange={setClaimOpen} open={claimOpen} summary={selectedSummary} />
    </PageContainer>
  );
}

function LoadingState() {
  const { t } = useTranslation();
  return (
    <div className="p-32 flex flex-col items-center gap-6 animate-pulse">
      <div className="w-12 h-12 border-4 border-primary-600 border-t-transparent rounded-full animate-spin"></div>
      <span className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.3em]">
        {t("common.loading")}
      </span>
    </div>
  );
}

function EmptyState() {
  const { t } = useTranslation();
  return (
    <div className="py-40 text-center flex flex-col items-center border-4 border-dashed border-slate-50 dark:border-slate-800 mx-2 sm:m-8 rounded-3xl sm:rounded-[3rem] bg-slate-50 dark:bg-slate-800/30">
      <div className="w-24 h-24 bg-white dark:bg-slate-900 rounded-full flex items-center justify-center shadow-xl border border-slate-100 dark:border-slate-800 mb-8">
        <Calendar className="text-slate-200 dark:text-slate-700" size={40} />
      </div>
      <Typography className="text-xl font-bold text-slate-700 dark:text-slate-200 uppercase tracking-tight" variant="h3">
        {t("ess.attendance.no_data")}
      </Typography>
      <Typography className="text-slate-500 dark:text-slate-400 text-[10px] font-bold uppercase tracking-widest mt-2 opacity-50 italic" variant="p">
        {t("common.no_records_found")}
      </Typography>
    </div>
  );
}
