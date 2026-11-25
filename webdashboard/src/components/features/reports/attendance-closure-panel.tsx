import { AlertCircle, CheckCircle2, LockKeyhole, UnlockKeyhole } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import type { AttendanceClosureRequest, AttendancePeriodStatus } from "@/lib/api/attendance-closure";

import Button from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { Typography } from "@/components/ui/typography";
import { useAttendanceClosure } from "@/hooks/use-attendance-closure";
import { useMyPermissions } from "@/hooks/use-permissions";
import { ApiError } from "@/lib/api/errors";

interface AttendanceClosurePanelProps {
  employerId: string | null;
  month: number;
  year: number;
}

type ClosureAction = "close" | "reopen";
type ClosureMutation = (request: AttendanceClosureRequest) => Promise<unknown>;

const ERROR_KEYS: Record<string, string> = {
  "error.closure_conflict": "closure.errors.closure_conflict",
  "error.locked_period": "closure.errors.locked_period",
  "error.missing_coverage": "closure.errors.missing_coverage",
  "error.open_punch": "closure.errors.open_punch",
  "error.self_reopen_forbidden": "closure.errors.self_reopen_forbidden",
  "error.stale_period": "closure.errors.stale_period",
  VALIDATION_ERROR: "closure.errors.validation",
};

function getErrorKey(error: unknown): string {
  if (error instanceof ApiError) {
    const messageKey = typeof error.errorBody?.message === "string" ? ERROR_KEYS[error.errorBody.message] : undefined;
    if (messageKey) return messageKey;
    if (error.errorCode && ERROR_KEYS[error.errorCode]) return ERROR_KEYS[error.errorCode];
  }
  return "closure.errors.generic";
}

function getReasonError(reason: string): string | null {
  return reason.trim() ? null : "closure.errors.reason_required";
}

function isClosureQueryEnabled(canRead: boolean, isLoadingPermissions: boolean): boolean {
  return canRead && !isLoadingPermissions;
}

function shouldHideClosurePanel(employerId: string | null, isLoadingPermissions: boolean, canRead: boolean): boolean {
  return !employerId || (!isLoadingPermissions && !canRead);
}

function getClosureActions(
  canClosePermission: boolean,
  canReopenPermission: boolean,
  isClosed: boolean,
  payrollLocked: boolean,
): Pick<ClosureActionFormProps, "canClose" | "canReopen"> {
  return {
    canClose: canClosePermission && !isClosed && !payrollLocked,
    canReopen: canReopenPermission && isClosed && !payrollLocked,
  };
}

interface ClosureActionExecution {
  action: ClosureAction;
  close: ClosureMutation;
  reason: string;
  reopen: ClosureMutation;
  version: number;
}

async function executeClosureAction({ action, close, reason, reopen, version }: ClosureActionExecution): Promise<string | null> {
  try {
    const mutation = action === "close" ? close : reopen;
    await mutation({ reason: reason.trim(), version });
    return null;
  } catch (mutationError) {
    return getErrorKey(mutationError);
  }
}

function ClosureLoadError({ onRetry }: { onRetry: () => void }) {
  const { t } = useTranslation();
  return (
    <section className="rounded-3xl border border-rose-200 bg-rose-50 p-5 text-rose-900 dark:border-rose-900/60 dark:bg-rose-950/30 dark:text-rose-100" role="alert">
      <div className="flex items-start gap-3">
        <AlertCircle aria-hidden="true" className="mt-0.5 shrink-0" size={20} />
        <div className="space-y-2">
          <Typography className="font-semibold" variant="p">{t("closure.errors.load")}</Typography>
          <Button onClick={onRetry} size="sm" variant="outline">{t("closure.retry")}</Button>
        </div>
      </div>
    </section>
  );
}

function ClosureEmptyState() {
  const { t } = useTranslation();
  return (
    <section className="rounded-3xl border border-slate-200 bg-white p-5 text-slate-700 dark:border-slate-800 dark:bg-slate-900 dark:text-slate-200" data-testid="closure-empty">
      <Typography className="font-semibold" variant="p">{t("closure.empty.title")}</Typography>
      <Typography className="mt-1 text-sm text-slate-500 dark:text-slate-400" variant="p">{t("closure.empty.description")}</Typography>
    </section>
  );
}

interface ClosureHeaderProps {
  isClosed: boolean;
  payrollLocked: boolean;
}

function ClosureHeader({ isClosed, payrollLocked }: ClosureHeaderProps) {
  const { t } = useTranslation();
  return (
    <div className="flex flex-col gap-4 border-b border-slate-100 p-5 sm:flex-row sm:items-center sm:justify-between dark:border-slate-800">
      <div className="flex items-start gap-3">
        {isClosed ? <LockKeyhole aria-hidden="true" className="mt-0.5 text-emerald-600" size={21} /> : <UnlockKeyhole aria-hidden="true" className="mt-0.5 text-slate-500" size={21} />}
        <div>
          <Typography className="font-semibold text-slate-900 dark:text-slate-100" variant="h2">{t("closure.title")}</Typography>
          <Typography className="mt-1 text-sm text-slate-500 dark:text-slate-400" variant="p">{t(isClosed ? "closure.status.closed" : "closure.status.open")}</Typography>
        </div>
      </div>
      <ClosureStatusBadge isClosed={isClosed} />
      {payrollLocked ? <Typography className="text-sm text-amber-700 dark:text-amber-300" role="status" variant="p">{t("closure.errors.locked_period")}</Typography> : null}
    </div>
  );
}

function ClosureStatusBadge({ isClosed }: { isClosed: boolean }) {
  const { t } = useTranslation();
  const className = isClosed
    ? "inline-flex items-center gap-2 rounded-full bg-emerald-50 px-3 py-1.5 text-sm font-semibold text-emerald-700 dark:bg-emerald-950/40 dark:text-emerald-300"
    : "inline-flex items-center gap-2 rounded-full bg-slate-100 px-3 py-1.5 text-sm font-semibold text-slate-700 dark:bg-slate-800 dark:text-slate-200";
  return <div className={className}><CheckCircle2 aria-hidden="true" size={16} />{t(isClosed ? "closure.badge.closed" : "closure.badge.open")}</div>;
}

function ClosureEvidence({ data }: { data: AttendancePeriodStatus }) {
  const { t } = useTranslation();
  return (
    <div className="grid gap-3 border-b border-slate-100 p-5 text-sm dark:border-slate-800 sm:grid-cols-3" data-testid="closure-evidence">
      <span>{t("closure.evidence.number")}: {data.closure_number ?? "—"}</span>
      <span>{t("closure.evidence.days")}: {data.day_count ?? "—"}</span>
      <span className="truncate" title={data.snapshot_hash ?? undefined}>{t("closure.evidence.hash")}: {data.snapshot_hash ?? "—"}</span>
    </div>
  );
}

interface ClosureActionFormProps {
  actionError: string | null;
  canClose: boolean;
  canReopen: boolean;
  error: unknown;
  isClosing: boolean;
  isMutating: boolean;
  isReopening: boolean;
  onAction: (action: ClosureAction) => void;
  reason: string;
  setReason: (reason: string) => void;
}

function ClosureActionForm({ actionError, canClose, canReopen, error, isClosing, isMutating, isReopening, onAction, reason, setReason }: ClosureActionFormProps) {
  const { t } = useTranslation();
  return (
    <div className="space-y-4 p-5">
      <label className="block text-sm font-medium text-slate-700 dark:text-slate-200" htmlFor="closure-reason">{t("closure.reason")}</label>
      <textarea aria-describedby="closure-reason-hint" className="min-h-20 w-full rounded-2xl border border-slate-200 bg-slate-50 px-4 py-3 text-sm outline-none transition focus-visible:ring-2 focus-visible:ring-primary-500 dark:border-slate-700 dark:bg-slate-950" id="closure-reason" maxLength={2000} onChange={(event) => { setReason(event.target.value); }} placeholder={t("closure.reason_placeholder")} value={reason} />
      <Typography className="text-xs text-slate-500 dark:text-slate-400" id="closure-reason-hint" variant="p">{t("closure.reason_hint")}</Typography>
      {actionError || error ? <Typography className="text-sm font-medium text-rose-700 dark:text-rose-300" role="alert" variant="p">{t(actionError ?? getErrorKey(error))}</Typography> : null}
      <div className="flex flex-wrap gap-3">
        {canClose ? <Button disabled={isMutating} loading={isClosing} onClick={() => { onAction("close"); }}>{t("closure.actions.close")}</Button> : null}
        {canReopen ? <Button disabled={isMutating} loading={isReopening} onClick={() => { onAction("reopen"); }} variant="outline">{t("closure.actions.reopen")}</Button> : null}
      </div>
    </div>
  );
}

export default function AttendanceClosurePanel({ employerId, month, year }: AttendanceClosurePanelProps) {
  const { hasPermission, isLoading: isLoadingPermissions } = useMyPermissions();
  const [reason, setReason] = useState("");
  const [actionError, setActionError] = useState<string | null>(null);
  const canRead = hasPermission("attendance:closure:read");
  const { data, isLoading, isError, refetch, close, reopen, isClosing, isReopening, error } = useAttendanceClosure(
    year,
    month,
    employerId,
    isClosureQueryEnabled(canRead, isLoadingPermissions),
  );

  if (shouldHideClosurePanel(employerId, isLoadingPermissions, canRead)) return null;
  if (isLoading || isLoadingPermissions) return <Skeleton className="h-36 w-full rounded-3xl" data-testid="closure-skeleton" />;
  if (isError) return <ClosureLoadError onRetry={() => { void refetch(); }} />;
  if (!data) return <ClosureEmptyState />;

  const isClosed = data.status === "closed";
  const payrollLocked = data.payroll_status === "locked";
  const { canClose, canReopen } = getClosureActions(
    hasPermission("attendance:closure:close"),
    hasPermission("attendance:closure:reopen"),
    isClosed,
    payrollLocked,
  );
  const isMutating = isClosing || isReopening;
  const runAction = async (action: ClosureAction) => {
    const reasonError = getReasonError(reason);
    if (reasonError) {
      setActionError(reasonError);
      return;
    }
    setActionError(null);
    const mutationError = await executeClosureAction({ action, close, reason, reopen, version: data.version });
    if (mutationError) {
      setActionError(mutationError);
      return;
    }
    setReason("");
  };

  return (
    <section className="overflow-hidden rounded-3xl border border-slate-200 bg-white shadow-sm dark:border-slate-800 dark:bg-slate-900" data-testid="closure-panel">
      <ClosureHeader isClosed={isClosed} payrollLocked={payrollLocked} />
      {isClosed ? <ClosureEvidence data={data} /> : null}
      {canClose || canReopen ? <ClosureActionForm actionError={actionError} canClose={canClose} canReopen={canReopen} error={error} isClosing={isClosing} isMutating={isMutating} isReopening={isReopening} onAction={(action) => { void runAction(action); }} reason={reason} setReason={setReason} /> : null}
    </section>
  );
}
