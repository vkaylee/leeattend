import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import { useConfirm } from "../../../hooks/use-confirm";
import { useAllEmployees } from "../../../hooks/use-employees";
import { useDepartments } from "../../../hooks/use-departments";
import { useOrganizationMutation } from "../../../hooks/common/use-organization-query";
import { deleteDepartment, Department } from "../../../lib/api";
import { departmentKeys } from "../../../lib/query-keys";


export function useDepartmentsPage() {
  const { t } = useTranslation();
  
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

  const { data: departments = [], isLoading } = useDepartments();

  const { employees } = useAllEmployees({
    permission: undefined,
  });


  const deleteMutation = useOrganizationMutation<string>({ keys: [departmentKeys.all()],
  mutationFn: (id) => deleteDepartment(id), });

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
        // eslint-disable-next-line @typescript-eslint/prefer-nullish-coalescing -- empty-string should map to undefined, ?? would preserve ""
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
