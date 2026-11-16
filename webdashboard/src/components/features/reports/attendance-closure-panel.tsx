import { AlertCircle, CheckCircle2, LockKeyhole, UnlockKeyhole } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { useAttendanceClosure } from "@/hooks/use-attendance-closure";
import { useMyPermissions } from "@/hooks/use-permissions";
import { ApiError } from "@/lib/api/errors";

interface AttendanceClosurePanelProps {
  employerId: string | null;
  month: number;
  year: number;
}

const ERROR_KEYS: Record<string, string> = {
  "error.closure_conflict": "closure.errors.closure_conflict",
  "error.locked_period": "closure.errors.locked_period",
  "error.missing_coverage": "closure.errors.missing_coverage",
  "error.open_punch": "closure.errors.open_punch",
  "error.stale_period": "closure.errors.stale_period",
  VALIDATION_ERROR: "closure.errors.validation",
};

function getErrorKey(error: unknown): string {
  if (error instanceof ApiError && error.errorCode) {
    return ERROR_KEYS[error.errorCode] ?? "closure.errors.generic";
  }
  return "closure.errors.generic";
}

export default function AttendanceClosurePanel({ employerId, month, year }: AttendanceClosurePanelProps) {
  const { t } = useTranslation();
  const { hasPermission, isLoading: isLoadingPermissions } = useMyPermissions();
  const [reason, setReason] = useState("");
  const [actionError, setActionError] = useState<string | null>(null);
  const canRead = hasPermission("attendance:closure:read");
  const { data, isLoading, isError, refetch, close, reopen, isClosing, isReopening, error } = useAttendanceClosure(
    year,
    month,
    employerId,
    canRead && !isLoadingPermissions,
  );

  if (!employerId || (!isLoadingPermissions && !canRead)) return null;
  if (isLoading || isLoadingPermissions) {
    return <Skeleton className="h-36 w-full rounded-3xl" data-testid="closure-skeleton" />;
  }
  if (isError) {
    return (
      <section className="rounded-3xl border border-rose-200 bg-rose-50 p-5 text-rose-900 dark:border-rose-900/60 dark:bg-rose-950/30 dark:text-rose-100" role="alert">
        <div className="flex items-start gap-3">
          <AlertCircle aria-hidden="true" className="mt-0.5 shrink-0" size={20} />
          <div className="space-y-2">
            <p className="font-semibold">{t("closure.errors.load")}</p>
            <Button onClick={() => void refetch()} size="sm" variant="outline">{t("closure.retry")}</Button>
          </div>
        </div>
      </section>
    );
  }
  if (!data) {
    return (
      <section className="rounded-3xl border border-slate-200 bg-white p-5 text-slate-700 dark:border-slate-800 dark:bg-slate-900 dark:text-slate-200" data-testid="closure-empty">
        <p className="font-semibold">{t("closure.empty.title")}</p>
        <p className="mt-1 text-sm text-slate-500 dark:text-slate-400">{t("closure.empty.description")}</p>
      </section>
    );
  }

  const isClosed = data.status === "closed";
  const payrollLocked = data.payroll_status === "locked";
  const canClose = canRead && hasPermission("attendance:closure:close") && !isClosed && !payrollLocked;
  const canReopen = canRead && hasPermission("attendance:closure:reopen") && isClosed && !payrollLocked;
  const isMutating = isClosing || isReopening;
  const runAction = async (action: "close" | "reopen") => {
    if (!reason.trim()) {
      setActionError("closure.errors.reason_required");
      return;
    }
    setActionError(null);
    try {
      await (action === "close" ? close : reopen)({ reason: reason.trim(), version: data.version });
      setReason("");
    } catch (mutationError) {
      setActionError(getErrorKey(mutationError));
    }
  };

  return (
    <section className="overflow-hidden rounded-3xl border border-slate-200 bg-white shadow-sm dark:border-slate-800 dark:bg-slate-900" data-testid="closure-panel">
      <div className="flex flex-col gap-4 border-b border-slate-100 p-5 sm:flex-row sm:items-center sm:justify-between dark:border-slate-800">
        <div className="flex items-start gap-3">
          {isClosed ? <LockKeyhole aria-hidden="true" className="mt-0.5 text-emerald-600" size={21} /> : <UnlockKeyhole aria-hidden="true" className="mt-0.5 text-slate-500" size={21} />}
          <div>
            <h2 className="font-semibold text-slate-900 dark:text-slate-100">{t("closure.title")}</h2>
            <p className="mt-1 text-sm text-slate-500 dark:text-slate-400">{t(isClosed ? "closure.status.closed" : "closure.status.open")}</p>
          </div>
        </div>
        <div className={isClosed ? "inline-flex items-center gap-2 rounded-full bg-emerald-50 px-3 py-1.5 text-sm font-semibold text-emerald-700 dark:bg-emerald-950/40 dark:text-emerald-300" : "inline-flex items-center gap-2 rounded-full bg-slate-100 px-3 py-1.5 text-sm font-semibold text-slate-700 dark:bg-slate-800 dark:text-slate-200"}>
          <CheckCircle2 aria-hidden="true" size={16} />
          {t(isClosed ? "closure.badge.closed" : "closure.badge.open")}
        </div>
        {!!payrollLocked && <p className="text-sm text-amber-700 dark:text-amber-300" role="status">{t("closure.errors.locked_period")}</p>}
      </div>
      {!!isClosed && (
        <div className="grid gap-3 border-b border-slate-100 p-5 text-sm dark:border-slate-800 sm:grid-cols-3" data-testid="closure-evidence">
          <span>{t("closure.evidence.number")}: {data.closure_number ?? "—"}</span>
          <span>{t("closure.evidence.days")}: {data.day_count ?? "—"}</span>
          <span className="truncate" title={data.snapshot_hash ?? undefined}>{t("closure.evidence.hash")}: {data.snapshot_hash ?? "—"}</span>
        </div>
      )}
      {!!(canClose || canReopen) && (
        <div className="space-y-4 p-5">
          <label className="block text-sm font-medium text-slate-700 dark:text-slate-200" htmlFor="closure-reason">{t("closure.reason")}</label>
          <textarea aria-describedby="closure-reason-hint" className="min-h-20 w-full rounded-2xl border border-slate-200 bg-slate-50 px-4 py-3 text-sm outline-none transition focus-visible:ring-2 focus-visible:ring-primary-500 dark:border-slate-700 dark:bg-slate-950" id="closure-reason" maxLength={2000} onChange={(event) => { setReason(event.target.value); }} placeholder={t("closure.reason_placeholder")} value={reason} />
          <p className="text-xs text-slate-500 dark:text-slate-400" id="closure-reason-hint">{t("closure.reason_hint")}</p>
          {!!(actionError || error) && <p className="text-sm font-medium text-rose-700 dark:text-rose-300" role="alert">{t(actionError ?? getErrorKey(error))}</p>}
          <div className="flex flex-wrap gap-3">
            {!!canClose && <Button disabled={isMutating} loading={isClosing} onClick={() => void runAction("close")}>{t("closure.actions.close")}</Button>}
            {!!canReopen && <Button disabled={isMutating} loading={isReopening} onClick={() => void runAction("reopen")} variant="outline">{t("closure.actions.reopen")}</Button>}
          </div>
        </div>
      )}
    </section>
  );
}
