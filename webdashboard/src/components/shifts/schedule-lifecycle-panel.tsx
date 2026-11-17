import { useMutation, useQueryClient } from "@tanstack/react-query";
import { AlertTriangle, CheckCircle2, FileSearch, History, RefreshCw, Send, ShieldCheck } from "lucide-react";
import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import type {
  EmployeeSchedule,
  ScheduleEvidenceResponse,
  ScheduleEvidenceSegment,
  ScheduleHistoryResponse,
  SchedulePreviewResponse,
  ScheduleReplanPreviewResponse,
  ScheduleReplanReplacement,
} from "@/lib/api";

import Button from "@/components/ui/button";
import { Combobox } from "@/components/ui/combobox";
import { Skeleton } from "@/components/ui/skeleton";
import StatusBadge from "@/components/ui/status-badge";
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
  employeeSchedule: EmployeeSchedule;
  weekDays: Date[];
}

type TranslationFunction = (key: string) => string;

interface DateRange {
  end: string;
  start: string;
}

interface DateOption {
  label: string;
  value: string;
}

interface ScheduleLifecycleMutationOptions {
  dateRange: DateRange;
  employeeSchedule: EmployeeSchedule;
  expectedRevisions: Record<string, number>;
  organizationTimezone: string;
  reason: string;
  selectedDate: string;
}

function formatDate(date: Date): string {
  const year = date.getUTCFullYear();
  const month = String(date.getUTCMonth() + 1).padStart(2, "0");
  const day = String(date.getUTCDate()).padStart(2, "0");
  return `${year}-${month}-${day}`;
}

function getDateRange(weekDays: Date[]): DateRange {
  return {
    end: formatDate(weekDays[weekDays.length - 1]),
    start: formatDate(weekDays[0]),
  };
}

function buildExpectedRevisions(
  employeeSchedule: EmployeeSchedule,
  scheduleByDate: Map<string, EmployeeSchedule["schedules"][number]>,
  weekDays: Date[],
): Record<string, number> {
  return Object.fromEntries(
    weekDays.map((day) => {
      const date = formatDate(day);
      return [`${employeeSchedule.employment_id}:${date}`, scheduleByDate.get(date)?.revision_number ?? 0];
    }),
  );
}

function buildReplacement(segment: ScheduleEvidenceSegment | undefined): ScheduleReplanReplacement | null {
  if (!segment?.selected_shift_version_id) return null;

  return {
    intervals: segment.scheduled_start_at && segment.scheduled_end_at
      ? [{ breaks: [], end: segment.scheduled_end_at, sort_order: 0, start: segment.scheduled_start_at }]
      : [],
    shift_version_id: segment.selected_shift_version_id,
    source: "manual",
    status: "work",
  };
}

function useScheduleLifecycleMutations({
  dateRange,
  employeeSchedule,
  expectedRevisions,
  organizationTimezone,
  reason,
  selectedDate,
}: ScheduleLifecycleMutationOptions) {
  const queryClient = useQueryClient();
  const [preview, setPreview] = useState<SchedulePreviewResponse | null>(null);
  const [replanPreview, setReplanPreview] = useState<ScheduleReplanPreviewResponse | null>(null);
  const [history, setHistory] = useState<ScheduleHistoryResponse | null>(null);
  const [evidence, setEvidence] = useState<ScheduleEvidenceResponse | null>(null);

  const previewMutation = useMutation({
    mutationFn: () => previewSchedules({
      approval_instance_id: null,
      employment_ids: [employeeSchedule.employment_id],
      end_date: dateRange.end,
      expected_revisions: expectedRevisions,
      mode: "office",
      reason: reason.trim(),
      retroactive: false,
      start_date: dateRange.start,
      timezone: organizationTimezone,
    }),
    onSuccess: ({ data }) => { setPreview(data); },
  });
  const publishMutation = useMutation({
    mutationFn: () => {
      if (!preview) throw new Error("scheduleLifecycle.previewRequired");
      return publishSchedules({
        approval_instance_id: null,
        employment_ids: [employeeSchedule.employment_id],
        end_date: dateRange.end,
        expected_revisions: expectedRevisions,
        preview_hash: preview.preview_hash,
        preview_id: preview.preview_id,
        reason: reason.trim(),
        retroactive: false,
        start_date: dateRange.start,
        timezone: organizationTimezone,
      }, crypto.randomUUID());
    },
    onSuccess: () => {
      setPreview(null);
      void queryClient.invalidateQueries({ queryKey: createQueryKey("schedules") });
    },
  });
  const historyMutation = useMutation({
    mutationFn: () => fetchScheduleHistory({ employment_id: employeeSchedule.employment_id, end_date: dateRange.end, start_date: dateRange.start }),
    onSuccess: ({ data }) => { setHistory(data); },
  });
  const evidenceMutation = useMutation({
    mutationFn: () => fetchScheduleEvidence(employeeSchedule.employment_id, selectedDate),
    onSuccess: ({ data }) => { setEvidence(data); },
  });
  const selectedRevision = evidence?.revisions.at(-1);
  const replacement = buildReplacement(evidence?.segments.at(-1));
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
        confirm_diff_hash: replanPreview.diff_hash,
        expected_revision: selectedRevision.revision_number,
        preview_hash: replanPreview.preview_hash,
        preview_id: replanPreview.preview_id,
        reason: reason.trim(),
        replacement,
      }, crypto.randomUUID());
    },
    onSuccess: () => {
      setReplanPreview(null);
      void queryClient.invalidateQueries({ queryKey: createQueryKey("schedules") });
    },
  });
  const mutations = [previewMutation, publishMutation, historyMutation, evidenceMutation, replanPreviewMutation, replanMutation];

  return {
    clearEvidence: () => { setEvidence(null); },
    evidence,
    evidenceMutation,
    history,
    historyMutation,
    isBusy: mutations.some((mutation) => mutation.isPending),
    activeError: mutations.find((mutation) => mutation.isError)?.error,
    preview,
    previewMutation,
    replanMutation,
    replacement,
    replanPreview,
    replanPreviewMutation,
    publishMutation,
  };
}

function lifecycleStatus(status: string | null | undefined): "success" | "warning" | "danger" | "info" | "neutral" {
  const statuses: Record<string, "success" | "warning" | "danger" | "info" | "neutral"> = {
    closed: "info",
    explicit_off: "neutral",
    missing: "danger",
    published: "success",
    superseded: "warning",
  };
  return status ? statuses[status] ?? "warning" : "warning";
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

function LifecycleStatusList({ schedule, t }: { schedule: EmployeeSchedule; t: TranslationFunction }) {
  if (schedule.schedules.length === 0) {
    return <p className="text-sm text-slate-500 dark:text-slate-400">{t("scheduleLifecycle.empty")}</p>;
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

function LifecycleHeader({ dateOptions, onDateChange, selectedDate, t }: {
  dateOptions: DateOption[];
  onDateChange: (value: string) => void;
  selectedDate: string;
  t: TranslationFunction;
}) {
  return (
    <div className="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
      <div>
        <div className="flex items-center gap-2">
          <ShieldCheck className="text-primary-600" size={18} />
          <h2 className="text-base font-bold text-slate-800 dark:text-slate-100" id="schedule-lifecycle-title">{t("scheduleLifecycle.title")}</h2>
        </div>
        <p className="mt-1 text-xs text-slate-500 dark:text-slate-400">{t("scheduleLifecycle.subtitle")}</p>
      </div>
      <div className="min-w-48">
        <span className="mb-1 block text-xs font-semibold text-slate-600 dark:text-slate-300">{t("scheduleLifecycle.dateLabel")}</span>
        <Combobox
          onChange={onDateChange}
          options={dateOptions}
          placeholder={t("scheduleLifecycle.dateLabel")}
          searchPlaceholder={t("common.combobox.search")}
          value={selectedDate}
        />
      </div>
    </div>
  );
}

function ScheduleEditor({
  hasPreview,
  isPreviewPending,
  isPublishPending,
  onPreview,
  onPublish,
  onReasonChange,
  reason,
  t,
}: {
  hasPreview: boolean;
  isPreviewPending: boolean;
  isPublishPending: boolean;
  onPreview: () => void;
  onPublish: () => void;
  onReasonChange: (value: string) => void;
  reason: string;
  t: TranslationFunction;
}) {
  return (
    <div className="grid gap-3 lg:grid-cols-[1fr_auto]">
      <label className="text-xs font-semibold text-slate-600 dark:text-slate-300">
        {t("scheduleLifecycle.reasonLabel")}
        <textarea aria-describedby="schedule-lifecycle-reason-hint" className="mt-1 min-h-20 w-full resize-y rounded-xl border border-slate-300 bg-white p-3 text-sm font-normal outline-none focus-visible:ring-2 focus-visible:ring-primary-500 dark:border-slate-600 dark:bg-slate-800" maxLength={2000} onChange={(event) => { onReasonChange(event.target.value); }} placeholder={t("scheduleLifecycle.reasonPlaceholder")} value={reason} />
        <span className="mt-1 block font-normal text-slate-500" id="schedule-lifecycle-reason-hint">{t("scheduleLifecycle.reasonHint")}</span>
      </label>
      <div className="flex flex-wrap items-end gap-2">
        <Button icon={<RefreshCw size={15} />} loading={isPreviewPending} onClick={onPreview} variant="outline">{t("scheduleLifecycle.preview")}</Button>
        <Button disabled={!hasPreview} icon={<Send size={15} />} loading={isPublishPending} onClick={onPublish}>{t("scheduleLifecycle.publish")}</Button>
      </div>
    </div>
  );
}

function PreviewNotice({ preview, t }: { preview: SchedulePreviewResponse | null; t: TranslationFunction }) {
  if (!preview) return null;

  return (
    <div className="space-y-2 rounded-xl border border-blue-200 bg-blue-50 p-3 dark:border-blue-900 dark:bg-blue-950/30">
      <div className="flex items-center gap-2 text-xs font-semibold text-blue-800 dark:text-blue-200">
        <CheckCircle2 size={15} />
        {t("scheduleLifecycle.provisionalNotice")}
        <StatusBadge variant="info">{t("scheduleLifecycle.states.provisional")}</StatusBadge>
      </div>
      <JsonPreview value={preview.data} />
    </div>
  );
}

function LifecycleDataActions({
  evidencePending,
  historyPending,
  onEvidence,
  onHistory,
  onReplan,
  replanDisabled,
  replanPending,
  t,
}: {
  evidencePending: boolean;
  historyPending: boolean;
  onEvidence: () => void;
  onHistory: () => void;
  onReplan: () => void;
  replanDisabled: boolean;
  replanPending: boolean;
  t: TranslationFunction;
}) {
  return (
    <div className="flex flex-wrap gap-2 border-t border-slate-100 pt-4 dark:border-slate-800">
      <Button icon={<History size={15} />} loading={historyPending} onClick={onHistory} variant="outline">{t("scheduleLifecycle.history")}</Button>
      <Button icon={<FileSearch size={15} />} loading={evidencePending} onClick={onEvidence} variant="outline">{t("scheduleLifecycle.evidence")}</Button>
      <Button disabled={replanDisabled} icon={<RefreshCw size={15} />} loading={replanPending} onClick={onReplan} variant="outline">{t("scheduleLifecycle.replan")}</Button>
    </div>
  );
}

function HistoryPanel({ history, t }: { history: ScheduleHistoryResponse | null; t: TranslationFunction }) {
  if (!history) return null;

  return (
    <div aria-live="polite" className="space-y-3">
      <h3 className="text-sm font-bold text-slate-700 dark:text-slate-200">{t("scheduleLifecycle.history")}</h3>
      {history.data.length === 0 ? <p className="text-sm text-slate-500">{t("scheduleLifecycle.noHistory")}</p> : history.data.map((entry) => (
        <div className="flex flex-wrap items-center justify-between gap-2 rounded-xl border border-slate-200 p-3 dark:border-slate-700" key={entry.revision_id}>
          <span className="font-mono text-xs">{entry.business_date} · r{entry.revision_number}</span>
          <StatusBadge variant={lifecycleStatus(entry.state)}>{t(`scheduleLifecycle.states.${entry.state}`)}</StatusBadge>
          <span className="text-xs text-slate-500">{entry.reason || t("scheduleLifecycle.noReason")}</span>
        </div>
      ))}
    </div>
  );
}

function EvidencePanel({ evidence, t }: { evidence: ScheduleEvidenceResponse | null; t: TranslationFunction }) {
  if (!evidence) return null;

  return (
    <div aria-live="polite" className="space-y-3">
      <h3 className="text-sm font-bold text-slate-700 dark:text-slate-200">{t("scheduleLifecycle.evidence")}</h3>
      <div className="grid gap-2 text-xs text-slate-600 dark:text-slate-300 sm:grid-cols-3">
        <span>{t("scheduleLifecycle.revisions")}: {evidence.revisions.length}</span>
        <span>{t("scheduleLifecycle.segments")}: {evidence.segments.length}</span>
        <span>{t("scheduleLifecycle.closures")}: {evidence.closure_days.length}</span>
      </div>
      <JsonPreview value={{ audit_actions: evidence.audit_actions, diff_chain: evidence.diff_chain, events: evidence.events }} />
    </div>
  );
}

function ReplanPanel({
  isPending,
  onConfirm,
  preview,
  reason,
  t,
}: {
  isPending: boolean;
  onConfirm: () => void;
  preview: ScheduleReplanPreviewResponse | null;
  reason: string;
  t: TranslationFunction;
}) {
  if (!preview) return null;

  return (
    <div className="space-y-3 rounded-xl border border-amber-200 bg-amber-50 p-3 dark:border-amber-900 dark:bg-amber-950/30">
      <div className="flex items-center gap-2 text-xs font-semibold text-amber-800 dark:text-amber-200"><AlertTriangle size={15} />{t("scheduleLifecycle.diffNotice")}</div>
      <JsonPreview value={preview.diff} />
      <Button disabled={reason.trim().length === 0} loading={isPending} onClick={onConfirm}>{t("scheduleLifecycle.confirmReplan")}</Button>
    </div>
  );
}

function LifecycleLoading({ isBusy, hasContent, t }: { hasContent: boolean; isBusy: boolean; t: TranslationFunction }) {
  if (!isBusy || hasContent) return null;

  return <div aria-label={t("common.loading")} className="space-y-2"><Skeleton className="h-4 w-1/2" /><Skeleton className="h-16 w-full" /></div>;
}

function LifecycleError({ error, t }: { error: unknown; t: TranslationFunction }) {
  if (!error) return null;

  return <p className="rounded-xl border border-rose-200 bg-rose-50 p-3 text-sm text-rose-700 dark:border-rose-900 dark:bg-rose-950/30 dark:text-rose-200" role="alert">{t(safeErrorMessage(error))}</p>;
}

export function ScheduleLifecyclePanel({ employeeSchedule, weekDays }: ScheduleLifecyclePanelProps) {
  const { t } = useTranslation();
  const { organization } = useOrganization();
  const [reason, setReason] = useState("");
  const [selectedDate, setSelectedDate] = useState(employeeSchedule.schedules[0]?.date ?? formatDate(weekDays[0]));
  const dateRange = useMemo(() => getDateRange(weekDays), [weekDays]);
  const scheduleByDate = useMemo(() => new Map(employeeSchedule.schedules.map((day) => [day.date, day])), [employeeSchedule.schedules]);
  const expectedRevisions = useMemo(
    () => buildExpectedRevisions(employeeSchedule, scheduleByDate, weekDays),
    [employeeSchedule, scheduleByDate, weekDays],
  );
  const dateOptions = useMemo(() => weekDays.map((day) => {
    const date = formatDate(day);
    return { label: date, value: date };
  }), [weekDays]);
  const selectedSchedule = scheduleByDate.get(selectedDate);
  const lifecycle = useScheduleLifecycleMutations({
    dateRange,
    employeeSchedule,
    expectedRevisions,
    organizationTimezone: organization?.timezone ?? "UTC",
    reason,
    selectedDate,
  });

  return (
    <section aria-labelledby="schedule-lifecycle-title" className="space-y-5 rounded-[28px] border border-slate-200 bg-white p-5 shadow-sm dark:border-slate-700 dark:bg-slate-900">
      <LifecycleHeader
        dateOptions={dateOptions}
        onDateChange={(value) => { setSelectedDate(value); lifecycle.clearEvidence(); }}
        selectedDate={selectedDate}
        t={t}
      />
      <LifecycleStatusList schedule={employeeSchedule} t={t} />
      <ScheduleEditor
        hasPreview={lifecycle.preview !== null}
        isPreviewPending={lifecycle.previewMutation.isPending}
        isPublishPending={lifecycle.publishMutation.isPending}
        onPreview={() => { lifecycle.previewMutation.mutate(); }}
        onPublish={() => { lifecycle.publishMutation.mutate(); }}
        onReasonChange={setReason}
        reason={reason}
        t={t}
      />
      <PreviewNotice preview={lifecycle.preview} t={t} />
      <LifecycleDataActions
        evidencePending={lifecycle.evidenceMutation.isPending}
        historyPending={lifecycle.historyMutation.isPending}
        onEvidence={() => { lifecycle.evidenceMutation.mutate(); }}
        onHistory={() => { lifecycle.historyMutation.mutate(); }}
        onReplan={() => { lifecycle.replanPreviewMutation.mutate(); }}
        replanDisabled={!lifecycle.replacement || !selectedSchedule?.revision_number}
        replanPending={lifecycle.replanPreviewMutation.isPending}
        t={t}
      />
      <HistoryPanel history={lifecycle.history} t={t} />
      <EvidencePanel evidence={lifecycle.evidence} t={t} />
      <ReplanPanel isPending={lifecycle.replanMutation.isPending} onConfirm={() => { lifecycle.replanMutation.mutate(); }} preview={lifecycle.replanPreview} reason={reason} t={t} />
      <LifecycleError error={lifecycle.activeError} t={t} />
      <LifecycleLoading hasContent={Boolean(lifecycle.preview ?? lifecycle.history ?? lifecycle.evidence)} isBusy={lifecycle.isBusy} t={t} />
    </section>
  );
}
