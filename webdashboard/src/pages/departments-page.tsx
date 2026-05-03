import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Plus, Pencil, Trash2, Building2, UserCircle, Users, LayoutGrid, CalendarDays, Calculator } from "lucide-react";
import React, { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";

import TemporalConfigTable from "@/components/common/temporal-config-table";
import Button from "@/components/ui/button";
import { Combobox } from "@/components/ui/combobox";
import EmptyState from "@/components/ui/empty-state";
import FormField, { LABEL_BASE } from "@/components/ui/form-field";
import Modal from "@/components/ui/modal";
import PageHeader from "@/components/ui/page-header";
import { useToast } from "@/components/ui/toast";
import { useConfirm } from "@/hooks/use-confirm";

import { fetchDepartments, createDepartment, updateDepartment, deleteDepartment, getCurrentWorkspaceId, fetchUsers, fetchWorkingPatterns, fetchDepartmentPayrollConfigs, setDepartmentPayrollConfig, Department } from "../lib/api";

export default function DepartmentsPage() {
  const { t } = useTranslation();
  const wsId = getCurrentWorkspaceId() ?? "";
  const queryClient = useQueryClient();
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [isConfigModalOpen, setIsConfigModalOpen] = useState(false);
  const [editingDept, setEditingDept] = useState<Department | null>(null);
  const [selectedManagerId, setSelectedManagerId] = useState<string>("");
 
  const [selectedPatternId, setSelectedPatternId] = useState<string>("");
  const { confirm, ConfirmationDialog } = useConfirm();

  const { data: departments = [], isLoading } = useQuery({ queryKey: ["departments", wsId], queryFn: () => fetchDepartments(wsId), enabled: !!wsId });
  const { data: deptConfigs = [], isLoading: isConfigsLoading } = useQuery({ queryKey: ["dept-payroll-configs", editingDept?.id], queryFn: () => fetchDepartmentPayrollConfigs(wsId, editingDept!.id), enabled: isConfigModalOpen && !!editingDept?.id });

  const saveConfigMutation = useMutation({
    mutationFn: (data: { std_working_days: number; hours_per_day: number; effective_date: string }) => setDepartmentPayrollConfig(wsId, editingDept!.id, data),
    onSuccess: () => { void queryClient.invalidateQueries({ queryKey: ["dept-payroll-configs", editingDept?.id] }); },
  });

  const { data: users = [] } = useQuery({ queryKey: ["users"], queryFn: fetchUsers });
  const { data: patterns = [] } = useQuery({ queryKey: ["working-patterns", wsId], queryFn: () => fetchWorkingPatterns(wsId), enabled: !!wsId });

  const createMutation = useMutation({
    mutationFn: (data: Record<string, unknown>) => createDepartment(wsId, data),
    onSuccess: () => { void queryClient.invalidateQueries({ queryKey: ["departments", wsId] }); setIsModalOpen(false); toast.success(t("common.success")); },
 
  });

  const updateMutation = useMutation({
    mutationFn: (data: Record<string, unknown>) => updateDepartment(wsId, editingDept!.id, data),
    onSuccess: () => { void queryClient.invalidateQueries({ queryKey: ["departments", wsId] }); setIsModalOpen(false); setEditingDept(null); toast.success(t("common.success")); },
 
  });

  const deleteMutation = useMutation({
    mutationFn: (id: string) => deleteDepartment(wsId, id),
    onSuccess: () => { void queryClient.invalidateQueries({ queryKey: ["departments", wsId] }); toast.success(t("common.success")); },
  });

  const toast = useToast();

  const handleSubmit = (e: React.SyntheticEvent<HTMLFormElement>) => {
    e.preventDefault(); const formData = new FormData(e.currentTarget);
    const data = { name: formData.get("name") as string, description: formData.get("description") as string, manager_id: selectedManagerId || null, default_working_pattern_id: selectedPatternId || null };
    if (editingDept) { updateMutation.mutate(data); } else { createMutation.mutate(data); }
  };

  useEffect(() => {
    if (editingDept) {
      // eslint-disable-next-line react-hooks/set-state-in-effect -- Derived state synced from props; intentional pattern
      setSelectedManagerId(editingDept.manager_id ?? "");
      setSelectedPatternId(editingDept.default_working_pattern_id ?? "");
    } else {
      setSelectedManagerId("");
      setSelectedPatternId("");
    }
  }, [editingDept]);

  if (!wsId) return <div className="p-12 text-center text-slate-500 dark:text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50">{t("common.select_workspace")}</div>;

  return (
    <div className="space-y-10 animate-in fade-in duration-700 pb-20">
      <PageHeader
        actions={
          <Button data-testid="add-dept-btn" icon={<Plus size={18} />} onClick={() => { setEditingDept(null); setIsModalOpen(true); }}>{t("departments.add_btn").toUpperCase()}</Button>
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
          <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 p-10 rounded-[2.5rem] shadow-sm hover:shadow-2xl hover:shadow-primary-100/10 hover:scale-[1.02] transition-all group relative overflow-hidden flex flex-col" key={dept.id}>
            <div className="absolute top-0 right-0 p-8 flex gap-3 translate-y-4 opacity-0 group-hover:translate-y-0 group-hover:opacity-100 transition-all duration-500">
              <button className="p-3 bg-white dark:bg-slate-900 text-slate-500 dark:text-slate-400 hover:text-emerald-600 dark:text-emerald-300 rounded-xl shadow-md border border-slate-100 dark:border-slate-800 transition-all active:scale-90" onClick={() => { setEditingDept(dept); setIsConfigModalOpen(true); }} title={t("payroll.settings.title")}><Calculator size={18} /></button>
              <button className="p-3 bg-white dark:bg-slate-900 text-slate-500 dark:text-slate-400 hover:text-primary-600 dark:text-primary-400 rounded-xl shadow-md border border-slate-100 dark:border-slate-800 transition-all active:scale-90" onClick={() => { setEditingDept(dept); setIsModalOpen(true); }} title={t("common.edit")}><Pencil size={18} /></button>
              <button className="p-3 bg-white dark:bg-slate-900 text-slate-500 dark:text-slate-400 hover:text-rose-600 dark:text-rose-300 rounded-xl shadow-md border border-slate-100 dark:border-slate-800 transition-all active:scale-90" onClick={async () => {
                const ok = await confirm({
                  title: t("common.warning", { defaultValue: "Warning" }),
                  message: t("departments.messages.confirm_delete"),
                  variant: "danger"
                });
                if(ok) deleteMutation.mutate(dept.id);
              }} title={t("common.delete")}><Trash2 size={18} /></button>
            </div>

            <div className="bg-primary-50 dark:bg-primary-950 w-16 h-16 rounded-[1.5rem] flex items-center justify-center text-primary-600 dark:text-primary-400 mb-8 shadow-inner border border-primary-100 dark:border-primary-800/50 group-hover:scale-110 group-hover:rotate-3 transition-all duration-500">
              <Building2 size={32} />
            </div>
            
            <h2 className="text-2xl font-bold text-slate-700 dark:text-slate-200 tracking-tight mb-3 uppercase leading-none">{dept.name}</h2>
            <p className="text-slate-500 dark:text-slate-400 text-xs mb-10 line-clamp-2 min-h-[3rem] font-bold uppercase tracking-tight opacity-60 leading-relaxed italic">{dept.description ?? t("common.no_description")}</p>
            
            <div className="mt-auto pt-8 border-t border-slate-50 dark:border-slate-800 flex flex-col gap-6">
              <div className="flex items-center gap-5 group/info">
                <div className="w-10 h-10 rounded-xl bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 flex items-center justify-center text-slate-300 dark:text-slate-600 shadow-inner group-hover/info:text-primary-600 dark:text-primary-400 group-hover/info:bg-primary-50 dark:bg-primary-950 group-hover/info:border-primary-100 dark:border-primary-800 transition-all"><Users size={20} /></div>
                <div>
                  <span className="block text-slate-500 dark:text-slate-400 font-bold uppercase tracking-[0.2em] text-[9px] mb-1">{t("departments.active_employees_label")}</span>
                  <span className="font-bold text-slate-800 dark:text-slate-100 text-xs uppercase tracking-tight">{dept.active_employee_count}</span>
                </div>
              </div>

              <div className="flex items-center gap-5 group/info">
                <div className="w-10 h-10 rounded-xl bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 flex items-center justify-center text-slate-300 dark:text-slate-600 shadow-inner group-hover/info:text-primary-600 dark:text-primary-400 group-hover/info:bg-primary-50 dark:bg-primary-950 group-hover/info:border-primary-100 dark:border-primary-800 transition-all"><UserCircle size={20} /></div>
                <div>
                  <span className="block text-slate-500 dark:text-slate-400 font-bold uppercase tracking-[0.2em] text-[9px] mb-1">{t("departments.manager_label")}</span>
                  <span className="font-bold text-slate-800 dark:text-slate-100 text-xs uppercase tracking-tight">{users.find(u => u.id === dept.manager_id)?.full_name ?? t("departments.not_assigned").toUpperCase()}</span>
                </div>
              </div>

              <div className="flex items-center gap-5 group/info">
                <div className="w-10 h-10 rounded-xl bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 flex items-center justify-center text-slate-300 dark:text-slate-600 shadow-inner group-hover/info:text-primary-600 dark:text-primary-400 group-hover/info:bg-primary-50 dark:bg-primary-950 group-hover/info:border-primary-100 dark:border-primary-800 transition-all"><CalendarDays size={20} /></div>
                <div>
                  <span className="block text-slate-500 dark:text-slate-400 font-bold uppercase tracking-[0.2em] text-[9px] mb-1">{t("departments.default_pattern_label")}</span>
                  <span className="font-bold text-slate-800 dark:text-slate-100 text-xs uppercase tracking-tight">{patterns.find(p => p.id === dept.default_working_pattern_id)?.name ?? t("departments.not_assigned").toUpperCase()}</span>
                </div>
              </div>
            </div>
          </div>
        ))}
      </div>

      <Modal icon={<Calculator size={24} />} isOpen={!!isConfigModalOpen && !!editingDept} maxWidth="3xl" onClose={() => { setIsConfigModalOpen(false); }} subtitle={editingDept?.name.toUpperCase()} title={t("payroll.settings.dept_title")}>
        <div className="space-y-10 pt-4 animate-in fade-in duration-500">
          <TemporalConfigTable isLoading={isConfigsLoading} onSave={async (data) => { await saveConfigMutation.mutateAsync(data); }} records={deptConfigs as any[]} title={t("payroll.settings.std_days_history")} />
          <div className="pt-8 border-t border-slate-50 dark:border-slate-800 flex justify-end"><Button onClick={() => { setIsConfigModalOpen(false); }} size="lg" variant="secondary">{t("common.close").toUpperCase()}</Button></div>
        </div>
      </Modal>

      <Modal icon={<LayoutGrid size={24} />} isOpen={isModalOpen} maxWidth="lg" onClose={() => { setIsModalOpen(false); }} subtitle={t("departments.modal.subtitle")} title={editingDept ? t("departments.modal.edit_title") : t("departments.modal.add_title")}>
        <form className="space-y-8 pt-4 animate-in fade-in duration-500" onSubmit={handleSubmit}>
          <FormField defaultValue={editingDept?.name} id="dept-name" label={t("departments.form.name_label")} name="name" placeholder={t("departments.form.name_placeholder")} required />
          <FormField defaultValue={editingDept?.description ?? undefined} id="dept-desc" label={t("common.description")} name="description" placeholder={t("departments.form.desc_placeholder")} rows={4} type="textarea" />
          <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
            <div className="space-y-3 px-1"><label className={LABEL_BASE} htmlFor="dept-manager">{t("departments.manager_label")}</label><Combobox className="bg-slate-50 dark:bg-slate-800 border-slate-100 dark:border-slate-800 h-14" id="dept-manager" onChange={setSelectedManagerId} options={users.map(u => ({ value: u.id, label: (u.full_name ?? u.email).toUpperCase(), icon: <UserCircle size={14} /> }))} placeholder={t("departments.form.manager_placeholder")} value={selectedManagerId} /></div>
            <div className="space-y-3 px-1"><label className={LABEL_BASE} htmlFor="dept-pattern">{t("departments.default_pattern_label")}</label><Combobox className="bg-slate-50 dark:bg-slate-800 border-slate-100 dark:border-slate-800 h-14" data-testid="dept-pattern" id="dept-pattern" onChange={setSelectedPatternId} options={patterns.map(p => ({ value: p.id, label: p.name.toUpperCase(), icon: <CalendarDays size={14} /> }))} placeholder={t("departments.form.pattern_placeholder")} value={selectedPatternId} /></div>
          </div>
          <div className="pt-6 grid grid-cols-2 gap-6 border-t border-slate-50 dark:border-slate-800">
            <Button onClick={() => { setIsModalOpen(false); }} size="lg" variant="outline">{t("common.cancel").toUpperCase()}</Button>
            <Button data-testid="save-dept-btn" size="lg" type="submit" variant="secondary">{editingDept ? t("common.update").toUpperCase() : t("departments.modal.submit_btn").toUpperCase()}</Button>
          </div>
        </form>
      </Modal>
      <ConfirmationDialog />
    </div>
  );
}