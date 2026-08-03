import { Calculator, CheckCircle2, Clock, History } from "lucide-react";
import React from "react";

import { TFunction } from "i18next";

import { Typography } from "@/components/ui/typography";
import { maybeNull } from "@/types/api-overrides";
import { AttendancePolicyVersion } from "@/lib/api";
import { cn } from "@/lib/utils";
import { useDateTimeFormatter } from "@/hooks/use-datetime";

import { roundingLabel, workUnitLabel } from "./utils";

interface AttendanceVersionTimelineProps {
  t: TFunction;
  versions: AttendancePolicyVersion[];
}

export function AttendanceVersionTimeline({ t, versions }: AttendanceVersionTimelineProps) {
  return (
    <div className="bg-white dark:bg-slate-900 p-6 sm:p-10 rounded-2xl sm:rounded-[2.5rem] border border-slate-100 dark:border-slate-800 shadow-sm space-y-8">
      <TimelineHeader count={versions.length} t={t} />

      {versions.length === 0 ? (
        <EmptyTimeline t={t} />
      ) : (
        <div className="space-y-4">
          {versions.map((v, index) => (
            <TimelineItem
              isCurrent={index === 0 && !v.valid_to}
              key={v.id}
              t={t}
              version={v}
            />
          ))}
        </div>
      )}
    </div>
  );
}

function TimelineHeader({ count, t }: { count: number; t: TFunction }) {
  return (
    <div className="flex items-center justify-between border-b border-slate-50 dark:border-slate-800 pb-6">
      <div className="flex items-center gap-3">
        <History className="w-6 h-6 text-primary-500" />
        <Typography className="text-xl font-bold text-slate-800 dark:text-slate-100 tracking-tight uppercase" variant="h2">
          {t("common.workspace_settings.attendance_config")}
        </Typography>
      </div>
      <span className="text-[10px] font-bold text-slate-400 uppercase tracking-widest">
        {count} {count === 1 ? "VERSION" : "VERSIONS"}
      </span>
    </div>
  );
}

function EmptyTimeline({ t }: { t: TFunction }) {
  return (
    <div className="py-16 text-center border-2 border-dashed border-slate-100 dark:border-slate-800 rounded-[2rem] bg-slate-50 dark:bg-slate-800/30 flex flex-col items-center">
      <div className="w-20 h-20 bg-white dark:bg-slate-900 rounded-full flex items-center justify-center shadow-sm mb-5 border border-slate-100 dark:border-slate-800">
        <Calculator className="text-slate-200 dark:text-slate-700" size={36} />
      </div>
      <Typography className="text-lg font-bold text-slate-700 dark:text-slate-200 uppercase tracking-tight" variant="h3">
        {t("common.workspace_settings.no_versions") || "No configuration yet"}
      </Typography>
      <Typography className="text-slate-500 dark:text-slate-400 text-xs font-bold uppercase tracking-widest mt-1 opacity-50 italic" variant="p">
        {t("common.workspace_settings.no_versions_desc") || "Create the first version to get started"}
      </Typography>
    </div>
  );
}

interface TimelineItemProps {
  isCurrent: boolean;
  t: TFunction;
  version: AttendancePolicyVersion;
}

function TimelineItem({ isCurrent, t, version }: TimelineItemProps) {
  const { formatDate } = useDateTimeFormatter();
  return (
    <div
      className={cn(
        "relative p-6 rounded-2xl border transition-all group",
        isCurrent
          ? "bg-primary-50/50 dark:bg-primary-950/30 border-primary-200 dark:border-primary-800 shadow-md"
          : "bg-slate-50/50 dark:bg-slate-800/30 border-slate-100 dark:border-slate-800 opacity-70"
      )}
    >
      <div className="flex items-start justify-between gap-4">
        <div className="flex items-center gap-4">
          <div
            className={cn(
              "w-12 h-12 rounded-xl flex items-center justify-center flex-shrink-0",
              isCurrent
                ? "bg-primary-600 text-white shadow-lg"
                : "bg-white dark:bg-slate-900 text-slate-400 dark:text-slate-600 border border-slate-100 dark:border-slate-800"
            )}
          >
            {isCurrent ? <CheckCircle2 size={24} /> : <Clock size={24} />}
          </div>
          <div>
            <div className="flex items-center gap-3 mb-1">
              {isCurrent ? (
                <span className="px-2.5 py-0.5 bg-primary-600 text-white text-[9px] font-bold uppercase tracking-widest rounded-lg shadow-sm">
                  {t("status_active_caps")}
                </span>
              ) : null}
              {version.valid_to ? (
                <span className="px-2.5 py-0.5 bg-slate-200 dark:bg-slate-700 text-slate-600 dark:text-slate-300 text-[9px] font-bold uppercase tracking-widest rounded-lg">
                  {t("status_expired")}
                </span>
              ) : null}
            </div>
            <Typography className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest" variant="p">
              {formatDate(version.valid_from)}
              {version.valid_to ? ` → ${formatDate(version.valid_to)}` : " → Present"}
            </Typography>
          </div>
        </div>
      </div>

      <TimelineItemGrid isCurrent={isCurrent} t={t} version={version} />
      <Typography className="mt-3 text-[9px] font-medium text-slate-400 dark:text-slate-500 italic" variant="p">
        Created {formatDate(version.created_at)}
      </Typography>
    </div>
  );
}

function TimelineItemGrid({
  isCurrent,
  t,
  version,
}: {
  isCurrent: boolean;
  t: TFunction;
  version: AttendancePolicyVersion;
}) {
  return (
    <div className="mt-4 grid grid-cols-2 gap-4">
      <div className="p-4 bg-white dark:bg-slate-900 rounded-xl border border-slate-100 dark:border-slate-800">
        <Typography className="text-[9px] font-bold text-slate-400 uppercase tracking-widest mb-1" variant="p">
          {t("common.workspace_settings.work_unit")}
        </Typography>
        <Typography className={cn(
            "text-lg font-bold tracking-tight",
            isCurrent ? "text-primary-600 dark:text-primary-400" : "text-slate-600 dark:text-slate-300"
          )}
          variant="p"
        >
          {workUnitLabel(version.work_unit_minutes, t)}
        </Typography>
      </div>
      <div className="p-4 bg-white dark:bg-slate-900 rounded-xl border border-slate-100 dark:border-slate-800">
        <Typography className="text-[9px] font-bold text-slate-400 uppercase tracking-widest mb-1" variant="p">
          {t("common.workspace_settings.rounding_policy")}
        </Typography>
        <Typography className={cn(
            "text-lg font-bold tracking-tight",
            isCurrent ? "text-primary-600 dark:text-primary-400" : "text-slate-600 dark:text-slate-300"
          )}
          variant="p"
        >
          {roundingLabel(version.rounding_policy, t)}
        </Typography>
      </div>
      <div className="p-4 bg-white dark:bg-slate-900 rounded-xl border border-slate-100 dark:border-slate-800">
        <Typography className="text-[9px] font-bold text-slate-400 uppercase tracking-widest mb-1" variant="p">
          {t("common.workspace_settings.early_arrival_strategy")}
        </Typography>
        <Typography className={cn(
            "text-sm font-bold tracking-tight",
            isCurrent ? "text-amber-600 dark:text-amber-400" : "text-slate-600 dark:text-slate-300"
          )}
          variant="p"
        >
          {t(`common.workspace_settings.early_arrival_options.${version.early_arrival_strategy}`)}
        </Typography>
      </div>
      <div className="p-4 bg-white dark:bg-slate-900 rounded-xl border border-slate-100 dark:border-slate-800">
        <Typography className="text-[9px] font-bold text-slate-400 uppercase tracking-widest mb-1" variant="p">
          {t("common.workspace_settings.grace_period_minutes") || "Late Grace Period"}
        </Typography>
        <Typography className={cn(
            "text-lg font-bold tracking-tight",
            isCurrent ? "text-primary-600 dark:text-primary-400" : "text-slate-600 dark:text-slate-300"
          )}
          variant="p"
        >
          {maybeNull(version.grace_period_minutes) != null ? `${maybeNull(version.grace_period_minutes)}m` : "15m"}
        </Typography>
      </div>
    </div>
  );
}
