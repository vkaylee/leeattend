import { useQuery } from "@tanstack/react-query";
import { UserX } from "lucide-react";
import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import { TFunction } from "i18next";

import { getScheduleLifecycleCellState } from "@/components/shifts/roster/roster-utils";
import {
  AdjustmentData,
  AdjustmentRequestModal,
} from "@/components/time-tracking/adjustment-request-modal";
import { PunchCard } from "@/components/time-tracking/punch-card";
import { PageContainer } from "@/components/ui/page-container";
import { PageHeader } from "@/components/ui/page-header";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import { useMyClock, useMyProfile } from "@/hooks/use-ess";
import { fetchOrganization, fetchSchedules, submitAttendanceAdjustment } from "@/lib/api";
import { toUTCISO } from "@/lib/datetime";
import { createQueryKey } from "@/lib/query-keys";

import { HistoryTable } from "./components/history-table";
import { MyClockSkeleton } from "./components/my-clock-skeleton";
import { PunchGeofenceStatus } from "./components/punch-geofence-status";
import { Shift24hTimeline, type ShiftTimeWindow } from "./components/shift-24h-timeline";
import { TodayHoursSummary } from "./components/today-hours-summary";

function formatDateInTimezone(date: Date, timeZone: string): string {
  try {
    return new Intl.DateTimeFormat("en-CA", {
      day: "2-digit",
      month: "2-digit",
      timeZone,
      year: "numeric",
    }).format(date);
  } catch {
    return date.toISOString().split("T")[0];
  }
}

function extractIntervalBreakHours(intervals?: unknown[]): { breakEndHour?: number; breakStartHour?: number } {
  if (!Array.isArray(intervals)) return {};
  for (const raw of intervals) {
    const breaks = (raw as { breaks?: { end?: string; start?: string }[] } | null)?.breaks;
    if (!Array.isArray(breaks)) continue;
    for (const br of breaks) {
      const startHour = parseClockHours(br.start);
      const endHour = parseClockHours(br.end);
      if (startHour !== undefined && endHour !== undefined && endHour > startHour) {
        return { breakEndHour: endHour, breakStartHour: startHour };
      }
    }
  }
  return {};
}

function parseClockHours(clock?: string | null): number | undefined {
  if (!clock) return undefined;
  const parts = clock.split(":");
  if (parts.length < 2) return undefined;
  const hours = Number(parts[0]);
  const minutes = Number(parts[1]);
  if (Number.isNaN(hours) || Number.isNaN(minutes)) return undefined;
  return hours + minutes / 60;
}

export default function MyClockPage() {
  const { t } = useTranslation();
  const toast = useToast();


  const { employee, isLoading: isLoadingProfile } = useMyProfile();
  const {
    activeEntry,
    entries,
    isLoading: isLoadingClock,
    isPunching,
    submitAction,
  } = useMyClock({ enabled: !isLoadingProfile && Boolean(employee) });

  const { data: organization } = useQuery({
    queryKey: createQueryKey("organization"),
    queryFn: fetchOrganization,
  });
  const [isAdjustmentModalOpen, setIsAdjustmentModalOpen] = useState(false);
  const [nowMs] = useState(() => Date.now());

  const orgTimezone = organization?.timezone ?? "UTC";
  const todayStr = useMemo(
    () => formatDateInTimezone(new Date(), orgTimezone),
    [orgTimezone],
  );
  const activeEmploymentId = employee?.active_employment_id;

  const { data: todayScheduleRows } = useQuery({
    queryKey: createQueryKey("schedules", todayStr, activeEmploymentId, "ess-today"),
    queryFn: () =>
      activeEmploymentId
        ? fetchSchedules(todayStr, todayStr, [activeEmploymentId], "all", "official")
        : Promise.resolve([]),
    enabled: Boolean(activeEmploymentId),
  });

  const { todayScheduleState, todayScheduledShift } = useMemo(() => {
    const rows = Array.isArray(todayScheduleRows) ? todayScheduleRows : [];
    const empRow = rows.find((s) => s.employment_id === activeEmploymentId) ?? rows[0];
    const daySchedules = empRow?.schedules.filter((s) => s.date === todayStr) ?? [];
    const state = getScheduleLifecycleCellState(daySchedules);
    const primary = daySchedules.find((s) => s.shift_id && s.start_time && s.end_time);
    if (!primary || state === "missing" || state === "explicit_off") {
      return { todayScheduleState: state, todayScheduledShift: null as ShiftTimeWindow | null };
    }
    const startHour = parseClockHours(primary.start_time);
    const endHour = parseClockHours(primary.end_time);
    if (startHour === undefined || endHour === undefined) {
      return { todayScheduleState: state, todayScheduledShift: null as ShiftTimeWindow | null };
    }
    const intervalBreak = extractIntervalBreakHours((primary as { intervals?: unknown[] }).intervals);
    return {
      todayScheduleState: state,
      todayScheduledShift: {
        startHour,
        endHour,
        breakStartHour: parseClockHours(primary.break_start) ?? intervalBreak.breakStartHour,
        breakEndHour: parseClockHours(primary.break_end) ?? intervalBreak.breakEndHour,
        label: primary.shift_name ?? t("scheduleLifecycle.states.published"),
      } satisfies ShiftTimeWindow,
    };
  }, [activeEmploymentId, t, todayScheduleRows, todayStr]);

  const todayPunches = useMemo(() => {
    return entries
      .filter((e) => formatDateInTimezone(new Date(e.punch_in_time), orgTimezone) === todayStr)
      .map((e) => ({
        punchInTime: e.punch_in_time,
        punchOutTime: e.punch_out_time,
      }));
  }, [entries, orgTimezone, todayStr]);

  const todayActualMinutes = useMemo(() => {
    let total = 0;
    for (const p of todayPunches) {
      const inMs = new Date(p.punchInTime).getTime();
      const outMs = p.punchOutTime ? new Date(p.punchOutTime).getTime() : nowMs;
      if (outMs > inMs) {
        total += Math.round((outMs - inMs) / 60000);
      }
    }
    return total;
  }, [nowMs, todayPunches]);
  const handlePunch = async () => {
    try {
      await submitAction(activeEntry ? "work_out" : "work_in");
      toast.success(t("ess.clock.punch_success", "Chấm công thành công"));
    } catch (e: unknown) {
      const message = e instanceof Error ? e.message : "Unknown error";
      toast.error(message);
    }
  };


  const handleAdjustmentSubmit = async (data: AdjustmentData) => {
    if (!employee) {
      toast.error(t("common.error"));
      return;
    }

    try {
      const tz = organization?.timezone ?? "UTC";
      const punchTime = toUTCISO(data.date, data.punchTime, tz);

      await submitAttendanceAdjustment({
        employment_id: employee.active_employment_id ?? "",
        reason: data.reason,
        requested_punch_time: punchTime,
        time_entry_id: data.timeEntryId,
      });
      toast.success(t("ess.clock.adjustment_requested"));
    } catch (e: unknown) {
      const message = e instanceof Error ? e.message : "Unknown error";
      toast.error(message);
    }
  };

  if (isLoadingProfile || isLoadingClock) return <MyClockSkeleton />;

  if (!employee) return <EmptyState t={t} />;

  return (
    <PageContainer>
      <PageHeader data-testid="ess-clock-title" subtitle={t("ess.clock.subtitle")} title={t("ess.clock.title")} />

      <PunchGeofenceStatus />

      <div className="grid grid-cols-1 lg:grid-cols-12 gap-6">
        <div className="lg:col-span-8">
          <Shift24hTimeline
            actualPunches={todayPunches}
            scheduledShift={todayScheduledShift}
            scheduleStatus={todayScheduleState}
          />
        </div>
        <div className="lg:col-span-4">
          <TodayHoursSummary actualMinutes={todayActualMinutes} />
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-12 gap-8">
        <div className="lg:col-span-5">
          <PunchCard
            activeEntry={activeEntry}
            isPunching={isPunching}
            onPunch={handlePunch}
            onRequestAdjustment={() => {
              setIsAdjustmentModalOpen(true);
            }}
            userName={employee.full_name}
          />
        </div>

        <HistoryTable entries={entries} />
      </div>

      <AdjustmentRequestModal
        isOpen={isAdjustmentModalOpen}
        onClose={() => {
          setIsAdjustmentModalOpen(false);
        }}
        onSubmit={handleAdjustmentSubmit}
        timezone={organization?.timezone ?? "UTC"}
      />
    </PageContainer>
  );
}

function EmptyState({ t }: { t: TFunction }) {
  return (
    <div className="py-24 text-center border-2 border-dashed border-slate-200 dark:border-slate-800 rounded-2xl sm:rounded-[2.5rem] bg-slate-50 dark:bg-slate-800/30 flex flex-col items-center max-w-lg mx-auto mt-10">
      <div className="w-20 h-20 bg-white dark:bg-slate-900 rounded-full flex items-center justify-center shadow-sm dark:shadow-black/10 mb-6 border border-slate-100 dark:border-slate-800 text-slate-350 dark:text-slate-650">
        <UserX size={32} />
      </div>
      <Typography className="text-[10px] font-bold text-slate-300 dark:text-slate-600 uppercase tracking-[0.25em] mb-2" variant="p">
        {t("common.no_profile")}
      </Typography>
      <Typography className="text-xs text-slate-400 dark:text-slate-500 max-w-xs leading-relaxed px-6" variant="p">
        Please contact your administrator to set up your employee profile.
      </Typography>
    </div>
  );
}


