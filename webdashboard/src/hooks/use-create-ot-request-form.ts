import { useState } from "react";
import { useTranslation } from "react-i18next";

import { format } from "date-fns";

import { useToast } from "@/components/ui/toast";
import { useBehalfEmployees } from "@/hooks/use-behalf-employees";
import { useCreateOvertimeRequest } from "@/hooks/use-overtime-requests";

export function useCreateOtRequestForm(onSuccess?: () => void) {
  const { t } = useTranslation();
  const toast = useToast();

  const [open, setOpen] = useState(false);
  const [selectedEmployees, setSelectedEmployees] = useState<string[]>([]);
  const [plannedDate, setPlannedDate] = useState(format(new Date(), "yyyy-MM-dd"));
  const [plannedStartTime, setPlannedStartTime] = useState("17:00");
  const [plannedEndTime, setPlannedEndTime] = useState("19:00");
  const [reason, setReason] = useState("");

  const { employees, hasAssignments, isLoading: behalfLoading } = useBehalfEmployees("overtime_request");
  const { mutate: createRequest, isPending } = useCreateOvertimeRequest();

  const calculateMinutes = () => {
    if (!plannedStartTime || !plannedEndTime) return 0;
    const [startH, startM] = plannedStartTime.split(":").map(Number);
    const [endH, endM] = plannedEndTime.split(":").map(Number);
    let minutes = (endH * 60 + endM) - (startH * 60 + startM);
    if (minutes < 0) minutes += 24 * 60; // Crosses midnight
    return minutes;
  };

  const handleCreate = () => {
    if (selectedEmployees.length === 0) {
      toast.error(t("approvals.pre_ot_dialog.validation.select_employee"));
      return;
    }
    if (!reason.trim()) {
      toast.error(t("approvals.pre_ot_dialog.validation.provide_reason"));
      return;
    }

    const minutes = calculateMinutes();
    if (minutes <= 0) {
      toast.error(t("approvals.pre_ot_dialog.validation.invalid_time"));
      return;
    }

    createRequest(
      {
        planned_date: plannedDate,
        planned_start_time: plannedStartTime + ":00",
        planned_end_time: plannedEndTime + ":00",
        planned_minutes: minutes,
        reason,
        employees: selectedEmployees,
      },
      {
        onSuccess: () => {
          toast.success(t("approvals.pre_ot_dialog.success_message"));
          setOpen(false);
          onSuccess?.();
          // Reset
          setSelectedEmployees([]);
          setReason("");
        },
        onError: (err: unknown) => {
          const apiErr = err as { response?: { data?: { error?: string } } };
          toast.error(apiErr.response?.data?.error ?? t("approvals.pre_ot_dialog.error_message"));
        },
      }
    );
  };

  const toggleEmployee = (id: string) => {
    setSelectedEmployees(prev =>
      prev.includes(id) ? prev.filter(e => e !== id) : [...prev, id]
    );
  };

  return {
    behalfLoading,
    calculateMinutes,
    employees,
    handleCreate,
    hasAssignments,
    isPending,
    open,
    plannedDate,
    plannedEndTime,
    plannedStartTime,
    reason,
    selectedEmployees,
    setOpen,
    setPlannedDate,
    setPlannedEndTime,
    setPlannedStartTime,
    setReason,
    setSelectedEmployees,
    toggleEmployee,
  };
}
