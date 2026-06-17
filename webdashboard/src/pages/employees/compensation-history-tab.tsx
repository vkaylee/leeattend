import { useQuery } from "@tanstack/react-query";
import React from "react";
import { useTranslation } from "react-i18next";

import CompensationTimeline from "@/components/compensation/compensation-timeline";
import { Typography } from "@/components/ui/typography";
import { fetchCompensationHistory } from "@/lib/api/employees";

interface CompensationHistoryTabProps {
  employeeId: string;
  employmentId?: string;
  formatDate: (date: string) => string;
  wsId: string;
}

export default function CompensationHistoryTab({
  employeeId,
  employmentId,
  wsId,
  formatDate,
}: CompensationHistoryTabProps) {
  const { t } = useTranslation();

  const { data, isLoading, error } = useQuery({
    enabled: !!wsId && !!employeeId,
    queryFn: () => fetchCompensationHistory(wsId, employeeId, employmentId),
    queryKey: ["employee-compensation-history", wsId, employeeId, employmentId],
  });

  if (isLoading) {
    return (
      <div className="bg-slate-50 dark:bg-slate-900 border border-slate-100 dark:border-slate-800 p-8 rounded-[2.5rem] shadow-sm animate-pulse h-96" />
    );
  }

  if (error) {
    return (
      <div className="p-8 text-center bg-rose-50 dark:bg-rose-950/20 text-rose-600 dark:text-rose-400 rounded-3xl border border-rose-100 dark:border-rose-900/30">
        <Typography
          className="text-sm font-bold uppercase tracking-wider"
          variant="p"
        >
          {t("common.error", "Đã xảy ra lỗi")}
        </Typography>
        <Typography className="text-xs mt-1" variant="p">
          {error instanceof Error ? error.message : t("employees.compensation.fetch_error", "Không thể tải lịch sử lương.")}
        </Typography>
      </div>
    );
  }

  const timeline = data?.timeline ?? [];

  return (
    <div className="space-y-6 animate-in fade-in duration-500">
      <div className="flex flex-col md:flex-row justify-between items-start md:items-center gap-4">
        <div>
          <Typography
            className="text-xl font-bold text-slate-800 dark:text-slate-200 tracking-tight"
            variant="h3"
          >
            {t("employees.compensation.timeline_title", "Lịch sử Lương & Đãi ngộ")}
          </Typography>
          <Typography
            className="text-xs text-slate-500 dark:text-slate-400 mt-1"
            variant="p"
          >
            {t("employees.compensation.timeline_subtitle", "Theo dõi sự thay đổi về lương cơ bản và các khoản phụ cấp")}
          </Typography>
        </div>
      </div>

      <CompensationTimeline events={timeline} formatDate={formatDate} />
    </div>
  );
}
