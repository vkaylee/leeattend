import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useState, useMemo } from "react";
import { useTranslation } from "react-i18next";

import { useConfirm } from "../../../hooks/use-confirm";
import { fetchDepartments, deleteDepartment, getCurrentWorkspaceId, fetchEmployees, Department } from "../../../lib/api";

export function useDepartmentsPage() {
  const { t } = useTranslation();
  const wsId = getCurrentWorkspaceId() ?? "";
  const queryClient = useQueryClient();
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [isConfigModalOpen, setIsConfigModalOpen] = useState(false);
  const [editingDept, setEditingDept] = useState<Department | null>(null);

  // Detail Drawer state
  const [selectedDeptForDetail, setSelectedDeptForDetail] = useState<Department | null>(null);

  // Search, Filter, and View Mode states
  const [viewMode, setViewMode] = useState<"grid" | "table">("table");
  const [searchQuery, setSearchQuery] = useState("");
  const [selectedManagerId, setSelectedManagerId] = useState("all");

  const { confirm, ConfirmationDialog } = useConfirm();

  const { data: departments = [], isLoading } = useQuery({ 
    queryKey: ["departments", wsId], 
    queryFn: () => fetchDepartments(wsId), 
    enabled: !!wsId 
  });

  const { data: employeesData } = useQuery({ 
    queryKey: ["employees", wsId], 
    queryFn: () => fetchEmployees(wsId, 1, 1000), 
    enabled: !!wsId 
  });

  // eslint-disable-next-line react-hooks/exhaustive-deps
  const employees = employeesData?.data ?? [];

  const deleteMutation = useMutation({
    mutationFn: (id: string) => deleteDepartment(wsId, id),
    onSuccess: () => { 
      void queryClient.invalidateQueries({ queryKey: ["departments", wsId] }); 
    },
  });

  const handleDelete = async (id: string) => {
    const ok = await confirm({
      title: t("common.warning", { defaultValue: "Warning" }),
      message: t("departments.messages.confirm_delete"),
      variant: "danger"
    });
    if (ok) deleteMutation.mutate(id);
  };

  // Compute unique managers options for combobox filter
  const managerOptions = useMemo(() => {
    const managerIds = new Set(departments.map(d => d.manager_id).filter(Boolean));
    const activeManagers = employees.filter(e => managerIds.has(e.id));
    
    return [
      { value: "all", label: t("departments.all_managers").toUpperCase() },
      ...activeManagers.map(e => ({
        value: e.id,
        label: e.full_name.toUpperCase(),
        // Keep ternary (not ??): empty-string department_name should map to undefined
        description: e.department_name ? e.department_name : undefined
      }))
    ];
  }, [departments, employees, t]);

  // Compute filtered departments
  const filteredDepartments = useMemo(() => {
    return departments.filter(dept => {
      const matchesSearch = 
        !searchQuery ||
        dept.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
        (dept.description?.toLowerCase().includes(searchQuery.toLowerCase()));
        
      const matchesManager = 
        selectedManagerId === "all" || 
        dept.manager_id === selectedManagerId;
        
      return matchesSearch && matchesManager;
    });
  }, [departments, searchQuery, selectedManagerId]);

  return {
    t,
    wsId,
    isModalOpen,
    setIsModalOpen,
    isConfigModalOpen,
    setIsConfigModalOpen,
    editingDept,
    setEditingDept,
    selectedDeptForDetail,
    setSelectedDeptForDetail,
    viewMode,
    setViewMode,
    searchQuery,
    setSearchQuery,
    selectedManagerId,
    setSelectedManagerId,
    ConfirmationDialog,
    departments,
    isLoading,
    employees,
    deleteMutation,
    handleDelete,
    managerOptions,
    filteredDepartments,
  };
}
