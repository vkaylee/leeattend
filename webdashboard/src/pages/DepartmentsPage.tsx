import React, { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Plus, Search, Edit2, Trash2, Building2, UserCircle, Users, LayoutGrid, CalendarDays, Calculator } from "lucide-react";
import { fetchDepartments, createDepartment, updateDepartment, deleteDepartment, getCurrentWorkspaceId, fetchUsers, fetchWorkingPatterns, fetchDepartmentPayrollConfigs, setDepartmentPayrollConfig } from "../lib/api";
import { Combobox } from "@/components/ui/Combobox";
import Modal from "@/components/ui/Modal";
import TemporalConfigTable from "@/components/common/TemporalConfigTable";

export default function DepartmentsPage() {
  const { t } = useTranslation();
  const wsId = getCurrentWorkspaceId() || "";
  const queryClient = useQueryClient();
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [isConfigModalOpen, setIsConfigModalOpen] = useState(false);
  const [editingDept, setEditingDept] = useState<any>(null);
  const [selectedManagerId, setSelectedManagerId] = useState<string>("");
  const [selectedPatternId, setSelectedPatternId] = useState<string>("");

  const { data: departments = [], isLoading } = useQuery({
    queryKey: ["departments", wsId],
    queryFn: () => fetchDepartments(wsId),
    enabled: !!wsId,
  });

  const { data: deptConfigs = [], isLoading: isConfigsLoading } = useQuery({
    queryKey: ["dept-payroll-configs", editingDept?.id],
    queryFn: () => fetchDepartmentPayrollConfigs(wsId, editingDept.id),
    enabled: isConfigModalOpen && !!editingDept?.id,
  });

  const saveConfigMutation = useMutation({
    mutationFn: (data: { std_working_days: number; hours_per_day: number; effective_date: string }) => 
      setDepartmentPayrollConfig(wsId, editingDept.id, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["dept-payroll-configs", editingDept?.id] });
    },
  });

  const { data: users = [] } = useQuery({
    queryKey: ["users"],
    queryFn: fetchUsers,
  });

  const { data: patterns = [] } = useQuery({
    queryKey: ["working-patterns", wsId],
    queryFn: () => fetchWorkingPatterns(wsId),
    enabled: !!wsId,
  });

  const createMutation = useMutation({
    mutationFn: (data: any) => createDepartment(wsId, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["departments", wsId] });
      setIsModalOpen(false);
    },
  });

  const updateMutation = useMutation({
    mutationFn: (data: any) => updateDepartment(wsId, editingDept.id, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["departments", wsId] });
      setIsModalOpen(false);
      setEditingDept(null);
    },
  });

  const deleteMutation = useMutation({
    mutationFn: (id: string) => deleteDepartment(wsId, id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["departments", wsId] });
    },
  });

  const handleSubmit = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const formData = new FormData(e.currentTarget);
    const data = {
      name: formData.get("name") as string,
      description: formData.get("description") as string,
      manager_id: selectedManagerId || null,
      default_working_pattern_id: selectedPatternId || null,
    };

    if (editingDept) {
      updateMutation.mutate(data);
    } else {
      createMutation.mutate(data);
    }
  };

  useEffect(() => {
    if (editingDept) {
      setSelectedManagerId(editingDept.manager_id || "");
      setSelectedPatternId(editingDept.default_working_pattern_id || "");
    } else {
      setSelectedManagerId("");
      setSelectedPatternId("");
    }
  }, [editingDept]);

  if (!wsId) return <div className="p-8 text-center">{t("common.select_workspace")}</div>;

  return (
    <div className="space-y-8 animate-in fade-in duration-500">
      <div className="flex justify-between items-center px-2">
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-2xl bg-primary-50 flex items-center justify-center text-primary-600 shadow-sm border border-primary-100/50">
            <Building2 size={20} />
          </div>
          <div>
            <h3 className="text-lg font-bold text-slate-800 tracking-tight">{t("departments.title")}</h3>
            <p className="text-xs text-slate-500 font-medium">{t("departments.subtitle")}</p>
          </div>
        </div>
        <button 
          onClick={() => { setEditingDept(null); setIsModalOpen(true); }}
          data-testid="add-dept-btn"
          className="flex items-center gap-2 bg-primary-600 text-white px-6 py-2.5 rounded-2xl hover:bg-primary-700 transition-all shadow-lg shadow-primary-100 font-black text-[10px] uppercase tracking-widest active:scale-95"
        >
          <Plus size={14} />
          {t("departments.add_btn")}
        </button>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {isLoading ? (
          Array.from({ length: 3 }).map((_, i) => (
            <div key={i} className="bg-white border border-slate-100 p-8 rounded-[32px] shadow-sm animate-pulse">
              <div className="w-14 h-14 bg-slate-100 rounded-2xl mb-6" />
              <div className="h-6 bg-slate-100 rounded-lg w-2/3 mb-4" />
              <div className="h-4 bg-slate-50 rounded-lg w-full mb-8" />
              <div className="h-12 bg-slate-50 rounded-xl w-full" />
            </div>
          ))
        ) : departments.length === 0 ? (
          <div className="col-span-full py-24 text-center border-2 border-dashed border-slate-100 rounded-[40px] bg-slate-50/30 flex flex-col items-center">
            <div className="w-20 h-20 bg-white rounded-full flex items-center justify-center shadow-sm mb-4">
              <Building2 size={32} className="text-slate-300" />
            </div>
            <h3 className="text-lg font-bold text-slate-900">{t("departments.messages.no_departments")}</h3>
            <p className="text-slate-400 text-sm max-w-xs mt-1">{t("departments.messages.empty_desc", { defaultValue: "Bắt đầu bằng cách tạo phòng ban đầu tiên cho tổ chức của bạn." })}</p>
          </div>
        ) : departments.map((dept) => (
          <div key={dept.id} className="bg-white border border-slate-100 p-8 rounded-[32px] shadow-sm hover:shadow-xl hover:shadow-primary-100/20 hover:scale-[1.02] transition-all group relative overflow-hidden">
            <div className="absolute top-0 right-0 p-6 flex gap-2 translate-y-2 opacity-0 group-hover:translate-y-0 group-hover:opacity-100 transition-all">
              <button 
                onClick={() => { setEditingDept(dept); setIsConfigModalOpen(true); }}
                className="p-2.5 bg-white text-slate-400 hover:text-emerald-600 rounded-xl transition-all shadow-md border border-slate-100"
                title={t("payroll.settings.title")}
              >
                <Calculator className="w-4 h-4" />
              </button>
              <button 
                onClick={() => { setEditingDept(dept); setIsModalOpen(true); }}
                className="p-2.5 bg-white text-slate-400 hover:text-primary-600 rounded-xl transition-all shadow-md border border-slate-100"
                title={t("common.edit")}
              >
                <Edit2 className="w-4 h-4" />
              </button>
              <button 
                onClick={() => { if(confirm(t("departments.messages.confirm_delete"))) deleteMutation.mutate(dept.id); }}
                className="p-2.5 bg-white text-slate-400 hover:text-rose-600 rounded-xl transition-all shadow-md border border-slate-100"
                title={t("common.delete")}
              >
                <Trash2 className="w-4 h-4" />
              </button>
            </div>

            <div className="bg-primary-50 w-14 h-14 rounded-2xl flex items-center justify-center text-primary-600 mb-6 shadow-inner border border-primary-100/50">
              <Building2 className="w-7 h-7" />
            </div>
            
            <h2 className="text-xl font-black text-slate-900 tracking-tight mb-2 uppercase">{dept.name}</h2>
            <p className="text-slate-500 text-xs mb-8 line-clamp-2 min-h-[2.5rem] font-bold leading-relaxed opacity-70 italic">{dept.description || t("common.no_description")}</p>
            
            <div className="pt-6 border-t border-slate-50 flex flex-col gap-5">
              <div className="flex items-center gap-4">
                <div className="w-9 h-9 rounded-xl bg-slate-50 border border-slate-100 flex items-center justify-center text-slate-400 shadow-inner group-hover:text-primary-500 group-hover:bg-primary-50 group-hover:border-primary-100 transition-all">
                  <UserCircle size={18} />
                </div>
                <div>
                  <span className="block text-slate-400 font-black uppercase tracking-widest text-[9px] mb-0.5">{t("departments.manager_label")}</span>
                  <span className="font-black text-slate-800 text-xs uppercase tracking-tight">
                    {users.find(u => u.id === dept.manager_id)?.full_name || t("departments.not_assigned")}
                  </span>
                </div>
              </div>

              <div className="flex items-center gap-4">
                <div className="w-9 h-9 rounded-xl bg-slate-50 border border-slate-100 flex items-center justify-center text-slate-400 shadow-inner group-hover:text-primary-500 group-hover:bg-primary-50 group-hover:border-primary-100 transition-all">
                  <CalendarDays size={18} />
                </div>
                <div>
                  <span className="block text-slate-400 font-black uppercase tracking-widest text-[9px] mb-0.5">{t("departments.default_pattern_label")}</span>
                  <span className="font-black text-slate-800 text-xs uppercase tracking-tight">
                    {patterns.find(p => p.id === dept.default_working_pattern_id)?.name || t("departments.not_assigned")}
                  </span>
                </div>
              </div>
            </div>
          </div>
        ))}
      </div>

      {/* Dept Config Modal */}
      <Modal
        isOpen={isConfigModalOpen && !!editingDept}
        onClose={() => setIsConfigModalOpen(false)}
        title={t("payroll.settings.dept_title")}
        subtitle={editingDept?.name}
        icon={<Calculator size={24} />}
        maxWidth="2xl"
      >
        <div className="space-y-8 pt-4">
          <TemporalConfigTable 
            title={t("payroll.settings.std_days_history")}
            records={deptConfigs}
            isLoading={isConfigsLoading}
            onSave={async (data) => {
              await saveConfigMutation.mutateAsync(data);
            }}
          />
          <div className="pt-6 border-t border-slate-50 flex justify-end">
            <button 
              onClick={() => setIsConfigModalOpen(false)}
              className="px-10 py-3.5 bg-slate-900 text-white rounded-2xl font-black text-xs uppercase tracking-widest hover:bg-black transition-all shadow-xl shadow-slate-200 active:scale-95"
            >
              {t("common.close")}
            </button>
          </div>
        </div>
      </Modal>

      <Modal
        isOpen={isModalOpen}
        onClose={() => setIsModalOpen(false)}
        title={editingDept ? t("departments.modal.edit_title") : t("departments.modal.add_title")}
        subtitle={t("departments.modal.subtitle")}
        icon={<LayoutGrid size={24} />}
        maxWidth="lg"
      >
        <form onSubmit={handleSubmit} className="space-y-8 pt-4">
          <div className="space-y-2">
            <label htmlFor="dept-name" className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("departments.form.name_label")}</label>
            <input 
              id="dept-name"
              name="name" 
              required 
              defaultValue={editingDept?.name} 
              className="w-full px-5 py-4 bg-slate-50 border border-slate-100 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all font-black text-sm uppercase tracking-tight" 
              placeholder={t("departments.form.name_placeholder")}
            />
          </div>
          
          <div className="space-y-2">
            <label htmlFor="dept-desc" className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("common.description")}</label>
            <textarea 
              id="dept-desc"
              name="description" 
              defaultValue={editingDept?.description} 
              className="w-full px-5 py-4 bg-slate-50 border border-slate-100 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all font-medium text-sm h-28 resize-none" 
              placeholder={t("departments.form.desc_placeholder")}
            />
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="space-y-2">
              <label htmlFor="dept-manager" className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("departments.manager_label")}</label>
              <Combobox
                id="dept-manager"
                options={users.map(u => ({
                  value: u.id,
                  label: u.full_name || u.email,
                  icon: <UserCircle className="w-4 h-4" />
                }))}
                value={selectedManagerId}
                onChange={setSelectedManagerId}
                placeholder={t("departments.form.manager_placeholder")}
                className="bg-slate-50 border-slate-100"
              />
            </div>

            <div className="space-y-2">
              <label htmlFor="dept-pattern" className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("departments.default_pattern_label")}</label>
              <Combobox
                id="dept-pattern"
                data-testid="dept-pattern"
                options={patterns.map(p => ({
                  value: p.id,
                  label: p.name,
                  icon: <CalendarDays className="w-4 h-4" />
                }))}
                value={selectedPatternId}
                onChange={setSelectedPatternId}
                placeholder={t("departments.form.pattern_placeholder")}
                className="bg-slate-50 border-slate-100"
              />
            </div>
          </div>

          <div className="pt-6 grid grid-cols-2 gap-5">
            <button 
              type="button" 
              onClick={() => setIsModalOpen(false)} 
              className="px-8 py-4 border border-slate-100 rounded-2xl font-black text-xs uppercase tracking-widest text-slate-500 hover:bg-slate-50 transition-all"
            >
              {t("common.cancel")}
            </button>
            <button 
              type="submit" 
              data-testid="save-dept-btn"
              className="px-8 py-4 bg-slate-900 text-white rounded-2xl font-black text-xs uppercase tracking-widest hover:bg-black transition-all shadow-xl shadow-slate-200 active:scale-95"
            >
              {editingDept ? t("common.update") : t("departments.modal.submit_btn")}
            </button>
          </div>
        </form>
      </Modal>
    </div>
  );
}
