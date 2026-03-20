import React, { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Plus, Search, Pencil, Trash2, Building2, UserCircle, Users, LayoutGrid, CalendarDays, Calculator, Loader2, Calendar } from "lucide-react";
import { fetchDepartments, createDepartment, updateDepartment, deleteDepartment, getCurrentWorkspaceId, fetchUsers, fetchWorkingPatterns, fetchDepartmentPayrollConfigs, setDepartmentPayrollConfig } from "../lib/api";
import { Combobox } from "@/components/ui/Combobox";
import Modal from "@/components/ui/Modal";
import Button from "@/components/ui/Button";
import FormField, { LABEL_BASE } from "@/components/ui/FormField";
import TemporalConfigTable from "@/components/common/TemporalConfigTable";
import { useToast } from "@/components/ui/Toast";
import { cn } from "@/lib/utils";
import { useDateTimeFormatter } from "@/hooks/use-datetime";

export default function DepartmentsPage() {
  const { t } = useTranslation();
  const wsId = getCurrentWorkspaceId() || "";
  const queryClient = useQueryClient();
  const { formatDate } = useDateTimeFormatter();
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [isConfigModalOpen, setIsConfigModalOpen] = useState(false);
  const [editingDept, setEditingDept] = useState<any>(null);
  const [selectedManagerId, setSelectedManagerId] = useState<string>("");
  const [selectedPatternId, setSelectedPatternId] = useState<string>("");

  const { data: departments = [], isLoading } = useQuery({ queryKey: ["departments", wsId], queryFn: () => fetchDepartments(wsId), enabled: !!wsId });
  const { data: deptConfigs = [], isLoading: isConfigsLoading } = useQuery({ queryKey: ["dept-payroll-configs", editingDept?.id], queryFn: () => fetchDepartmentPayrollConfigs(wsId, editingDept.id), enabled: isConfigModalOpen && !!editingDept?.id });

  const saveConfigMutation = useMutation({
    mutationFn: (data: { std_working_days: number; hours_per_day: number; effective_date: string }) => setDepartmentPayrollConfig(wsId, editingDept.id, data),
    onSuccess: () => { queryClient.invalidateQueries({ queryKey: ["dept-payroll-configs", editingDept?.id] }); },
  });

  const { data: users = [] } = useQuery({ queryKey: ["users"], queryFn: fetchUsers });
  const { data: patterns = [] } = useQuery({ queryKey: ["working-patterns", wsId], queryFn: () => fetchWorkingPatterns(wsId), enabled: !!wsId });

  const createMutation = useMutation({
    mutationFn: (data: any) => createDepartment(wsId, data),
    onSuccess: () => { queryClient.invalidateQueries({ queryKey: ["departments", wsId] }); setIsModalOpen(false); toast.success(t("common.success")); },
  });

  const updateMutation = useMutation({
    mutationFn: (data: any) => updateDepartment(wsId, editingDept.id, data),
    onSuccess: () => { queryClient.invalidateQueries({ queryKey: ["departments", wsId] }); setIsModalOpen(false); setEditingDept(null); toast.success(t("common.success")); },
  });

  const deleteMutation = useMutation({
    mutationFn: (id: string) => deleteDepartment(wsId, id),
    onSuccess: () => { queryClient.invalidateQueries({ queryKey: ["departments", wsId] }); toast.success(t("common.success")); },
  });

  const toast = useToast();

  const handleSubmit = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault(); const formData = new FormData(e.currentTarget);
    const data = { name: formData.get("name") as string, description: formData.get("description") as string, manager_id: selectedManagerId || null, default_working_pattern_id: selectedPatternId || null };
    if (editingDept) { updateMutation.mutate(data); } else { createMutation.mutate(data); }
  };

  useEffect(() => {
    if (editingDept) {
      // eslint-disable-next-line react-hooks/set-state-in-effect
      setSelectedManagerId(editingDept.manager_id || "");
      setSelectedPatternId(editingDept.default_working_pattern_id || "");
    } else {
      setSelectedManagerId("");
      setSelectedPatternId("");
    }
  }, [editingDept]);

  if (!wsId) return <div className="p-12 text-center text-slate-500 dark:text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50">{t("common.select_workspace")}</div>;

  return (
    <div className="space-y-10 animate-in fade-in duration-700 pb-20">
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-8 px-2">
        <div className="flex items-center gap-4">
          <div className="w-12 h-12 rounded-[1.25rem] bg-primary-50 dark:bg-primary-950 flex items-center justify-center text-primary-600 dark:text-primary-400 shadow-sm border border-primary-100 dark:border-primary-800/50"><Building2 size={24} /></div>
          <div>
            <h3 data-testid="departments-title" className="text-2xl font-bold tracking-tight text-slate-700 dark:text-slate-200 uppercase leading-none">{t("departments.title")}</h3>
            <p className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest mt-1.5 opacity-70">{t("departments.subtitle")}</p>
          </div>
        </div>
        <Button data-testid="add-dept-btn" onClick={() => { setEditingDept(null); setIsModalOpen(true); }} icon={<Plus size={18} />}>{t("departments.add_btn").toUpperCase()}</Button>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8">
        {isLoading ? (
          Array.from({ length: 3 }).map((_, i) => (
            <div key={i} className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 p-10 rounded-[2.5rem] shadow-sm animate-pulse h-80" />
          ))
        ) : departments.length === 0 ? (
          <div className="col-span-full py-32 text-center border-2 border-dashed border-slate-100 dark:border-slate-800 rounded-[3rem] bg-slate-50 dark:bg-slate-800/30 flex flex-col items-center animate-in zoom-in-95 duration-500">
            <div className="w-24 h-24 bg-white dark:bg-slate-900 rounded-full flex items-center justify-center shadow-sm mb-6 border border-slate-100 dark:border-slate-800"><Building2 size={40} className="text-slate-200 dark:text-slate-700" /></div>
            <h3 className="text-xl font-bold text-slate-700 dark:text-slate-200 uppercase tracking-tight">{t("departments.messages.no_departments")}</h3>
            <p className="text-slate-500 dark:text-slate-400 text-sm font-bold uppercase tracking-widest mt-1 opacity-50 italic">{t("departments.messages.empty_desc")}</p>
          </div>
        ) : departments.map((dept) => (
          <div key={dept.id} className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 p-10 rounded-[2.5rem] shadow-sm hover:shadow-2xl hover:shadow-primary-100/10 hover:scale-[1.02] transition-all group relative overflow-hidden flex flex-col">
            <div className="absolute top-0 right-0 p-8 flex gap-3 translate-y-4 opacity-0 group-hover:translate-y-0 group-hover:opacity-100 transition-all duration-500">
              <button onClick={() => { setEditingDept(dept); setIsConfigModalOpen(true); }} className="p-3 bg-white dark:bg-slate-900 text-slate-500 dark:text-slate-400 hover:text-emerald-600 dark:text-emerald-300 rounded-xl shadow-md border border-slate-100 dark:border-slate-800 transition-all active:scale-90" title={t("payroll.settings.title")}><Calculator size={18} /></button>
              <button onClick={() => { setEditingDept(dept); setIsModalOpen(true); }} className="p-3 bg-white dark:bg-slate-900 text-slate-500 dark:text-slate-400 hover:text-primary-600 dark:text-primary-400 rounded-xl shadow-md border border-slate-100 dark:border-slate-800 transition-all active:scale-90" title={t("common.edit")}><Pencil size={18} /></button>
              <button onClick={() => { if(confirm(t("departments.messages.confirm_delete"))) deleteMutation.mutate(dept.id); }} className="p-3 bg-white dark:bg-slate-900 text-slate-500 dark:text-slate-400 hover:text-rose-600 dark:text-rose-300 rounded-xl shadow-md border border-slate-100 dark:border-slate-800 transition-all active:scale-90" title={t("common.delete")}><Trash2 size={18} /></button>
            </div>

            <div className="bg-primary-50 dark:bg-primary-950 w-16 h-16 rounded-[1.5rem] flex items-center justify-center text-primary-600 dark:text-primary-400 mb-8 shadow-inner border border-primary-100 dark:border-primary-800/50 group-hover:scale-110 group-hover:rotate-3 transition-all duration-500">
              <Building2 size={32} />
            </div>
            
            <h2 className="text-2xl font-bold text-slate-700 dark:text-slate-200 tracking-tight mb-3 uppercase leading-none">{dept.name}</h2>
            <p className="text-slate-500 dark:text-slate-400 text-xs mb-10 line-clamp-2 min-h-[3rem] font-bold uppercase tracking-tight opacity-60 leading-relaxed italic">{dept.description || t("common.no_description")}</p>
            
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
                  <span className="font-bold text-slate-800 dark:text-slate-100 text-xs uppercase tracking-tight">{users.find(u => u.id === dept.manager_id)?.full_name || t("departments.not_assigned").toUpperCase()}</span>
                </div>
              </div>

              <div className="flex items-center gap-5 group/info">
                <div className="w-10 h-10 rounded-xl bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 flex items-center justify-center text-slate-300 dark:text-slate-600 shadow-inner group-hover/info:text-primary-600 dark:text-primary-400 group-hover/info:bg-primary-50 dark:bg-primary-950 group-hover/info:border-primary-100 dark:border-primary-800 transition-all"><CalendarDays size={20} /></div>
                <div>
                  <span className="block text-slate-500 dark:text-slate-400 font-bold uppercase tracking-[0.2em] text-[9px] mb-1">{t("departments.default_pattern_label")}</span>
                  <span className="font-bold text-slate-800 dark:text-slate-100 text-xs uppercase tracking-tight">{patterns.find(p => p.id === dept.default_working_pattern_id)?.name || t("departments.not_assigned").toUpperCase()}</span>
                </div>
              </div>
            </div>
          </div>
        ))}
      </div>

      <Modal isOpen={isConfigModalOpen && !!editingDept} onClose={() => setIsConfigModalOpen(false)} title={t("payroll.settings.dept_title")} subtitle={editingDept?.name?.toUpperCase()} icon={<Calculator size={24} />} maxWidth="3xl">
        <div className="space-y-10 pt-4 animate-in fade-in duration-500">
          <TemporalConfigTable title={t("payroll.settings.std_days_history")} records={deptConfigs} isLoading={isConfigsLoading} onSave={async (data) => { await saveConfigMutation.mutateAsync(data); }} />
          <div className="pt-8 border-t border-slate-50 dark:border-slate-800 flex justify-end"><Button variant="secondary" size="lg" onClick={() => setIsConfigModalOpen(false)}>{t("common.close").toUpperCase()}</Button></div>
        </div>
      </Modal>

      <Modal isOpen={isModalOpen} onClose={() => setIsModalOpen(false)} title={editingDept ? t("departments.modal.edit_title") : t("departments.modal.add_title")} subtitle={t("departments.modal.subtitle")} icon={<LayoutGrid size={24} />} maxWidth="lg">
        <form onSubmit={handleSubmit} className="space-y-8 pt-4 animate-in fade-in duration-500">
          <FormField label={t("departments.form.name_label")} id="dept-name" name="name" required defaultValue={editingDept?.name} placeholder={t("departments.form.name_placeholder")} />
          <FormField label={t("common.description")} id="dept-desc" name="description" type="textarea" defaultValue={editingDept?.description} placeholder={t("departments.form.desc_placeholder")} rows={4} />
          <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
            <div className="space-y-3 px-1"><label htmlFor="dept-manager" className={LABEL_BASE}>{t("departments.manager_label")}</label><Combobox id="dept-manager" options={users.map(u => ({ value: u.id, label: (u.full_name || u.email).toUpperCase(), icon: <UserCircle size={14} /> }))} value={selectedManagerId} onChange={setSelectedManagerId} placeholder={t("departments.form.manager_placeholder")} className="bg-slate-50 dark:bg-slate-800 border-slate-100 dark:border-slate-800 h-14" /></div>
            <div className="space-y-3 px-1"><label htmlFor="dept-pattern" className={LABEL_BASE}>{t("departments.default_pattern_label")}</label><Combobox id="dept-pattern" data-testid="dept-pattern" options={patterns.map(p => ({ value: p.id, label: p.name.toUpperCase(), icon: <CalendarDays size={14} /> }))} value={selectedPatternId} onChange={setSelectedPatternId} placeholder={t("departments.form.pattern_placeholder")} className="bg-slate-50 dark:bg-slate-800 border-slate-100 dark:border-slate-800 h-14" /></div>
          </div>
          <div className="pt-6 grid grid-cols-2 gap-6 border-t border-slate-50 dark:border-slate-800">
            <Button variant="outline" size="lg" onClick={() => setIsModalOpen(false)}>{t("common.cancel").toUpperCase()}</Button>
            <Button variant="secondary" size="lg" type="submit" data-testid="save-dept-btn">{editingDept ? t("common.update").toUpperCase() : t("departments.modal.submit_btn").toUpperCase()}</Button>
          </div>
        </form>
      </Modal>
    </div>
  );
}