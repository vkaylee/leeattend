import { Calculator, CheckCircle2, Clock, History, Plus } from "lucide-react";
import React from "react";

import { TFunction } from "i18next";

import Button from "@/components/ui/button";
import EmptyState from "@/components/ui/empty-state";
import { Typography } from "@/components/ui/typography";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { AttendancePolicyVersion } from "@/lib/api";
import { cn } from "@/lib/utils";
import { maybeNull } from "@/types/api-overrides";

import { roundingLabel, workUnitLabel } from "./utils";

interface AttendanceVersionTimelineProps {
  onNewVersion?: () => void;
  t: TFunction;
  versions: AttendancePolicyVersion[];
}

export function AttendanceVersionTimeline({ onNewVersion, t, versions }: AttendanceVersionTimelineProps) {
  return (
    <div className="bg-white dark:bg-slate-900 p-6 sm:p-10 rounded-2xl sm:rounded-[2.5rem] border border-slate-100 dark:border-slate-800 shadow-sm space-y-8">
      <TimelineHeader count={versions.length} t={t} />

      {versions.length === 0 ? (
        <EmptyTimeline onNewVersion={onNewVersion} t={t} />
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
        <Typography className="text-lg font-bold text-slate-800 dark:text-slate-100 tracking-tight" variant="h2">
          {t("common.workspace_settings.attendance_config")}
        </Typography>
      </div>
      <span className="text-xs font-semibold text-slate-500 dark:text-slate-400">
        {count} {count === 1 ? t("common.version", "phiên bản") : t("common.versions", "phiên bản")}
      </span>
    </div>
  );
}

function EmptyTimeline({ onNewVersion, t }: { onNewVersion?: () => void; t: TFunction }) {
  return (
    <EmptyState
      action={
        onNewVersion ? (
          <Button
            className="rounded-xl px-4 py-2 text-xs font-semibold shadow-xs normal-case tracking-normal"
            icon={<Plus size={16} />}
            onClick={onNewVersion}
            variant="primary"
          >
            {t("common.workspace_settings.new_version", { defaultValue: "Tạo phiên bản đầu tiên" })}
          </Button>
        ) : undefined
      }
      description={t("common.workspace_settings.no_versions_desc") || "Nếu không tạo phiên bản cụ thể, cấu hình chấm công mặc định sẽ được áp dụng."}
      icon={<Calculator size={40} />}
      title={t("common.workspace_settings.no_versions") || "Chưa có phiên bản cấu hình"}
    />
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
