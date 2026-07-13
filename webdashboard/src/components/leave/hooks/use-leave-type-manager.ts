import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import { useToast } from "@/components/ui/toast";
import { ApiError } from "@/lib/api/errors";
import { fetchLeaveTypes, updateLeaveTypeConfig, createLeaveType, deleteLeaveType, restoreLeaveType } from "@/lib/api/leaves";
import { LeaveType, LeaveTypeConfig } from "@/lib/api/types";

interface RetiredLeaveTypeHint {
  leave_type_id: string;
  name: string;
}

interface UseLeaveTypeManagerProps {
  wsId: string;
}

export function useLeaveTypeManager({ wsId }: UseLeaveTypeManagerProps) {
  const { t } = useTranslation();
  const toast = useToast();
  const queryClient = useQueryClient();

  const [editingType, setEditingType] = useState<LeaveType | null>(null);
  const [isAddingType, setIsAddingType] = useState(false);
  const [deletingType, setDeletingType] = useState<LeaveType | null>(null);
  const [restoringType, setRestoringType] = useState<RetiredLeaveTypeHint | null>(null);

  const { data: leaveTypes = [], isLoading: isTypesLoading } = useQuery({
    queryKey: ["leave-types", wsId],
    queryFn: () => fetchLeaveTypes(wsId),
  });

  const createTypeMutation = useMutation({
    mutationFn: (data: { code: string; name: string; is_paid: boolean }) => createLeaveType(wsId, data),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["leave-types"] });
      toast.success(t("settings.leave.type_manager.create_success", "Leave type created successfully"));
      setIsAddingType(false);
    },
    onError: (error: unknown) => {
      // Backend returns 409 Conflict for a duplicate leave-type code (unique
      // constraint on workspace_id + code). When the conflicting code belongs to
      // a RETIRED (soft-deleted) leave type, the error carries a structured
      // `retry` hint so the UI can offer to restore it instead of creating new.
      if (error instanceof ApiError && error.isConflict) {
        const retry = error.errorBody?.error?.retry as
          | { reason: string; leave_type_id: string; name: string }
          | undefined;
        if (retry?.reason === "retired_leave_type" && retry.leave_type_id) {
          setRestoringType({ leave_type_id: retry.leave_type_id, name: retry.name });
          return;
        }
        toast.error(
          t(
            "settings.leave.type_manager.create_error_duplicate_code",
            "A leave type with this code already exists",
          ),
        );
        return;
      }
      toast.error(error instanceof Error ? error.message : "Failed to create leave type");
    },
  });

  const updateTypeMutation = useMutation({
    mutationFn: (data: { typeId: string; config: LeaveTypeConfig }) =>
      updateLeaveTypeConfig(wsId, data.typeId, data.config),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["leave-types"] });
      toast.success("Leave type updated successfully");
      setEditingType(null);
    },
    onError: () => {
      toast.error("Failed to update leave type");
    },
  });

  const deleteTypeMutation = useMutation({
    mutationFn: (typeId: string) => deleteLeaveType(wsId, typeId),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["leave-types"] });
      toast.success("Leave type deleted successfully");
      setDeletingType(null);
    },
    onError: (error: Error) => {
      toast.error(error.message);
      setDeletingType(null);
    },
  });

  const restoreTypeMutation = useMutation({
    mutationFn: (typeId: string) => restoreLeaveType(wsId, typeId),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["leave-types"] });
      toast.success(
        t("settings.leave.type_manager.restore_success", "Leave type restored successfully"),
      );
      // Restore replaces the "create new" intent, so close both dialogs and
      // return the user to the clean list view.
      setRestoringType(null);
      setIsAddingType(false);
    },
    onError: (error: Error) => {
      toast.error(error.message);
      setRestoringType(null);
    },
  });

  return {
    editingType,
    isAddingType,
    deletingType,
    restoringType,
    leaveTypes,
    isTypesLoading,
    setEditingType,
    setIsAddingType,
    setDeletingType,
    setRestoringType,
    createLeaveType: (data: { code: string; name: string; is_paid: boolean }) => { createTypeMutation.mutate(data); },
    updateLeaveType: (typeId: string, config: LeaveTypeConfig) => { updateTypeMutation.mutate({ typeId, config }); },
    deleteLeaveType: (typeId: string) => { deleteTypeMutation.mutate(typeId); },
    restoreLeaveType: (typeId: string) => { restoreTypeMutation.mutate(typeId); },
    isCreating: createTypeMutation.isPending,
    isUpdating: updateTypeMutation.isPending,
    isDeleting: deleteTypeMutation.isPending,
    isRestoring: restoreTypeMutation.isPending,
  };
}
