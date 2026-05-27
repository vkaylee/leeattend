import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Plus, Building2 } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import { DepartmentCard } from "@/components/features/departments/department-card";
import { DepartmentConfigModal } from "@/components/features/departments/department-config-modal";
import { DepartmentFormModal } from "@/components/features/departments/department-form-modal";
import Button from "@/components/ui/button";
import EmptyState from "@/components/ui/empty-state";
import PageHeader from "@/components/ui/page-header";

import { fetchDepartments, deleteDepartment, getCurrentWorkspaceId, fetchEmployees, fetchWorkingPatterns, Department } from "../lib/api";

export default function DepartmentsPage() {
  const { t } = useTranslation();
  const wsId = getCurrentWorkspaceId() ?? "";
  const queryClient = useQueryClient();
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [isConfigModalOpen, setIsConfigModalOpen] = useState(false);
  const [editingDept, setEditingDept] = useState<Department | null>(null);

  const { data: departments = [], isLoading } = useQuery({ queryKey: ["departments", wsId], queryFn: () => fetchDepartments(wsId), enabled: !!wsId });
  const { data: employeesData } = useQuery({ queryKey: ["employees", wsId], queryFn: () => fetchEmployees(wsId, 1, 1000), enabled: !!wsId });
  const employees = employeesData?.data ?? [];
  const { data: patterns = [] } = useQuery({ queryKey: ["working-patterns", wsId], queryFn: () => fetchWorkingPatterns(wsId), enabled: !!wsId });

  const deleteMutation = useMutation({
    mutationFn: (id: string) => deleteDepartment(wsId, id),
    onSuccess: () => { void queryClient.invalidateQueries({ queryKey: ["departments", wsId] }); },
  });

  if (!wsId) return <div className="p-12 text-center text-slate-500 dark:text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50">{t("common.select_workspace")}</div>;

  return (
    <div className="space-y-10 animate-in fade-in duration-700 pb-20">
      <PageHeader
        actions={
          <Button data-testid="add-dept-btn" icon={<Plus size={18} />} onClick={() => { setEditingDept(null); setIsModalOpen(true); }} variant="ghost">{t("departments.add_btn").toUpperCase()}</Button>
        }
        data-testid="departments-title"
        icon={<Building2 size={24} />}
        subtitle={t("departments.subtitle")}
        title={t("departments.title")}
      />

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8">
        {isLoading ? (
          Array.from({ length: 3 }).map((_, index) => (
            <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 p-10 rounded-[2.5rem] shadow-sm animate-pulse h-80" key={index} />
          ))
        ) : departments.length === 0 ? (
          <EmptyState
            description={t("departments.messages.empty_desc")}
            icon={<Building2 size={40} />}
            title={t("departments.messages.no_departments")}
          />
        ) : departments.map((dept) => (
          <DepartmentCard
            deleteMutationMutate={deleteMutation.mutate}
            dept={dept}
            employees={employees}
            key={dept.id}
            onConfig={(d) => { setEditingDept(d); setIsConfigModalOpen(true); }}
            onEdit={(d) => { setEditingDept(d); setIsModalOpen(true); }}
            patterns={patterns}
            t={t}
          />
        ))}
      </div>

      <DepartmentConfigModal
        editingDept={editingDept}
        isOpen={isConfigModalOpen}
        onClose={() => { setIsConfigModalOpen(false); }}
        t={t}
        wsId={wsId}
      />

      <DepartmentFormModal
        editingDept={editingDept}
        employees={employees}
        isOpen={isModalOpen}
        onClose={() => { setIsModalOpen(false); setEditingDept(null); }}
        patterns={patterns}
        t={t}
        wsId={wsId}
      />
    </div>
  );
}