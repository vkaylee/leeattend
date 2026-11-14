import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useTranslation } from "react-i18next";

import { useToast } from "@/components/ui/toast";
import {
    reprocessAttendance,
    adminConfirmAttendance,
    approveOt,
    upsertRoster,
    DailyAttendanceReport,
} from "@/lib/api";
import { createQueryKey } from "@/lib/query-keys";


interface UseDailyDrawerMutationsProps {
    employeeId?: string;
    onConfirmSuccess: () => void;
    onReprocessSuccess: () => void;
    onResolveSuccess: () => void;
}

export function useDailyDrawerMutations({employeeId,
    onReprocessSuccess,
    onConfirmSuccess,
    onResolveSuccess,
}: UseDailyDrawerMutationsProps) {
    const { t } = useTranslation();
    const queryClient = useQueryClient();
    const toast = useToast();

    const reprocessMutation = useMutation({
        mutationFn: ({ empId, date }: { empId: string; date: string }) =>
            reprocessAttendance(empId, date, date),
        onSuccess: () => {
            void queryClient.invalidateQueries({ queryKey: createQueryKey("attendance-report") });
            toast.success(t("common.success"));
            onReprocessSuccess();
        },
    });

    const confirmMutation = useMutation({
        mutationFn: ({
            row,
            existingRequestId,
            reason,
        }: {
            row: DailyAttendanceReport;
            existingRequestId?: string;
            reason?: string;
        }) => adminConfirmAttendance(row, existingRequestId, reason),
        onSuccess: () => {
            void queryClient.invalidateQueries({ queryKey: createQueryKey("attendance-report") });
            void queryClient.invalidateQueries({ queryKey: createQueryKey("attendance-adjustments") });
            toast.success(t("reports.confirm.success"));
            onConfirmSuccess();
        },
        onError: () => {
            toast.error(t("reports.confirm.error"));
        },
    });

    const approveOtMutation = useMutation({
        mutationFn: ({ row, action }: { row: DailyAttendanceReport; action: "approved" | "rejected" }) =>
            approveOt(employeeId ?? "", {
                date: row.date,
                approved_minutes: Math.round(row.computed_ot_hours * 60),
                status: action,
            }),
        onSuccess: (_data, variables) => {
            void queryClient.invalidateQueries({ queryKey: createQueryKey("attendance-report") });
            const message =
                variables.action === "approved"
                    ? t("reports.ot_approve.success_approved")
                    : t("reports.ot_approve.success_rejected");
            toast.success(message);
        },
        onError: () => {
            toast.error(t("reports.ot_approve.error"));
        },
    });

    const resolveUnscheduledMutation = useMutation({
        mutationFn: async ({ row, shiftId }: { row: DailyAttendanceReport; shiftId: string }) => {
            const empId = employeeId ?? "";
            await upsertRoster(empId, {
                date: row.date,
                shift_id: shiftId,
            });
            await reprocessAttendance(empId, row.date, row.date);
        },
        onSuccess: () => {
            void queryClient.invalidateQueries({ queryKey: createQueryKey("attendance-report") });
            toast.success(t("reports.unscheduled.success"));
            onResolveSuccess();
        },
        onError: () => {
            toast.error(t("reports.unscheduled.error"));
        },
    });

    return {
        reprocessMutation,
        confirmMutation,
        approveOtMutation,
        resolveUnscheduledMutation,
    };
}
