import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useTranslation } from "react-i18next";

import type {
  AssignPatternRequest,
  EmployeeWorkingPattern,
  Shift,
  ShiftRoster,
  CreateShiftRequest,
  UpdateShiftRequest,
} from "@/lib/api/types";

import { useToast } from "@/components/ui/toast";
import {
  assignPattern,
  bulkUpsertRoster,
  createShift,
  deleteRoster,
  updateShift,
  upsertRoster,
} from "@/lib/api";
import { ApiError, ComplianceConflictError, parseComplianceConflict } from "@/lib/api/errors";
import { createQueryKey } from "@/lib/query-keys";


interface UseShiftMutationsProps { onComplianceConflict?: (error: ComplianceConflictError) => void;
onSuccess?: (type: string) => void;
selectedEmployeeId: string | undefined;
selectedEmploymentId: string | undefined;  }

export function useShiftMutations({ selectedEmployeeId, selectedEmploymentId, onSuccess, onComplianceConflict }: UseShiftMutationsProps) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const toast = useToast();

  /** Detect a 409 compliance conflict and forward to the callback if present. */
  const handlePossibleComplianceConflict = (error: Error): boolean => {
    if (error instanceof ComplianceConflictError) {
      onComplianceConflict?.(error);
      return true;
    }
    if (error instanceof ApiError && error.isConflict) {
      const retry = parseComplianceConflict(error.errorBody);
      if (retry) {
        onComplianceConflict?.(
          new ComplianceConflictError(error.status, error.statusText, error.errorBody, retry),
        );
        return true;
      }
    }
    return false;
  };

  const assignPatternMutation = useMutation<EmployeeWorkingPattern, Error, AssignPatternRequest>({
    mutationFn: (data: AssignPatternRequest) => {
      if (!selectedEmploymentId) return Promise.reject(new Error("No active employment found"));
      return assignPattern(selectedEmploymentId, data);
    },
    onSuccess: () => {
      toast.success(t("shifts.messages.assign_success"));
      onSuccess?.("assignPattern");
      void queryClient.invalidateQueries({ queryKey: createQueryKey("schedules") });
      void queryClient.invalidateQueries({ queryKey: createQueryKey("assignment_history") });
    },
    onError: (error: Error) => { toast.error(error.message, t("shifts.messages.assign_error")); }
  });

  const rosterMutation = useMutation<ShiftRoster, Error, { date: string; shift_id: string | null; notes?: string }>({
    mutationFn: (data: { date: string; shift_id: string | null; notes?: string }) => {
      if (!selectedEmployeeId) return Promise.reject(new Error("No employee selected"));
      return upsertRoster(selectedEmployeeId, data);
    },
    onSuccess: () => {
      toast.success(t("shifts.messages.update_roster_success"));
      void queryClient.invalidateQueries({ queryKey: createQueryKey("schedules") });
      void queryClient.invalidateQueries({ queryKey: createQueryKey("assignment_history") });
    },
    onError: (error: Error) => {
      if (handlePossibleComplianceConflict(error)) return;
      toast.error(error.message, t("shifts.messages.update_roster_error"));
    }
  });

  const bulkRosterMutation = useMutation<unknown, Error, { date: string; shift_id: string | null }[]>({
    mutationFn: (items: { date: string; shift_id: string | null }[]) => {
      if (!selectedEmployeeId) return Promise.reject(new Error("No employee selected"));
      return bulkUpsertRoster(selectedEmployeeId, items);
    },
    onSuccess: () => {
      toast.success(t("shifts.messages.bulk_update_success"));
      onSuccess?.("bulkRoster");
      void queryClient.invalidateQueries({ queryKey: createQueryKey("schedules") });
      void queryClient.invalidateQueries({ queryKey: createQueryKey("assignment_history") });
    },
    onError: (error: Error) => {
      if (handlePossibleComplianceConflict(error)) return;
      toast.error(error.message, t("shifts.messages.bulk_update_error"));
    }
  });

  const deleteRosterMutation = useMutation<unknown, Error, string>({
    mutationFn: (date: string) => {
      if (!selectedEmployeeId) return Promise.reject(new Error("No employee selected"));
      return deleteRoster(selectedEmployeeId, date);
    },
    onSuccess: () => {
      toast.success(t("shifts.messages.remove_override_success"));
      void queryClient.invalidateQueries({ queryKey: createQueryKey("schedules") });
      void queryClient.invalidateQueries({ queryKey: createQueryKey("assignment_history") });
    },
    onError: (error: Error) => { toast.error(error.message, t("shifts.messages.remove_override_error")); }
  });

  const createShiftMutation = useMutation<Shift, Error, CreateShiftRequest>({
    mutationFn: (data: CreateShiftRequest) => createShift(data),
    onSuccess: (data) => {
      toast.success(t("shifts.messages.create_shift_success", { name: data.name }));
      onSuccess?.("createShift");
      void queryClient.invalidateQueries({ queryKey: createQueryKey("shifts") });
    },
    onError: (error: Error) => { toast.error(error.message, t("shifts.messages.create_shift_error")); }
  });

  const updateShiftMutation = useMutation<Shift, Error, { id: string, data: UpdateShiftRequest }>({
    mutationFn: ({ id, data }: { id: string, data: UpdateShiftRequest }) => updateShift(id, data),
    onSuccess: () => {
      toast.success(t("shifts.messages.update_shift_success"));
      onSuccess?.("updateShift");
      void queryClient.invalidateQueries({ queryKey: createQueryKey("shifts") });
      void queryClient.invalidateQueries({ queryKey: createQueryKey("schedules") });
      void queryClient.invalidateQueries({ queryKey: createQueryKey("assignment_history") });
    },
    onError: (error: Error) => { toast.error(error.message, t("shifts.messages.update_shift_error")); }
  });

  return {
    assignPatternMutation,
    rosterMutation,
    bulkRosterMutation,
    deleteRosterMutation,
    createShiftMutation,
    updateShiftMutation
  };
}
