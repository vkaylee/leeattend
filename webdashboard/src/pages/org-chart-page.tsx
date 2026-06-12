import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { FolderTree, Search } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import { DepartmentConfigModal } from "@/components/features/departments/department-config-modal";
import { DepartmentFormModal } from "@/components/features/departments/department-form-modal";
import { DepartmentDetailDrawer } from "@/components/features/departments/department-detail-drawer";
import { DepartmentOrgChart } from "@/components/features/departments/department-org-chart";
import PageHeader from "@/components/ui/page-header";
import { useConfirm } from "@/hooks/use-confirm";

import { fetchDepartments, deleteDepartment, getCurrentWorkspaceId, fetchEmployees, Department } from "../lib/api";

export default function OrgChartPage() {
  const { t } = useTranslation();
  const wsId = getCurrentWorkspaceId() ?? "";
  const queryClient = useQueryClient();
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [isConfigModalOpen, setIsConfigModalOpen] = useState(false);
  const [editingDept, setEditingDept] = useState<Department | null>(null);

  // Detail Drawer state
  const [selectedDeptForDetail, setSelectedDeptForDetail] = useState<Department | null>(null);

  // Search state
  const [searchQuery, setSearchQuery] = useState("");

  const { confirm, ConfirmationDialog } = useConfirm();

  const { data: departments = [] } = useQuery({ 
    queryKey: ["departments", wsId], 
    queryFn: () => fetchDepartments(wsId), 
    enabled: !!wsId 
  });

  const { data: employeesData } = useQuery({ 
    queryKey: ["employees", wsId], 
    queryFn: () => fetchEmployees(wsId, 1, 1000), 
    enabled: !!wsId 
  });

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

  if (!wsId) return <div className="p-12 text-center text-slate-500 dark:text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50">{t("common.select_workspace")}</div>;

  return (
    <div className="w-full h-full flex flex-col bg-transparent animate-in fade-in duration-700">
      {/* Full-width Infinite Board Wrapper */}
      <div className="flex-1 relative">
        {/* Floating Search Glassmorphism Card */}
        <div className="absolute top-6 left-6 z-30 w-80 max-w-xs transition-all duration-300">
          <div className="relative flex items-center bg-white/80 dark:bg-slate-900/80 backdrop-blur-md border border-slate-200/50 dark:border-slate-800/50 p-2 pl-3 rounded-2xl shadow-xl shadow-slate-100/50 dark:shadow-black/40">
            <div className="w-8 h-8 rounded-xl bg-primary-50 dark:bg-primary-950/50 flex items-center justify-center text-primary-500 dark:text-primary-400 mr-3 shrink-0 shadow-inner">
              <FolderTree size={16} />
            </div>
            <div className="relative flex-1 min-w-0 pr-1">
              <input
                type="text"
                placeholder={t("departments.search_placeholder")}
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                className="w-full h-9 bg-transparent border-none text-xs font-bold tracking-tight text-slate-700 dark:text-slate-200 placeholder:text-slate-400/80 focus:outline-none"
              />
            </div>
          </div>
        </div>

        <div className="absolute inset-0">
          <DepartmentOrgChart
            departments={departments}
            employees={employees}
            searchQuery={searchQuery}
            wsId={wsId}
            onConfig={(d) => { setEditingDept(d); setIsConfigModalOpen(true); }}
            onEdit={(d) => { setEditingDept(d); setIsModalOpen(true); }}
            onDelete={(d) => { void handleDelete(d.id); }}
            onClick={(d) => setSelectedDeptForDetail(d)}
            t={t}
          />
        </div>
      </div>

      {/* Slide-over Detail Drawer */}
      <DepartmentDetailDrawer
        dept={selectedDeptForDetail}
        employees={employees}
        isOpen={Boolean(selectedDeptForDetail)}
        onClose={() => setSelectedDeptForDetail(null)}
        onConfig={(d) => { setEditingDept(d); setIsConfigModalOpen(true); }}
        onEdit={(d) => { setEditingDept(d); setIsModalOpen(true); }}
        onDelete={(d) => { setSelectedDeptForDetail(null); void handleDelete(d.id); }}
        t={t}
        wsId={wsId}
      />

      <DepartmentConfigModal
        editingDept={editingDept}
        isOpen={isConfigModalOpen}
        onClose={() => { setIsConfigModalOpen(false); }}
        t={t}
        wsId={wsId}
      />

      <DepartmentFormModal
        departments={departments}
        editingDept={editingDept}
        employees={employees}
        isOpen={isModalOpen}
        onClose={() => { setIsModalOpen(false); setEditingDept(null); }}
        t={t}
        wsId={wsId}
      />

      <ConfirmationDialog />
    </div>
  );
}
