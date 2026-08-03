import type { TFunction } from "i18next";

import {
  Briefcase,
  Building2,
  Calendar,
  ChevronDown,
  ChevronUp,
  DollarSign,
  GitCompare,
  Sparkles,
  TrendingUp
} from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import type { CompensationTimelineEvent } from "@/lib/api/types";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import { useCurrency } from "@/hooks/use-currency";

import CompensationDiffBadge from "./compensation-diff-badge";

interface CompensationTimelineProps {
  events: CompensationTimelineEvent[];
  formatDate: (date: string) => string;
}

const CHANGE_TYPE_ICONS: Record<string, React.ReactNode> = {
  initial: <Sparkles className="w-5 h-5 text-indigo-500" />,
  promotion: <TrendingUp className="w-5 h-5 text-emerald-500" />,
  adjustment: <DollarSign className="w-5 h-5 text-sky-500" />,
  transfer: <GitCompare className="w-5 h-5 text-amber-500" />,
};

function getChangeTypeIcon(type: string) {
  return CHANGE_TYPE_ICONS[type] ?? <Calendar className="w-5 h-5 text-slate-500" />;
}

function getChangeTypeLabel(type: string, t: TFunction) {
  const labels: Record<string, string> = {
    initial: t("employees.compensation.types.initial", "Lương khởi điểm"),
    promotion: t("employees.compensation.types.promotion", "Thăng chức"),
    adjustment: t("employees.compensation.types.adjustment", "Điều chỉnh lương"),
    transfer: t("employees.compensation.types.transfer", "Điều chuyển bộ phận"),
  };
  return labels[type] ?? type;
}

function CompensationComponentsBreakdown({
  event,
  isExpanded,
  onToggle,
  t,
}: {
  event: CompensationTimelineEvent;
  isExpanded: boolean;
  onToggle: () => void;
  t: TFunction;
}) {
  const { formatCurrency: fmt } = useCurrency();
  return (
    <div className="mt-5 pt-4 border-t border-slate-100 dark:border-slate-800/80">
      <Button
        className="flex items-center gap-1.5 text-[10px] font-bold text-slate-400 dark:text-slate-500 hover:text-slate-600 dark:hover:text-slate-300 uppercase tracking-widest focus:outline-none p-0 h-auto"
        onClick={onToggle}
        variant="ghost"
      >
        <span>
          {t("employees.compensation.components", "Chi tiết thành phần lương")}
        </span>
        {isExpanded ? <ChevronUp size={12} /> : <ChevronDown size={12} />}
      </Button>

      {isExpanded ? (
        <div className="mt-4 space-y-2 animate-in slide-in-from-top-1 duration-200">
          <div className="grid grid-cols-2 gap-2 text-[10px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-wider px-2">
            <span>{t("employees.compensation.components", "Thành phần")}</span>
            <span className="text-right">{t("employees.compensation.amount", "Số tiền")}</span>
          </div>
          <div className="space-y-1.5">
            {event.components.map((comp) => (
              <div
                className="grid grid-cols-2 gap-2 text-xs py-2 px-3 bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-xl"
                key={comp.component.id}
              >
                <span className="font-bold text-slate-700 dark:text-slate-300">
                  {comp.definition.name}{" "}
                  <span className="text-[10px] text-slate-400 font-mono">
                    ({comp.definition.code})
                  </span>
                </span>
                <span className="text-right font-mono text-slate-600 dark:text-slate-400">
                  {fmt(comp.component.amount)}
                </span>
              </div>
            ))}
          </div>
        </div>
      ) : null}
    </div>
  );
}

interface EventCardHeaderProps {
  event: CompensationTimelineEvent;
  formatDate: (date: string) => string;
  getChangeTypeLabel: (type: string) => string;
  t: TFunction;
}

function EventCardHeader({ event, formatDate, getChangeTypeLabel, t }: EventCardHeaderProps) {
  const { formatCurrency: fmt } = useCurrency();
  return (
    <>
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4 mb-6">
        <div>
          <div className="flex items-center gap-3 mb-2 flex-wrap">
            <span className="text-xs font-bold font-mono tracking-widest text-slate-400 dark:text-slate-500 uppercase flex items-center gap-1.5 bg-white dark:bg-slate-800 px-3 py-1 rounded-full border border-slate-100 dark:border-slate-700">
              <Calendar size={12} />
              {formatDate(event.effective_date)}
            </span>
            <span className="bg-primary-50 dark:bg-primary-950/40 text-primary-600 dark:text-primary-400 text-[10px] font-bold uppercase tracking-wider px-3 py-1 rounded-full border border-primary-100/50 dark:border-primary-900/30">
              {getChangeTypeLabel(event.change_type)}
            </span>
          </div>
          <Typography
            className="text-2xl font-bold text-slate-800 dark:text-slate-200 tracking-tight"
            variant="h4"
          >
            {fmt(event.total_compensation)}
          </Typography>
        </div>

        <div className="flex items-center gap-3">
          <CompensationDiffBadge
            delta={event.change_details.delta}
          />
        </div>
      </div>

      <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 pt-4 border-t border-slate-100 dark:border-slate-800/80 text-xs text-slate-600 dark:text-slate-400">
        <div className="flex items-center gap-2">
          <Briefcase className="text-slate-400" size={14} />
          <span>
            <strong className="text-slate-700 dark:text-slate-300">
              {event.employment.job_title ?? t("common.no_title", "Không có chức danh")}
            </strong>
            {" @ "}
            <span className="text-slate-500 dark:text-slate-400">
              {event.employment.employer_name}
            </span>
          </span>
        </div>
        {event.employment.department_name ? (
          <div className="flex items-center gap-2">
            <Building2 className="text-slate-400" size={14} />
            <span>
              {t("employees.details.current_dept")}:{" "}
              <strong className="text-slate-700 dark:text-slate-300">
                {event.employment.department_name}
              </strong>
            </span>
          </div>
        ) : null}
      </div>
    </>
  );
}

export default function CompensationTimeline({
  events,
  formatDate,
}: CompensationTimelineProps) {
  const { t } = useTranslation();
  const [expandedEvents, setExpandedEvents] = useState<Record<number, boolean>>({});

  const getChangeTypeLabelMemo = (type: string) => getChangeTypeLabel(type, t);

  const toggleExpand = (index: number) => {
    setExpandedEvents((prev) => ({
      ...prev,
      [index]: !prev[index],
    }));
  };

  if (events.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center p-12 text-center bg-slate-50 dark:bg-slate-800/30 rounded-[2.5rem] border border-dashed border-slate-200 dark:border-slate-700">
        <DollarSign className="w-16 h-16 text-slate-300 dark:text-slate-600 mb-4 animate-bounce" />
        <Typography
          className="text-lg font-bold text-slate-700 dark:text-slate-300 uppercase tracking-wider mb-2"
          variant="h3"
        >
          {t("employees.compensation.no_history", "Chưa có lịch sử lương")}
        </Typography>
        <Typography
          className="text-xs text-slate-500 dark:text-slate-400 max-w-sm"
          variant="p"
        >
          {t(
            "employees.compensation.no_history_desc",
            "Dữ liệu lịch sử lương và các thành phần phụ cấp của nhân viên sẽ xuất hiện tại đây khi được cập nhật."
          )}
        </Typography>
      </div>
    );
  }

  return (
    <div className="relative border-l border-slate-200 dark:border-slate-800 ml-6 pl-8 space-y-10 py-4">
      {events.map((event, idx) => {
        const isExpanded = expandedEvents[idx] ?? false;
        const hasComponents = event.components.length > 0;

        return (
          <div
            className="relative group"
            key={`${event.effective_date}-${idx}`}
          >
            <div className="absolute -left-[45px] top-1 bg-white dark:bg-slate-900 border-2 border-slate-200 dark:border-slate-700 p-2 rounded-full shadow-md z-10 group-hover:scale-110 transition-transform duration-300">
              {getChangeTypeIcon(event.change_type)}
            </div>

            <div className="bg-slate-50 dark:bg-slate-800/40 border border-slate-100 dark:border-slate-800/80 p-8 rounded-[2.5rem] hover:bg-white dark:hover:bg-slate-900/60 hover:shadow-xl hover:shadow-primary-100/10 dark:hover:shadow-none transition-all duration-300">
              <EventCardHeader
                event={event}
                formatDate={formatDate}
                getChangeTypeLabel={getChangeTypeLabelMemo}
                t={t}
              />

              {hasComponents ? (
                <CompensationComponentsBreakdown
                  event={event}
                  isExpanded={isExpanded}
                  onToggle={() => { toggleExpand(idx); }}
                  t={t}
                />
              ) : null}
            </div>
          </div>
        );
      })}
    </div>
  );
}
