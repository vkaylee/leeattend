import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import { useToast } from "@/components/ui/toast";
import { ApiError } from "@/lib/api/errors";
import { fetchLeaveTypes, updateLeaveTypeConfig, createLeaveType, deleteLeaveType } from "@/lib/api/leaves";
import { LeaveType, LeaveTypeConfig } from "@/lib/api/types";

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
      // constraint on workspace_id + code). Surface the localized message
      // rather than the raw ApiError text.
      if (error instanceof ApiError && error.isConflict) {
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

  return {
    editingType,
    isAddingType,
    deletingType,
    leaveTypes,
    isTypesLoading,
    setEditingType,
    setIsAddingType,
    setDeletingType,
    createLeaveType: (data: { code: string; name: string; is_paid: boolean }) => { createTypeMutation.mutate(data); },
    updateLeaveType: (typeId: string, config: LeaveTypeConfig) => { updateTypeMutation.mutate({ typeId, config }); },
    deleteLeaveType: (typeId: string) => { deleteTypeMutation.mutate(typeId); },
    isCreating: createTypeMutation.isPending,
    isUpdating: updateTypeMutation.isPending,
    isDeleting: deleteTypeMutation.isPending,
  };
}
