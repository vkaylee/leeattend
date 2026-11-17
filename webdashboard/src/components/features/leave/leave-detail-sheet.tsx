import type { TFunction } from "i18next";

import {
  Calendar,
  CheckCircle2,
  FileText,
  User,
  XCircle,
} from "lucide-react";
import { useTranslation } from "react-i18next";

import type { Employee, LeaveRequest, LeaveType } from "@/lib/api/types";

import Button from "@/components/ui/button";
import Sheet from "@/components/ui/sheet";
import { cn } from "@/lib/utils";

export interface LeaveDetailSheetProps {
  emp?: Employee;
  formatDate: (d: string) => string;
  isOpen: boolean;
  leaveTypes: LeaveType[];
  onApprove?: (id: string, status: string) => void;
  onClose: () => void;
  request: LeaveRequest | null;
}

type Translate = TFunction;


interface StatusBadge {
  className: string;
  label: string;
}

function getStatusBadge(status: string, t: Translate): StatusBadge {
  if (status === "approved") {
    return {
      label: t("common.status_approved", { defaultValue: "Đã duyệt" }),
      className:
        "bg-emerald-50 text-emerald-600 dark:bg-emerald-950 dark:text-emerald-400 border-emerald-200 dark:border-emerald-800",
    };
  }

  if (status === "rejected") {
    return {
      label: t("common.status_rejected", "Từ chối"),
      className:
        "bg-rose-50 text-rose-600 dark:bg-rose-950 dark:text-rose-400 border-rose-200 dark:border-rose-800",
    };
  }

  return {
    label: t("common.status_pending", "Chờ duyệt"),
    className:
      "bg-amber-50 text-amber-600 dark:bg-amber-950 dark:text-amber-400 border-amber-200 dark:border-amber-800",
  };
}

function StatusHeader({
  status,
  typeName,
  t,
}: {
  status: string;
  t: Translate;
  typeName: string;
}) {
  const statusInfo = getStatusBadge(status, t);

  return (
    <div className="flex items-center justify-between rounded-xl border border-border bg-muted/40 p-4">
      <div className="space-y-1">
        <span className="text-xs font-semibold text-muted-foreground uppercase tracking-wider">
          {t("common.status", "Trạng thái")}
        </span>
        <div className="flex items-center gap-2">
          <span
            className={cn(
              "rounded-full border px-2.5 py-0.5 text-xs font-bold capitalize",
              statusInfo.className,
            )}
          >
            {statusInfo.label}
          </span>
        </div>
      </div>
      <div className="text-right space-y-1">
        <span className="text-xs font-semibold text-muted-foreground uppercase tracking-wider">
          {t("common.type", "Loại nghỉ")}
        </span>
        <div className="text-sm font-semibold text-foreground">{typeName}</div>
      </div>
    </div>
  );
}

function EmployeeInfo({ emp, t }: { emp?: Employee; t: Translate }) {
  return (
    <div className="space-y-3 rounded-xl border border-border p-4 bg-card">
      <div className="flex items-center gap-2 text-xs font-bold text-muted-foreground uppercase tracking-wider">
        <User className="h-4 w-4 text-primary" />
        <span>{t("common.employee", "Nhân sự")}</span>
      </div>
      <div className="flex items-center justify-between pt-1">
        <div>
          <div className="font-semibold text-foreground">{emp?.full_name ?? "Nhân viên"}</div>
          <div className="text-xs text-muted-foreground">
            {emp?.employee_id_number ? `Mã: ${emp.employee_id_number}` : ""}
          </div>
        </div>
        {!!emp?.department_name && (
          <span className="rounded-md bg-muted px-2 py-1 text-xs font-medium text-muted-foreground">
            {emp.department_name}
          </span>
        )}
      </div>
    </div>
  );
}

function LeaveDuration({
  formatDate,
  request,
  t,
}: {
  formatDate: (date: string) => string;
  request: LeaveRequest;
  t: Translate;
}) {
  return (
    <div className="space-y-3 rounded-xl border border-border p-4 bg-card">
      <div className="flex items-center gap-2 text-xs font-bold text-muted-foreground uppercase tracking-wider">
        <Calendar className="h-4 w-4 text-primary" />
        <span>{t("common.duration", "Thời gian nghỉ")}</span>
      </div>
      <div className="grid grid-cols-2 gap-4 pt-1">
        <div>
          <span className="text-xs text-muted-foreground">
            {t("common.start_date", "Từ ngày")}
          </span>
          <div className="text-sm font-semibold text-foreground">
            {formatDate(request.start_date)}
          </div>
        </div>
        <div>
          <span className="text-xs text-muted-foreground">
            {t("common.end_date", "Đến ngày")}
          </span>
          <div className="text-sm font-semibold text-foreground">
            {formatDate(request.end_date)}
          </div>
        </div>
      </div>
    </div>
  );
}

function LeaveReason({ request, t }: { request: LeaveRequest; t: Translate }) {
  return (
    <div className="space-y-3 rounded-xl border border-border p-4 bg-card">
      <div className="flex items-center gap-2 text-xs font-bold text-muted-foreground uppercase tracking-wider">
        <FileText className="h-4 w-4 text-primary" />
        <span>{t("common.reason", "Lý do xin nghỉ")}</span>
      </div>
      <p className="text-sm text-foreground/90 whitespace-pre-wrap rounded-lg bg-muted/30 p-3 leading-relaxed">
        {request.reason || t("common.no_notes", "Không có ghi chú")}
      </p>
    </div>
  );
}

function ApprovalActions({
  onApprove,
  onClose,
  request,
  t,
}: {
  onApprove: (id: string, status: string) => void;
  onClose: () => void;
  request: LeaveRequest;
  t: Translate;
}) {
  const approve = (status: "approved" | "rejected") => {
    onApprove(request.id, status);
    onClose();
  };

  return (
    <div className="flex items-center gap-3 pt-2">
      <Button
        className="flex-1 bg-emerald-600 hover:bg-emerald-700 text-white"
        data-testid="leave-sheet-approve-btn"
        onClick={() => { approve("approved"); }}
      >
        <CheckCircle2 className="h-4 w-4 mr-2" />
        {t("common.approve", "Phê duyệt")}
      </Button>
      <Button
        className="flex-1 text-rose-600 hover:bg-rose-50 dark:hover:bg-rose-950/30"
        data-testid="leave-sheet-reject-btn"
        onClick={() => { approve("rejected"); }}
        variant="outline"
      >
        <XCircle className="h-4 w-4 mr-2" />
        {t("common.reject", "Từ chối")}
      </Button>
    </div>
  );
}

export function LeaveDetailSheet({
  isOpen,
  onClose,
  request,
  emp,
  leaveTypes,
  formatDate,
  onApprove,
}: LeaveDetailSheetProps) {
  const { t } = useTranslation(["common", "leave"]);

  if (!request) return null;

  const leaveType = leaveTypes.find((lt) => lt.id === request.leave_type_id);
  const typeName = leaveType?.name || request.leave_type_id;
  const approvalHandler = request.status === "pending" ? onApprove : undefined;

  return (
    <Sheet
      className="w-full sm:max-w-lg"
      data-testid="leave-detail-sheet"
      isOpen={isOpen}
      onClose={onClose}
      side="right"
      subtitle={request.id}
      title={t("leave_requests.table.details", "Chi tiết yêu cầu nghỉ phép")}
    >
      <div className="space-y-6 p-6">
        <StatusHeader status={request.status} t={t} typeName={typeName} />
        <EmployeeInfo emp={emp} t={t} />
        <LeaveDuration formatDate={formatDate} request={request} t={t} />
        <LeaveReason request={request} t={t} />
        {approvalHandler ? (
          <ApprovalActions onApprove={approvalHandler} onClose={onClose} request={request} t={t} />
        ) : null}
      </div>
    </Sheet>
  );
}

export default LeaveDetailSheet;
