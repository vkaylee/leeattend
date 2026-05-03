import type { TFunction } from "i18next";

import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";

import type { Department } from "@/lib/api";

import { useToast } from "@/components/ui/toast";
import { createDepartment, updateDepartment } from "@/lib/api";

export function useDepartmentForm(wsId: string, editingDept: Department | null, onClose: () => void, t: TFunction) {
  const queryClient = useQueryClient();
  const toast = useToast();

  const [selectedManagerId, setSelectedManagerId] = useState<string>(editingDept?.manager_id ?? "");
  const [selectedPatternId, setSelectedPatternId] = useState<string>(editingDept?.default_working_pattern_id ?? "");
  const [prevDeptId, setPrevDeptId] = useState<string | null>(editingDept?.id ?? null);

  if (editingDept?.id !== prevDeptId) {
    setSelectedManagerId(editingDept?.manager_id ?? "");
    setSelectedPatternId(editingDept?.default_working_pattern_id ?? "");
    setPrevDeptId(editingDept?.id ?? null);
  }

  const createMutation = useMutation({
    mutationFn: (data: Record<string, unknown>) => createDepartment(wsId, data),
    onSuccess: () => { 
      void queryClient.invalidateQueries({ queryKey: ["departments", wsId] }); 
      onClose(); 
      toast.success(t("common.success")); 
    },
  });

  const updateMutation = useMutation({
    mutationFn: (data: Record<string, unknown>) => {
      if (!editingDept) throw new Error("Editing department is required for update");
      return updateDepartment(wsId, editingDept.id, data);
    },
    onSuccess: () => { 
      void queryClient.invalidateQueries({ queryKey: ["departments", wsId] }); 
      onClose(); 
      toast.success(t("common.success")); 
    },
  });

  const handleSubmit = (e: React.SyntheticEvent<HTMLFormElement>) => {
    e.preventDefault(); 
    const formData = new FormData(e.currentTarget);
    const data = { 
      name: formData.get("name") as string, 
      description: formData.get("description") as string, 
      manager_id: selectedManagerId || null, 
      default_working_pattern_id: selectedPatternId || null 
    };
    if (editingDept) { 
      updateMutation.mutate(data); 
    } else { 
      createMutation.mutate(data); 
    }
  };

  return {
    selectedManagerId,
    setSelectedManagerId,
    selectedPatternId,
    setSelectedPatternId,
    handleSubmit,
    isLoading: createMutation.isPending || updateMutation.isPending
  };
}
