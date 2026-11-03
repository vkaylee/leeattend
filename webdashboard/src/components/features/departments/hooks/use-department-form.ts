import type { TFunction } from "i18next";

import { useOrganizationMutation } from "@/hooks/common/use-workspace-query";
import { useState } from "react";

import type { Department } from "@/lib/api";

import { useToast } from "@/components/ui/toast";
import { createDepartment, updateDepartment } from "@/lib/api";
import { departmentKeys } from "@/lib/query-keys";
export function useDepartmentForm(wsId: string, editingDept: Department | null, onClose: () => void, t: TFunction) {
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
    } catch { /* localStorage unavailable or corrupt — ignore parent map */
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

  const createMutation = useOrganizationMutation<
    Parameters<typeof createDepartment>[1],
    Awaited<ReturnType<typeof createDepartment>>
  >({
    workspaceId: wsId,
    keys: [departmentKeys.all(wsId)],
    mutationFn: (workspaceId, data) => createDepartment(workspaceId, data),
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
      onClose();
      toast.success(t("common.success"));
    },
  });

  const updateMutation = useOrganizationMutation<
    Parameters<typeof updateDepartment>[2],
    Awaited<ReturnType<typeof updateDepartment>>
  >({
    workspaceId: wsId,
    keys: [departmentKeys.all(wsId)],
    mutationFn: (workspaceId, data) => {
      if (!editingDept) throw new Error("Editing department is required for update");
      return updateDepartment(workspaceId, editingDept.id, data);
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
