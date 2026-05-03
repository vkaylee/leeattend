import { Calculator, CheckCircle2, Clock, History, ShieldAlert } from "lucide-react";
import React from "react";

import { TFunction } from "i18next";

import { AttendancePolicyVersion } from "@/lib/api";
import { cn } from "@/lib/utils";

import { formatDate, roundingLabel, workUnitLabel } from "./utils";

interface AttendanceVersionTimelineProps {
  t: TFunction;
  versions: AttendancePolicyVersion[];
}

export function AttendanceVersionTimeline({ t, versions }: AttendanceVersionTimelineProps) {
  return (
    <div className="bg-white dark:bg-slate-900 p-10 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 shadow-sm space-y-8">
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
        <h2 className="text-xl font-bold text-slate-800 dark:text-slate-100 tracking-tight uppercase">
          {t("common.workspace_settings.attendance_config")}
        </h2>
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
      <h3 className="text-lg font-bold text-slate-700 dark:text-slate-200 uppercase tracking-tight">
        {t("common.workspace_settings.no_versions") || "No configuration yet"}
      </h3>
      <p className="text-slate-500 dark:text-slate-400 text-xs font-bold uppercase tracking-widest mt-1 opacity-50 italic">
        {t("common.workspace_settings.no_versions_desc") || "Create the first version to get started"}
      </p>
    </div>
  );
}

interface TimelineItemProps {
  isCurrent: boolean;
  t: TFunction;
  version: AttendancePolicyVersion;
}

function TimelineItem({ isCurrent, t, version }: TimelineItemProps) {
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
            <p className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">
              {formatDate(version.valid_from)}
              {version.valid_to ? ` → ${formatDate(version.valid_to)}` : " → Present"}
            </p>
          </div>
        </div>
      </div>

      <TimelineItemGrid isCurrent={isCurrent} t={t} version={version} />
      <p className="mt-3 text-[9px] font-medium text-slate-400 dark:text-slate-500 italic">
        Created {formatDate(version.created_at)}
      </p>
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
        <p className="text-[9px] font-bold text-slate-400 uppercase tracking-widest mb-1">
          {t("common.workspace_settings.work_unit")}
        </p>
        <p
          className={cn(
            "text-lg font-bold tracking-tight",
            isCurrent ? "text-primary-600 dark:text-primary-400" : "text-slate-600 dark:text-slate-300"
          )}
        >
          {workUnitLabel(version.work_unit_minutes, t)}
        </p>
      </div>
      <div className="p-4 bg-white dark:bg-slate-900 rounded-xl border border-slate-100 dark:border-slate-800">
        <p className="text-[9px] font-bold text-slate-400 uppercase tracking-widest mb-1">
          {t("common.workspace_settings.rounding_policy")}
        </p>
        <p
          className={cn(
            "text-lg font-bold tracking-tight",
            isCurrent ? "text-primary-600 dark:text-primary-400" : "text-slate-600 dark:text-slate-300"
          )}
        >
          {roundingLabel(version.rounding_policy, t)}
        </p>
      </div>
      <div className="col-span-2 p-4 bg-white dark:bg-slate-900 rounded-xl border border-slate-100 dark:border-slate-800 flex items-center justify-between">
        <div>
          <p className="text-[9px] font-bold text-slate-400 uppercase tracking-widest mb-1">
            {t("common.workspace_settings.early_arrival_strategy")}
          </p>
          <p
            className={cn(
              "text-sm font-bold tracking-tight",
              isCurrent ? "text-amber-600 dark:text-amber-400" : "text-slate-600 dark:text-slate-300"
            )}
          >
            {t(`common.workspace_settings.early_arrival_options.${version.early_arrival_strategy}`)}
          </p>
        </div>
        <ShieldAlert
          className={cn("w-5 h-5", isCurrent ? "text-amber-500/50" : "text-slate-400/50")}
        />
      </div>
    </div>
  );
}
