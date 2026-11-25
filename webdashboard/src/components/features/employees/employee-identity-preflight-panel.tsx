import type { TFunction } from "i18next";

import { AlertCircle, AlertTriangle, CheckCircle2, Link2, Loader2, RefreshCw, ShieldAlert, UserCheck } from "lucide-react";

import type { EmployeeMatchCandidate } from "@/lib/api";
import type { PreflightErrorKind, PreflightStatus } from "@/pages/employees/hooks/use-employee-identity-preflight";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";

interface PreflightPanelProps {
  candidates: EmployeeMatchCandidate[];
  canRehire: boolean;
  conflictMessage: string | null;
  errorKind: PreflightErrorKind;
  hasStagingLog: boolean;
  isAssigningStaging: boolean;
  onAcknowledgeSoft: () => void;
  onAssignStagingCandidate?: (candidate: EmployeeMatchCandidate) => void;
  onCancelReuse: () => void;
  onRetry: () => void;
  onSelectReuse: (candidate: EmployeeMatchCandidate) => void;
  selectedReuseCandidate: EmployeeMatchCandidate | null;
  softAcknowledged: boolean;
  status: PreflightStatus;
  t: TFunction;
}

interface CandidateListProps {
  candidates: EmployeeMatchCandidate[];
  canRehire: boolean;
  hasStagingLog: boolean;
  isAssigningStaging: boolean;
  onAssignStagingCandidate?: (candidate: EmployeeMatchCandidate) => void;
  onSelectReuse: (candidate: EmployeeMatchCandidate) => void;
  selectedId?: string;
  t: TFunction;
}

function getReasonLabel(reason: EmployeeMatchCandidate["reasons"][number], t: TFunction): string {
  switch (reason) {
    case "national_id_exact":
      return t("employees.preflight.reasons.national_id_exact", { defaultValue: "Trùng CCCD / CMND" });
    case "phone_exact":
      return t("employees.preflight.reasons.phone_exact", { defaultValue: "Trùng số điện thoại" });
    case "name_dob_review":
      return t("employees.preflight.reasons.name_dob_review", { defaultValue: "Trùng họ tên và ngày sinh" });
    case "name_phone_review":
      return t("employees.preflight.reasons.name_phone_review", { defaultValue: "Trùng họ tên và số điện thoại gần đúng" });
  }
}

function getErrorMessage(errorKind: PreflightErrorKind, t: TFunction): string {
  if (errorKind === "rate_limited") {
    return t("employees.preflight.errors.rate_limited", { defaultValue: "Bạn kiểm tra quá nhanh. Vui lòng chờ một chút rồi thử lại." });
  }
  if (errorKind === "unavailable") {
    return t("employees.preflight.errors.unavailable", { defaultValue: "Dịch vụ kiểm tra trùng nhân viên tạm thời không khả dụng." });
  }
  if (errorKind === "forbidden") {
    return t("employees.preflight.errors.forbidden", { defaultValue: "Bạn không có quyền kiểm tra trùng danh tính." });
  }
  return t("employees.preflight.errors.generic", { defaultValue: "Không thể kiểm tra trùng nhân viên lúc này." });
}

function CandidateActionButtons({
  candidate,
  canRehire,
  hasStagingLog,
  isAssigningStaging,
  isSelected,
  onAssignStagingCandidate,
  onSelectReuse,
  t,
}: {
  candidate: EmployeeMatchCandidate;
  canRehire: boolean;
  hasStagingLog: boolean;
  isAssigningStaging: boolean;
  isSelected: boolean;
  onAssignStagingCandidate?: (candidate: EmployeeMatchCandidate) => void;
  onSelectReuse: (candidate: EmployeeMatchCandidate) => void;
  t: TFunction;
}) {
  const isTerminated = candidate.status === "terminated";
  const showAssign = hasStagingLog && Boolean(onAssignStagingCandidate);

  return (
    <div className="flex shrink-0 flex-wrap items-center gap-2">
      {showAssign ? (
        <Button
          data-testid={`assign-staging-candidate-${candidate.employee_id}`}
          disabled={isAssigningStaging}
          onClick={() => { onAssignStagingCandidate?.(candidate); }}
          size="sm"
          type="button"
          variant="outline"
        >
          <Link2 aria-hidden="true" size={14} />
          {t("employees.preflight.assign_staging", { defaultValue: "Gán bản ghi chấm công" })}
        </Button>
      ) : null}
      {isTerminated && canRehire ? (
        <Button
          data-testid={`reuse-candidate-${candidate.employee_id}`}
          disabled={isSelected}
          onClick={() => { onSelectReuse(candidate); }}
          size="sm"
          type="button"
          variant={isSelected ? "secondary" : "primary"}
        >
          <UserCheck aria-hidden="true" size={14} />
          {isSelected
            ? t("employees.preflight.selected_for_reuse", { defaultValue: "Đã chọn dùng lại" })
            : t("employees.preflight.reuse_existing", { defaultValue: "Dùng lại nhân viên hiện có" })}
        </Button>
      ) : !isTerminated ? (
        <span className="text-xs font-medium text-rose-600 dark:text-rose-400">
          {t("employees.preflight.active_cannot_duplicate", { defaultValue: "Nhân viên đang hoạt động, không thể tạo trùng" })}
        </span>
      ) : null}
    </div>
  );
}

function CandidateList({
  candidates,
  canRehire,
  hasStagingLog,
  isAssigningStaging,
  onAssignStagingCandidate,
  onSelectReuse,
  selectedId,
  t,
}: CandidateListProps) {
  return (
    <ul className="space-y-2">
      {candidates.map((candidate) => {
        const isTerminated = candidate.status === "terminated";
        return (
          <li
            className="flex flex-col gap-3 rounded-xl border border-slate-200/80 bg-white p-4 shadow-sm dark:border-slate-700/80 dark:bg-slate-900 sm:flex-row sm:items-center sm:justify-between"
            data-testid={`match-candidate-${candidate.employee_id}`}
            key={candidate.employee_id}
          >
            <div className="min-w-0 space-y-1.5">
              <div className="flex flex-wrap items-center gap-2">
                <span className="font-semibold text-slate-900 dark:text-slate-100">{candidate.display_name}</span>
                <span className="rounded-md bg-slate-100 px-2 py-0.5 font-mono text-xs font-semibold text-slate-700 dark:bg-slate-800 dark:text-slate-300">
                  {candidate.employee_code}
                </span>
                <span
                  className={
                    isTerminated
                      ? "rounded-full bg-amber-100 px-2.5 py-0.5 text-xs font-semibold text-amber-800 dark:bg-amber-900/50 dark:text-amber-200"
                      : "rounded-full bg-emerald-100 px-2.5 py-0.5 text-xs font-semibold text-emerald-800 dark:bg-emerald-900/50 dark:text-emerald-200"
                  }
                >
                  {isTerminated
                    ? t("employees.preflight.status.terminated", { defaultValue: "Đã nghỉ việc" })
                    : t("employees.preflight.status.active", { defaultValue: "Đang làm việc" })}
                </span>
              </div>
              <div className="flex flex-wrap gap-1.5">
                {candidate.reasons.map((reason) => (
                  <span className="rounded-md bg-slate-100/90 px-2 py-0.5 text-xs text-slate-600 dark:bg-slate-800 dark:text-slate-300" key={reason}>
                    {getReasonLabel(reason, t)}
                  </span>
                ))}
              </div>
            </div>
            <CandidateActionButtons
              candidate={candidate}
              canRehire={canRehire}
              hasStagingLog={hasStagingLog}
              isAssigningStaging={isAssigningStaging}
              isSelected={selectedId === candidate.employee_id}
              onAssignStagingCandidate={onAssignStagingCandidate}
              onSelectReuse={onSelectReuse}
              t={t}
            />
          </li>
        );
      })}
    </ul>
  );
}

function PreflightStatusBanner({ errorKind, status, t }: { errorKind: PreflightErrorKind; status: PreflightStatus; t: TFunction }) {
  if (status === "loading") {
    return (
      <div className="flex items-center gap-2.5 text-sm text-slate-600 dark:text-slate-300" data-testid="identity-preflight-loading" role="status">
        <Loader2 aria-hidden="true" className="animate-spin" size={16} />
        <span>{t("employees.preflight.loading", { defaultValue: "Đang kiểm tra hồ sơ trùng..." })}</span>
      </div>
    );
  }
  if (status === "stale") {
    return (
      <div className="flex items-center gap-2.5 rounded-xl border border-slate-200 bg-white px-4 py-3 text-sm text-slate-600 dark:border-slate-700 dark:bg-slate-900 dark:text-slate-300" data-testid="identity-preflight-stale" role="status">
        <RefreshCw aria-hidden="true" size={16} />
        <span>{t("employees.preflight.stale", { defaultValue: "Thông tin định danh đã thay đổi. Đang chờ kiểm tra lại..." })}</span>
      </div>
    );
  }
  if (status === "none") {
    return (
      <div className="flex items-center gap-2.5 rounded-xl border border-emerald-200 bg-emerald-50/80 px-4 py-3 text-sm text-emerald-800 dark:border-emerald-900/60 dark:bg-emerald-950/40 dark:text-emerald-200" data-testid="identity-preflight-none" role="status">
        <CheckCircle2 aria-hidden="true" size={16} />
        <span>{t("employees.preflight.none", { defaultValue: "Chưa phát hiện hồ sơ trùng khớp. Bạn có thể tạo nhân viên mới." })}</span>
      </div>
    );
  }
  if (status === "error") {
    return (
      <div className="flex items-start gap-2.5 rounded-xl border border-rose-200 bg-rose-50/80 px-4 py-3 text-sm text-rose-800 dark:border-rose-900/60 dark:bg-rose-950/40 dark:text-rose-200" data-testid="identity-preflight-error" role="alert">
        <AlertCircle aria-hidden="true" className="mt-0.5 shrink-0" size={16} />
        <span>{getErrorMessage(errorKind, t)}</span>
      </div>
    );
  }
  return null;
}

export default function EmployeeIdentityPreflightPanel({
  candidates,
  canRehire,
  conflictMessage,
  errorKind,
  hasStagingLog,
  isAssigningStaging,
  onAcknowledgeSoft,
  onAssignStagingCandidate,
  onCancelReuse,
  onRetry,
  onSelectReuse,
  selectedReuseCandidate,
  softAcknowledged,
  status,
  t,
}: PreflightPanelProps) {
  if (status === "idle" && !conflictMessage && !selectedReuseCandidate) {
    return null;
  }

  return (
    <section
      aria-labelledby="identity-preflight-heading"
      aria-live="polite"
      className="space-y-4 rounded-2xl border border-slate-200/80 bg-slate-50/70 p-5 dark:border-slate-800 dark:bg-slate-900/60"
      data-testid="identity-preflight-panel"
    >
      <div className="flex flex-wrap items-center justify-between gap-2">
        <Typography className="text-sm font-bold uppercase tracking-wider text-slate-700 dark:text-slate-200" id="identity-preflight-heading" variant="h2">
          {t("employees.preflight.title", { defaultValue: "Kiểm tra trùng hồ sơ nhân viên" })}
        </Typography>
        {status === "error" || status === "stale" ? (
          <Button data-testid="identity-preflight-retry-btn" onClick={onRetry} size="sm" type="button" variant="outline">
            <RefreshCw aria-hidden="true" size={14} />
            {t("employees.preflight.retry", { defaultValue: "Kiểm tra lại" })}
          </Button>
        ) : null}
      </div>

      {conflictMessage ? (
        <div className="flex items-start gap-3 rounded-xl border border-rose-200 bg-rose-50 p-4 text-sm text-rose-800 dark:border-rose-900/60 dark:bg-rose-950/40 dark:text-rose-200" data-testid="identity-preflight-conflict" role="alert">
          <AlertCircle aria-hidden="true" className="mt-0.5 shrink-0" size={18} />
          <Typography className="text-sm text-inherit" variant="p">{conflictMessage}</Typography>
        </div>
      ) : null}

      {selectedReuseCandidate ? (
        <div className="flex flex-col gap-3 rounded-xl border border-primary-200 bg-primary-50/80 p-4 text-sm text-primary-900 dark:border-primary-800/60 dark:bg-primary-950/40 dark:text-primary-100 sm:flex-row sm:items-center sm:justify-between" data-testid="identity-preflight-reuse-selected" role="status">
          <div>
            <Typography className="text-sm font-semibold text-inherit" variant="p">
              {t("employees.preflight.reuse_mode_title", {
                defaultValue: "Đang tái sử dụng hồ sơ: {{name}} ({{code}})",
                name: selectedReuseCandidate.display_name,
                code: selectedReuseCandidate.employee_code,
              })}
            </Typography>
            <Typography className="mt-0.5 text-xs text-inherit opacity-90" variant="p">
              {t("employees.preflight.reuse_mode_hint", {
                defaultValue: "Lưu sẽ giữ nguyên mã và hồ sơ nhân viên, chỉ tạo giai đoạn công tác mới.",
              })}
            </Typography>
          </div>
          <Button data-testid="cancel-reuse-btn" onClick={onCancelReuse} size="sm" type="button" variant="outline">
            {t("employees.preflight.cancel_reuse", { defaultValue: "Hủy dùng lại" })}
          </Button>
        </div>
      ) : null}

      <PreflightStatusBanner errorKind={errorKind} status={status} t={t} />

      {status === "hard" ? (
        <div className="space-y-3" data-testid="identity-preflight-hard" role="alert">
          <div className="flex items-start gap-2.5 rounded-xl border border-rose-200 bg-rose-50 px-4 py-3 text-sm text-rose-800 dark:border-rose-900/60 dark:bg-rose-950/40 dark:text-rose-200">
            <ShieldAlert aria-hidden="true" className="mt-0.5 shrink-0" size={18} />
            <Typography className="text-sm text-inherit" variant="p">
              {t("employees.preflight.hard_warning", {
                defaultValue: "Phát hiện hồ sơ trùng định danh chính xác. Không thể tạo mới trùng lặp — hãy chọn dùng lại hồ sơ hiện có hoặc chỉnh lại thông tin.",
              })}
            </Typography>
          </div>
          <CandidateList
            candidates={candidates}
            canRehire={canRehire}
            hasStagingLog={hasStagingLog}
            isAssigningStaging={isAssigningStaging}
            onAssignStagingCandidate={onAssignStagingCandidate}
            onSelectReuse={onSelectReuse}
            selectedId={selectedReuseCandidate?.employee_id}
            t={t}
          />
        </div>
      ) : null}

      {status === "soft" ? (
        <div className="space-y-3" data-testid="identity-preflight-soft" role="status">
          <div className="flex flex-col gap-3 rounded-xl border border-amber-200 bg-amber-50 px-4 py-3 text-sm text-amber-900 dark:border-amber-900/60 dark:bg-amber-950/40 dark:text-amber-200 sm:flex-row sm:items-center sm:justify-between">
            <div className="flex items-start gap-2.5">
              <AlertTriangle aria-hidden="true" className="mt-0.5 shrink-0" size={18} />
              <Typography className="text-sm text-inherit" variant="p">
                {t("employees.preflight.soft_warning", {
                  defaultValue: "Có hồ sơ gần giống cần đối chiếu. Hệ thống không tự động gộp — vui lòng kiểm tra trước khi tiếp tục.",
                })}
              </Typography>
            </div>
            {selectedReuseCandidate ? null : (
              <Button aria-pressed={softAcknowledged} data-testid="soft-match-ack-btn" onClick={onAcknowledgeSoft} size="sm" type="button" variant={softAcknowledged ? "secondary" : "outline"}>
                {softAcknowledged
                  ? t("employees.preflight.soft_acknowledged", { defaultValue: "Đã xác nhận khác người — tiếp tục tạo mới" })
                  : t("employees.preflight.not_this_person", { defaultValue: "Không phải người này — tiếp tục tạo mới" })}
              </Button>
            )}
          </div>
          <CandidateList
            candidates={candidates}
            canRehire={canRehire}
            hasStagingLog={hasStagingLog}
            isAssigningStaging={isAssigningStaging}
            onAssignStagingCandidate={onAssignStagingCandidate}
            onSelectReuse={onSelectReuse}
            selectedId={selectedReuseCandidate?.employee_id}
            t={t}
          />
        </div>
      ) : null}
    </section>
  );
}
