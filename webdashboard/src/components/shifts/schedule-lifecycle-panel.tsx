import { useMutation, useQueryClient } from "@tanstack/react-query";
import { AlertTriangle, CheckCircle2, FileSearch, History, RefreshCw, Send, ShieldCheck } from "lucide-react";
import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import type {
  EmployeeSchedule,
  ScheduleEvidenceResponse,
  ScheduleHistoryResponse,
  SchedulePreviewResponse,
  ScheduleReplanPreviewResponse,
} from "@/lib/api";

import Button from "@/components/ui/button";
import { Combobox } from "@/components/ui/combobox";
import { Skeleton } from "@/components/ui/skeleton";
import StatusBadge from "@/components/ui/status-badge";
import { Typography } from "@/components/ui/typography";
import { useOrganization } from "@/hooks/use-organization";
import {
  commitScheduleReplan,
  fetchScheduleEvidence,
  fetchScheduleHistory,
  previewScheduleReplan,
  previewSchedules,
  publishSchedules,
} from "@/lib/api";
import { createQueryKey } from "@/lib/query-keys";


interface ScheduleLifecyclePanelProps {
  employeeSchedule?: EmployeeSchedule;
  error?: Error | null;
  isError?: boolean;
  isLoading?: boolean;
  weekDays: Date[];
}

function formatDate(date: Date): string {
  const year = date.getUTCFullYear();
  const month = String(date.getUTCMonth() + 1).padStart(2, "0");
  const day = String(date.getUTCDate()).padStart(2, "0");
  return `${year}-${month}-${day}`;
}

function lifecycleStatus(status: string | null | undefined): "success" | "warning" | "danger" | "info" | "neutral" {
  switch (status) {
    case "published": return "success";
    case "superseded": return "warning";
    case "missing": return "danger";
    case "closed": return "info";
    case "explicit_off": return "neutral";
    default: return "warning";
  }
}

function safeErrorMessage(error: unknown): string {
  return error instanceof Error && error.message.length > 0 ? error.message : "scheduleLifecycle.requestFailed";
}

function JsonPreview({ value }: { value: unknown }) {
  return (
    <pre className="max-h-48 overflow-auto rounded-xl bg-slate-950 p-3 text-left text-[11px] leading-relaxed text-slate-100">
      {JSON.stringify(value, null, 2)}
    </pre>
  );
}

function LifecycleStatusList({ schedule, t }: { schedule: EmployeeSchedule; t: (key: string) => string }) {
  if (schedule.schedules.length === 0) {
    return <Typography className="m-0 text-sm text-slate-500 dark:text-slate-400">{t("scheduleLifecycle.empty")}</Typography>;
  }

  return (
    <ul aria-label={t("scheduleLifecycle.statusesLabel")} className="grid gap-2 sm:grid-cols-2 lg:grid-cols-4">
      {schedule.schedules.map((day) => (
        <li className="flex items-center justify-between gap-3 rounded-xl border border-slate-200 bg-slate-50 px-3 py-2 dark:border-slate-700 dark:bg-slate-800/60" key={day.date}>
          <span className="font-mono text-xs text-slate-600 dark:text-slate-300">{day.date}</span>
          <StatusBadge variant={lifecycleStatus(day.schedule_status)}>
            {t(`scheduleLifecycle.states.${day.schedule_status ?? (day.shift_id ? "published" : "missing")}`)}
          </StatusBadge>
        </li>
      ))}
    </ul>
  );
}

export function ScheduleLifecyclePanel({ employeeSchedule, error, isError = false, isLoading = false, weekDays }: ScheduleLifecyclePanelProps) {
  const { t } = useTranslation();
  const scheduleErrorMessage = error?.message ?? "scheduleLifecycle.requestFailed";
  const scheduleState = isLoading ? "loading" : isError ? "error" : employeeSchedule ? "ready" : "missing";
  const { organization } = useOrganization();
  const queryClient = useQueryClient();
  const [reason, setReason] = useState("");
  if (scheduleState === "loading") {
    return <section aria-busy="true" aria-labelledby="schedule-lifecycle-title" className="space-y-4 rounded-[28px] border border-slate-200 bg-white p-5 shadow-sm dark:border-slate-700 dark:bg-slate-900"><Typography id="schedule-lifecycle-title" variant="h2">{t("scheduleLifecycle.title")}</Typography><Skeleton className="h-5 w-1/2" /><Skeleton className="h-16 w-full" /></section>;
  }
  if (scheduleState === "error") {
    return <section aria-labelledby="schedule-lifecycle-title" className="space-y-4 rounded-[28px] border border-rose-200 bg-rose-50 p-5 dark:border-rose-900 dark:bg-rose-950/30"><Typography id="schedule-lifecycle-title" variant="h2">{t("scheduleLifecycle.title")}</Typography><Typography role="alert">{t(scheduleErrorMessage === "scheduleLifecycle.requestFailed" ? scheduleErrorMessage : "scheduleLifecycle.requestFailed")}</Typography></section>;
  }
  if (!employeeSchedule) {
    return <section aria-labelledby="schedule-lifecycle-title" className="space-y-4 rounded-[28px] border border-slate-200 bg-white p-5 shadow-sm dark:border-slate-700 dark:bg-slate-900"><Typography id="schedule-lifecycle-title" variant="h2">{t("scheduleLifecycle.title")}</Typography><Typography>{t("scheduleLifecycle.empty")}</Typography></section>;
  }
  const [selectedDate, setSelectedDate] = useState(employeeSchedule.schedules[0]?.date ?? formatDate(weekDays[0]));
  const [preview, setPreview] = useState<SchedulePreviewResponse | null>(null);
  const [replanPreview, setReplanPreview] = useState<ScheduleReplanPreviewResponse | null>(null);
  const [history, setHistory] = useState<ScheduleHistoryResponse | null>(null);
  const [evidence, setEvidence] = useState<ScheduleEvidenceResponse | null>(null);

  const dateRange = useMemo(() => ({ start: formatDate(weekDays[0]), end: formatDate(weekDays[weekDays.length - 1]) }), [weekDays]);
  const scheduleByDate = new Map(employeeSchedule.schedules.map((day) => [day.date, day]));
  const expectedRevisions = Object.fromEntries(
    weekDays.map((day) => {
      const date = formatDate(day);
      return [`${employeeSchedule.employment_id}:${date}`, scheduleByDate.get(date)?.revision_number ?? 0];
    }),
  );
  const dateOptions = weekDays.map((day) => {
    const date = formatDate(day);
    return { label: date, value: date };
  });
  const selectedSchedule = scheduleByDate.get(selectedDate);
  const selectedRevision = evidence?.revisions.at(-1);
  const selectedSegment = evidence?.segments.at(-1);
  const replacement = selectedSegment?.selected_shift_version_id
    ? {
        status: "work" as const,
        source: "manual",
        shift_version_id: selectedSegment.selected_shift_version_id,
        intervals: selectedSegment.scheduled_start_at && selectedSegment.scheduled_end_at
          ? [{ sort_order: 0, start: selectedSegment.scheduled_start_at, end: selectedSegment.scheduled_end_at, breaks: [] }]
          : [],
      }
    : null;

  const previewMutation = useMutation({
    mutationFn: () => previewSchedules({
      employment_ids: [employeeSchedule.employment_id],
      start_date: dateRange.start,
      end_date: dateRange.end,
      timezone: organization?.timezone ?? "UTC",
      mode: "office",
      approval_instance_id: null,
      reason: reason.trim(),
      retroactive: false,
      expected_revisions: expectedRevisions,
    }),
    onSuccess: ({ data }) => { setPreview(data); },
  });
  const publishMutation = useMutation({
    mutationFn: () => {
      if (!preview) throw new Error("scheduleLifecycle.previewRequired");
      return publishSchedules({
        employment_ids: [employeeSchedule.employment_id],
        start_date: dateRange.start,
        end_date: dateRange.end,
        timezone: organization?.timezone ?? "UTC",
        approval_instance_id: null,
        preview_id: preview.preview_id,
        preview_hash: preview.preview_hash,
        reason: reason.trim(),
        retroactive: false,
        expected_revisions: expectedRevisions,
      }, crypto.randomUUID());
    },
    onSuccess: () => {
      setPreview(null);
      void queryClient.invalidateQueries({ queryKey: createQueryKey("schedules") });
    },
  });
  const historyMutation = useMutation({
    mutationFn: () => fetchScheduleHistory({ employment_id: employeeSchedule.employment_id, start_date: dateRange.start, end_date: dateRange.end }),
    onSuccess: ({ data }) => { setHistory(data); },
  });
  const evidenceMutation = useMutation({
    mutationFn: () => fetchScheduleEvidence(employeeSchedule.employment_id, selectedDate),
    onSuccess: ({ data }) => { setEvidence(data); },
  });
  const replanPreviewMutation = useMutation({
    mutationFn: () => {
      if (!selectedRevision || !replacement) throw new Error("scheduleLifecycle.evidenceRequired");
      return previewScheduleReplan(employeeSchedule.employment_id, selectedDate, {
        expected_revision: selectedRevision.revision_number,
        reason: reason.trim(),
        replacement,
      });
    },
    onSuccess: ({ data }) => { setReplanPreview(data); },
  });
  const replanMutation = useMutation({
    mutationFn: () => {
      if (!replanPreview || !selectedRevision || !replacement) throw new Error("scheduleLifecycle.replanPreviewRequired");
      return commitScheduleReplan(employeeSchedule.employment_id, selectedDate, {
        expected_revision: selectedRevision.revision_number,
        reason: reason.trim(),
        replacement,
        preview_id: replanPreview.preview_id,
        preview_hash: replanPreview.preview_hash,
        confirm_diff_hash: replanPreview.diff_hash,
      }, crypto.randomUUID());
    },
    onSuccess: () => {
      setReplanPreview(null);
      void queryClient.invalidateQueries({ queryKey: createQueryKey("schedules") });
    },
  });
  const activeError = [previewMutation, publishMutation, historyMutation, evidenceMutation, replanPreviewMutation, replanMutation].find((mutation) => mutation.isError)?.error;
  const isBusy = [previewMutation, publishMutation, historyMutation, evidenceMutation, replanPreviewMutation, replanMutation].some((mutation) => mutation.isPending);

  return (
    <section aria-labelledby="schedule-lifecycle-title" className="space-y-5 rounded-[28px] border border-slate-200 bg-white p-5 shadow-sm dark:border-slate-700 dark:bg-slate-900">
      <div className="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
        <div>
          <div className="flex items-center gap-2">
            <ShieldCheck className="text-primary-600" size={18} />
            <Typography className="m-0 text-base font-bold text-slate-800 dark:text-slate-100" id="schedule-lifecycle-title" variant="h2">{t("scheduleLifecycle.title")}</Typography>
          </div>
          <Typography className="m-0 mt-1 text-xs text-slate-500 dark:text-slate-400">{t("scheduleLifecycle.subtitle")}</Typography>
        </div>
        <div className="min-w-48">
          <span className="mb-1 block text-xs font-semibold text-slate-600 dark:text-slate-300">{t("scheduleLifecycle.dateLabel")}</span>
          <Combobox
            onChange={(value) => { setSelectedDate(value); setEvidence(null); }}
            options={dateOptions}
            placeholder={t("scheduleLifecycle.dateLabel")}
            searchPlaceholder={t("common.combobox.search")}
            value={selectedDate}
          />
        </div>
      </div>

      <LifecycleStatusList schedule={employeeSchedule} t={t} />

      <div className="grid gap-3 lg:grid-cols-[1fr_auto]">
        <label className="text-xs font-semibold text-slate-600 dark:text-slate-300">
          {t("scheduleLifecycle.reasonLabel")}
          <textarea aria-describedby="schedule-lifecycle-reason-hint" className="mt-1 min-h-20 w-full resize-y rounded-xl border border-slate-300 bg-white p-3 text-sm font-normal outline-none focus-visible:ring-2 focus-visible:ring-primary-500 dark:border-slate-600 dark:bg-slate-800" maxLength={2000} onChange={(event) => { setReason(event.target.value); }} placeholder={t("scheduleLifecycle.reasonPlaceholder")} value={reason} />
          <span className="mt-1 block font-normal text-slate-500" id="schedule-lifecycle-reason-hint">{t("scheduleLifecycle.reasonHint")}</span>
        </label>
        <div className="flex flex-wrap items-end gap-2">
          <Button icon={<RefreshCw size={15} />} loading={previewMutation.isPending} onClick={() => { previewMutation.mutate(); }} variant="outline">{t("scheduleLifecycle.preview")}</Button>
          <Button disabled={!preview} icon={<Send size={15} />} loading={publishMutation.isPending} onClick={() => { publishMutation.mutate(); }}>{t("scheduleLifecycle.publish")}</Button>
        </div>
      </div>

      {preview ? <div className="space-y-2 rounded-xl border border-blue-200 bg-blue-50 p-3 dark:border-blue-900 dark:bg-blue-950/30"><div className="flex items-center gap-2 text-xs font-semibold text-blue-800 dark:text-blue-200"><CheckCircle2 size={15} />{t("scheduleLifecycle.provisionalNotice")} <StatusBadge variant="info">{t("scheduleLifecycle.states.provisional")}</StatusBadge></div><JsonPreview value={preview.data} /></div> : null}

      <div className="flex flex-wrap gap-2 border-t border-slate-100 pt-4 dark:border-slate-800">
        <Button icon={<History size={15} />} loading={historyMutation.isPending} onClick={() => { historyMutation.mutate(); }} variant="outline">{t("scheduleLifecycle.history")}</Button>
        <Button icon={<FileSearch size={15} />} loading={evidenceMutation.isPending} onClick={() => { evidenceMutation.mutate(); }} variant="outline">{t("scheduleLifecycle.evidence")}</Button>
        <Button disabled={!replacement || !selectedSchedule?.revision_number} icon={<RefreshCw size={15} />} loading={replanPreviewMutation.isPending} onClick={() => { replanPreviewMutation.mutate(); }} variant="outline">{t("scheduleLifecycle.replan")}</Button>
      </div>

      {history ? <div aria-live="polite" className="space-y-3"><Typography className="m-0 text-sm font-bold text-slate-700 dark:text-slate-200" variant="h3">{t("scheduleLifecycle.history")}</Typography>{history.data.length === 0 ? <Typography className="m-0 text-sm text-slate-500">{t("scheduleLifecycle.noHistory")}</Typography> : history.data.map((entry) => <div className="flex flex-wrap items-center justify-between gap-2 rounded-xl border border-slate-200 p-3 dark:border-slate-700" key={entry.revision_id}><span className="font-mono text-xs">{entry.business_date} · r{entry.revision_number}</span><StatusBadge variant={lifecycleStatus(entry.state)}>{t(`scheduleLifecycle.states.${entry.state}`)}</StatusBadge><span className="text-xs text-slate-500">{entry.reason || t("scheduleLifecycle.noReason")}</span></div>)}</div> : null}
      {evidence ? <div aria-live="polite" className="space-y-3"><Typography className="m-0 text-sm font-bold text-slate-700 dark:text-slate-200" variant="h3">{t("scheduleLifecycle.evidence")}</Typography><div className="grid gap-2 text-xs text-slate-600 dark:text-slate-300 sm:grid-cols-3"><span>{t("scheduleLifecycle.revisions")}: {evidence.revisions.length}</span><span>{t("scheduleLifecycle.segments")}: {evidence.segments.length}</span><span>{t("scheduleLifecycle.closures")}: {evidence.closure_days.length}</span></div><JsonPreview value={{ diff_chain: evidence.diff_chain, audit_actions: evidence.audit_actions, events: evidence.events }} /></div> : null}
      {replanPreview ? <div className="space-y-3 rounded-xl border border-amber-200 bg-amber-50 p-3 dark:border-amber-900 dark:bg-amber-950/30"><div className="flex items-center gap-2 text-xs font-semibold text-amber-800 dark:text-amber-200"><AlertTriangle size={15} />{t("scheduleLifecycle.diffNotice")}</div><JsonPreview value={replanPreview.diff} /><Button disabled={reason.trim().length === 0} loading={replanMutation.isPending} onClick={() => { replanMutation.mutate(); }}>{t("scheduleLifecycle.confirmReplan")}</Button></div> : null}
      {activeError ? <Typography className="m-0 rounded-xl border border-rose-200 bg-rose-50 p-3 text-sm text-rose-700 dark:border-rose-900 dark:bg-rose-950/30 dark:text-rose-200" role="alert">{t(safeErrorMessage(activeError))}</Typography> : null}
      {isBusy && !preview && !history && !evidence ? <div aria-label={t("common.loading")} className="space-y-2"><Skeleton className="h-4 w-1/2" /><Skeleton className="h-16 w-full" /></div> : null}
    </section>
  );
}
