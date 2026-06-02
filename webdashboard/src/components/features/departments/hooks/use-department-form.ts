import type { TFunction } from "i18next";

import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";

import type { Department } from "@/lib/api";

import { useToast } from "@/components/ui/toast";
import { createDepartment, updateDepartment } from "@/lib/api";

export function useDepartmentForm(wsId: string, editingDept: Department | null, onClose: () => void, t: TFunction) {
  const queryClient = useQueryClient();
  const toast = useToast();

  const deptId = editingDept?.id ?? null;
  const managerId = editingDept?.manager_id ?? "";
  const patternId = editingDept?.default_working_pattern_id ?? "";
  
  const [selectedManagerId, setSelectedManagerId] = useState<string>(managerId);
  const [selectedPatternId, setSelectedPatternId] = useState<string>(patternId);
  const [prevDeptId, setPrevDeptId] = useState<string | null>(deptId);

  if (deptId !== prevDeptId) {
    setSelectedManagerId(managerId);
    setSelectedPatternId(patternId);
    setPrevDeptId(deptId);
  }

  const createMutation = useMutation({
    mutationFn: (data: Parameters<typeof createDepartment>[1]) => createDepartment(wsId, data),
    onSuccess: () => { 
      void queryClient.invalidateQueries({ queryKey: ["departments", wsId] }); 
      onClose(); 
      toast.success(t("common.success")); 
    },
  });

  const updateMutation = useMutation({
    mutationFn: (data: Parameters<typeof updateDepartment>[2]) => {
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
