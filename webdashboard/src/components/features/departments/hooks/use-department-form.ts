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

  const getParentId = (): string => {
    if (!editingDept) return "";
    try {
      const saved = localStorage.getItem(`leeattend_dept_parents_${wsId}`);
      if (saved) {
        const parentMap = JSON.parse(saved) as unknown;
        if (parentMap && typeof parentMap === "object") {
          return (parentMap as Record<string, string>)[editingDept.id] ?? "";
        }
      }
    } catch {
      // Ignore
    }
    return "";
  };

  const parentId = getParentId();
  const [selectedManagerId, setSelectedManagerId] = useState<string>(managerId);
  const [selectedParentId, setSelectedParentId] = useState<string>(parentId);
  const [prevDeptId, setPrevDeptId] = useState<string | null>(deptId);

  if (deptId !== prevDeptId) {
    setSelectedManagerId(managerId);
    setSelectedParentId(parentId);
    setPrevDeptId(deptId);
  }

  const createMutation = useMutation({
    mutationFn: (data: Parameters<typeof createDepartment>[1]) => createDepartment(wsId, data),
    onSuccess: (newDept) => { 
      if (selectedParentId && newDept.id) {
        try {
          const saved = localStorage.getItem(`leeattend_dept_parents_${wsId}`);
          const parentMap = (saved ? JSON.parse(saved) : {}) as Record<string, string>;
          parentMap[newDept.id] = selectedParentId;
          localStorage.setItem(`leeattend_dept_parents_${wsId}`, JSON.stringify(parentMap));
        } catch {
          // Ignore
        }
      }
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
      if (editingDept) {
        try {
          const saved = localStorage.getItem(`leeattend_dept_parents_${wsId}`);
          let parentMap = (saved ? JSON.parse(saved) : {}) as Record<string, string>;
          if (selectedParentId) {
            parentMap[editingDept.id] = selectedParentId;
          } else {
            const { [editingDept.id]: _, ...rest } = parentMap;
            parentMap = rest;
          }
          localStorage.setItem(`leeattend_dept_parents_${wsId}`, JSON.stringify(parentMap));
        } catch {
          // Ignore
        }
      }
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
      manager_id: selectedManagerId || null 
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
    selectedParentId,
    setSelectedParentId,
    handleSubmit,
    isLoading: createMutation.isPending || updateMutation.isPending
  };
}
