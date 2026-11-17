import type { ApprovalItemStatus, UnifiedApprovalItem } from "../types/unified-approvals";
import type { ItemBuilderContext } from "./unified-inbox-items";

function getRequester(
  employeeMap: ItemBuilderContext["employeeMap"],
  employmentId: string,
) {
  return employeeMap.get(employmentId) ?? {
    id: employmentId,
    name: employmentId,
  };
}

function buildLeaveItems({
  employeeMap,
  getTeamImpact,
  leaves,
  translate,
}: Pick<ItemBuilderContext, "employeeMap" | "getTeamImpact" | "leaves" | "translate">): UnifiedApprovalItem[] {
  return leaves.map((leave) => {
    const requester = getRequester(employeeMap, leave.employment_id);
    return {
      id: leave.id,
      type: "leave",
      status: leave.status as ApprovalItemStatus,
      requester,
      submittedAt: leave.created_at || new Date().toISOString(),
      targetDate: leave.start_date,
      title: translate("approvals.type.leave", "Đơn xin nghỉ phép"),
      subtitle: `${leave.start_date} → ${leave.end_date} (${leave.duration_type})`,
      details: {
        dateStr:
          leave.start_date === leave.end_date
            ? leave.start_date
            : `${leave.start_date} → ${leave.end_date}`,
        reason: leave.reason ?? undefined,
        durationLabel: leave.duration_type,
        rawPayload: leave,
      },
      impact: getTeamImpact(leave.start_date, requester.departmentName, leave.id),
    };
  });
}

function buildAdjustmentItems({
  adjustments,
  employeeMap,
  getTeamImpact,
  translate,
}: Pick<ItemBuilderContext, "adjustments" | "employeeMap" | "getTeamImpact" | "translate">): UnifiedApprovalItem[] {
  return adjustments.map((adjustment) => {
    const requester = getRequester(employeeMap, adjustment.employment_id);
    const targetDate = adjustment.requested_punch_time.split("T")[0] || adjustment.created_at;

    return {
      id: adjustment.id,
      type: "adjustment",
      status: adjustment.status as ApprovalItemStatus,
      requester,
      submittedAt: adjustment.created_at || new Date().toISOString(),
      targetDate,
      title: translate("approvals.type.adjustment", "Giải trình chấm công"),
      subtitle: `Thời gian yêu cầu: ${adjustment.requested_punch_time.replace("T", " ").slice(0, 16)}`,
      details: {
        dateStr: adjustment.requested_punch_time,
        reason: adjustment.reason ?? undefined,
        rawPayload: adjustment as unknown as Record<string, unknown>,
      },
      impact: getTeamImpact(targetDate, requester.departmentName, adjustment.id),
    };
  });
}

function buildSwapItems({
  employeeMap,
  getTeamImpact,
  swaps,
  translate,
}: Pick<ItemBuilderContext, "employeeMap" | "getTeamImpact" | "swaps" | "translate">): UnifiedApprovalItem[] {
  return swaps.map((swap) => {
    const requester = getRequester(employeeMap, swap.requester_id);
    const status: ApprovalItemStatus =
      swap.status === "pending_admin" || swap.status === "pending_target"
        ? "pending"
        : swap.status;

    return {
      id: swap.id,
      type: "shift_swap",
      status,
      requester,
      submittedAt: swap.created_at || new Date().toISOString(),
      targetDate: swap.swap_date,
      title: translate("approvals.type.shift_swap", "Đổi ca làm việc"),
      subtitle: `Ngày đổi ca: ${swap.swap_date}`,
      details: {
        dateStr: swap.swap_date,
        reason: swap.reason ?? undefined,
        rawPayload: swap as unknown as Record<string, unknown>,
      },
      impact: getTeamImpact(swap.swap_date, requester.departmentName, swap.id),
    };
  });
}

function buildOvertimeItems({
  employeeMap,
  getTeamImpact,
  overtime,
  translate,
}: Pick<ItemBuilderContext, "employeeMap" | "getTeamImpact" | "overtime" | "translate">): UnifiedApprovalItem[] {
  return overtime.flatMap((summary) => {
    if (!summary.claimed_ot_minutes && !summary.approved_ot_minutes) return [];

    const requester = getRequester(employeeMap, summary.employment_id);
    const status: ApprovalItemStatus =
      summary.ot_status === "approved"
        ? "approved"
        : summary.ot_status === "rejected"
          ? "rejected"
          : "pending";
    const claimedMinutes = summary.claimed_ot_minutes || 60;

    return [{
      id: summary.id || `${summary.employment_id}_${summary.date}`,
      type: "overtime",
      status,
      requester,
      submittedAt: summary.date,
      targetDate: summary.date,
      title: translate("approvals.type.overtime", "Xác nhận tăng ca (OT)"),
      subtitle: `Thời lượng: ${claimedMinutes} phút (${(claimedMinutes / 60).toFixed(1)}h)`,
      details: {
        dateStr: summary.date,
        reason: summary.ot_reason ?? undefined,
        durationLabel: `${claimedMinutes} phút`,
        rawPayload: summary as unknown as Record<string, unknown>,
      },
      impact: getTeamImpact(summary.date, requester.departmentName, summary.id),
    }];
  });
}

export function buildApprovalItems(context: ItemBuilderContext): UnifiedApprovalItem[] {
  const items = [
    ...buildLeaveItems(context),
    ...buildAdjustmentItems(context),
    ...buildSwapItems(context),
    ...buildOvertimeItems(context),
  ];

  return items.sort((left, right) => {
    const leftDate = left.submittedAt || left.targetDate;
    const rightDate = right.submittedAt || right.targetDate;
    return rightDate.localeCompare(leftDate);
  });
}
