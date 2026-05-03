import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useTranslation } from "react-i18next";

import type {
  AssignPatternRequest,
  EmployeeWorkingPattern,
  Shift,
  ShiftRoster,
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

interface UseShiftMutationsProps {
  onSuccess?: (type: string) => void;
  selectedEmployeeId: string | undefined;
  wsId: string;
}

export function useShiftMutations({ wsId, selectedEmployeeId, onSuccess }: UseShiftMutationsProps) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const toast = useToast();

  const assignPatternMutation = useMutation<EmployeeWorkingPattern, Error, AssignPatternRequest>({
    mutationFn: (data: AssignPatternRequest) => {
      if (!selectedEmployeeId) return Promise.reject(new Error("No employee selected"));
      return assignPattern(wsId, selectedEmployeeId, data);
    },
    onSuccess: () => {
      toast.success(t("shifts.messages.assign_success"));
      onSuccess?.("assignPattern");
      void queryClient.invalidateQueries({ queryKey: ["schedules"] });
      void queryClient.invalidateQueries({ queryKey: ["assignment_history"] });
    },
    onError: (error: Error) => { toast.error(error.message, t("shifts.messages.assign_error")); }
  });

  const rosterMutation = useMutation<ShiftRoster, Error, { date: string; shift_id: string | null; notes?: string }>({
    mutationFn: (data: { date: string; shift_id: string | null; notes?: string }) => {
      if (!selectedEmployeeId) return Promise.reject(new Error("No employee selected"));
      return upsertRoster(wsId, selectedEmployeeId, data);
    },
    onSuccess: () => {
      toast.success(t("shifts.messages.update_roster_success"));
      void queryClient.invalidateQueries({ queryKey: ["schedules"] });
      void queryClient.invalidateQueries({ queryKey: ["assignment_history"] });
    },
    onError: (error: Error) => { toast.error(error.message, t("shifts.messages.update_roster_error")); }
  });

  const bulkRosterMutation = useMutation<unknown, Error, { date: string; shift_id: string | null }[]>({
    mutationFn: (items: { date: string; shift_id: string | null }[]) => {
      if (!selectedEmployeeId) return Promise.reject(new Error("No employee selected"));
      return bulkUpsertRoster(wsId, selectedEmployeeId, items);
    },
    onSuccess: () => {
      toast.success(t("shifts.messages.bulk_update_success"));
      onSuccess?.("bulkRoster");
      void queryClient.invalidateQueries({ queryKey: ["schedules"] });
      void queryClient.invalidateQueries({ queryKey: ["assignment_history"] });
    },
    onError: (error: Error) => { toast.error(error.message, t("shifts.messages.bulk_update_error")); }
  });

  const deleteRosterMutation = useMutation<unknown, Error, string>({
    mutationFn: (date: string) => {
      if (!selectedEmployeeId) return Promise.reject(new Error("No employee selected"));
      return deleteRoster(wsId, selectedEmployeeId, date);
    },
    onSuccess: () => {
      toast.success(t("shifts.messages.remove_override_success"));
      void queryClient.invalidateQueries({ queryKey: ["schedules"] });
      void queryClient.invalidateQueries({ queryKey: ["assignment_history"] });
    },
    onError: (error: Error) => { toast.error(error.message, t("shifts.messages.remove_override_error")); }
  });

  const createShiftMutation = useMutation<Shift, Error, Partial<Shift>>({
    mutationFn: (data: Partial<Shift>) => createShift(wsId, data),
    onSuccess: (data) => {
      toast.success(t("shifts.messages.create_shift_success", { name: data.name }));
      onSuccess?.("createShift");
      void queryClient.invalidateQueries({ queryKey: ["shifts", wsId] });
    },
    onError: (error: Error) => { toast.error(error.message, t("shifts.messages.create_shift_error")); }
  });

  const updateShiftMutation = useMutation<Shift, Error, { id: string, data: Partial<Shift> }>({
    mutationFn: ({ id, data }: { id: string, data: Partial<Shift> }) => updateShift(wsId, id, data),
    onSuccess: () => {
      toast.success(t("shifts.messages.update_shift_success"));
      onSuccess?.("updateShift");
      void queryClient.invalidateQueries({ queryKey: ["shifts", wsId] });
      void queryClient.invalidateQueries({ queryKey: ["schedules"] });
      void queryClient.invalidateQueries({ queryKey: ["assignment_history"] });
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
