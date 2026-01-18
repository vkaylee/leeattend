import React, { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Plus, Search, Edit2, Trash2, Building2, UserCircle, Users, LayoutGrid, CalendarDays } from "lucide-react";
import { fetchDepartments, createDepartment, updateDepartment, deleteDepartment, getCurrentWorkspaceId, fetchUsers, fetchWorkingPatterns } from "../lib/api";
import { Combobox } from "@/components/ui/Combobox";
import Modal from "@/components/ui/Modal";

export default function DepartmentsPage() {
  const { t } = useTranslation();
  const wsId = getCurrentWorkspaceId() || "";
  const queryClient = useQueryClient();
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [editingDept, setEditingDept] = useState<any>(null);
  const [selectedManagerId, setSelectedManagerId] = useState<string>("");
  const [selectedPatternId, setSelectedPatternId] = useState<string>("");

  const { data: departments = [], isLoading } = useQuery({
    queryKey: ["departments", wsId],
    queryFn: () => fetchDepartments(wsId),
    enabled: !!wsId,
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
    const timer = setTimeout(() => {
      if (editingDept) {
        setSelectedManagerId(editingDept.manager_id || "");
        setSelectedPatternId(editingDept.default_working_pattern_id || "");
      } else {
        setSelectedManagerId("");
        setSelectedPatternId("");
      }
    }, 0);
    return () => clearTimeout(timer);
  }, [editingDept]);

  if (!wsId) return <div className="p-8 text-center">{t("common.select_workspace")}</div>;

  return (
    <div className="p-8 space-y-8 animate-enter">
      <div className="flex justify-between items-center">
        <div>
          <h1 className="text-3xl font-extrabold tracking-tight text-slate-900">{t("departments.title")}</h1>
          <p className="text-slate-500 mt-1">{t("departments.subtitle")}</p>
        </div>
        <button 
          onClick={() => { setEditingDept(null); setIsModalOpen(true); }}
          data-testid="add-dept-btn"
          className="flex items-center gap-2 bg-slate-900 text-white px-6 py-3 rounded-2xl hover:bg-slate-800 transition-all shadow-xl shadow-slate-200 font-bold"
        >
          <Plus className="w-5 h-5" />
          {t("departments.add_btn")}
        </button>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {isLoading ? (
          <div className="col-span-full py-20 text-center text-slate-400 font-medium italic">{t("common.loading")}</div>
        ) : departments.length === 0 ? (
          <div className="col-span-full py-20 text-center text-slate-400 font-medium border-2 border-dashed border-slate-100 rounded-3xl bg-slate-50/30">
            {t("departments.messages.no_departments")}
          </div>
        ) : departments.map((dept) => (
          <div key={dept.id} className="bg-white border border-slate-50 p-8 rounded-[2.5rem] shadow-sm hover:shadow-xl hover:scale-[1.02] transition-all group relative overflow-hidden">
            <div className="absolute top-0 right-0 p-6 flex gap-2 translate-y-2 opacity-0 group-hover:translate-y-0 group-hover:opacity-100 transition-all">
              <button 
                onClick={() => { setEditingDept(dept); setIsModalOpen(true); }}
                className="p-2.5 bg-slate-50 text-slate-400 hover:text-primary-600 hover:bg-primary-50 rounded-xl transition-all shadow-sm"
                title={t("common.edit")}
              >
                <Edit2 className="w-4 h-4" />
              </button>
              <button 
                onClick={() => { if(confirm(t("departments.messages.confirm_delete"))) deleteMutation.mutate(dept.id); }}
                className="p-2.5 bg-slate-50 text-slate-400 hover:text-rose-600 hover:bg-rose-50 rounded-xl transition-all shadow-sm"
                title={t("common.delete")}
              >
                <Trash2 className="w-4 h-4" />
              </button>
            </div>

            <div className="bg-primary-50 w-14 h-14 rounded-2xl flex items-center justify-center text-primary-600 mb-6 shadow-inner">
              <Building2 className="w-7 h-7" />
            </div>
            
            <h2 className="text-2xl font-black text-slate-900 tracking-tight mb-2 uppercase">{dept.name}</h2>
            <p className="text-slate-500 text-sm mb-8 line-clamp-2 min-h-[2.5rem] font-medium leading-relaxed">{dept.description || t("common.no_description")}</p>
            
            <div className="pt-6 border-t border-slate-50 flex flex-col gap-4">
              <div className="flex items-center gap-3">
                <div className="w-8 h-8 rounded-lg bg-slate-100 flex items-center justify-center text-slate-400">
                  <UserCircle className="w-4 h-4" />
                </div>
                <div className="text-xs">
                  <span className="block text-slate-400 font-bold uppercase tracking-widest text-[9px]">{t("departments.manager_label")}</span>
                  <span className="font-bold text-slate-700">
                    {users.find(u => u.id === dept.manager_id)?.full_name || t("departments.not_assigned")}
                  </span>
                </div>
              </div>

              <div className="flex items-center gap-3">
                <div className="w-8 h-8 rounded-lg bg-slate-100 flex items-center justify-center text-slate-400">
                  <CalendarDays className="w-4 h-4" />
                </div>
                <div className="text-xs">
                  <span className="block text-slate-400 font-bold uppercase tracking-widest text-[9px]">{t("departments.default_pattern_label")}</span>
                  <span className="font-bold text-slate-700">
                    {patterns.find(p => p.id === dept.default_working_pattern_id)?.name || t("departments.not_assigned")}
                  </span>
                </div>
              </div>
            </div>
          </div>
        ))}
      </div>

      <Modal
        isOpen={isModalOpen}
        onClose={() => setIsModalOpen(false)}
        title={editingDept ? t("departments.modal.edit_title") : t("departments.modal.add_title")}
        subtitle={t("departments.modal.subtitle")}
        icon={<LayoutGrid className="w-6 h-6" />}
      >
        <form onSubmit={handleSubmit} className="space-y-6">
          <div className="space-y-2">
            <label htmlFor="dept-name" className="text-xs font-black text-slate-400 uppercase tracking-widest px-1">{t("departments.form.name_label")}</label>
            <input 
              id="dept-name"
              name="name" 
              required 
              defaultValue={editingDept?.name} 
              className="w-full px-5 py-4 bg-slate-50 border border-slate-100 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:border-primary-300 outline-none transition-all font-bold" 
              placeholder={t("departments.form.name_placeholder")}
            />
          </div>
          
          <div className="space-y-2">
            <label htmlFor="dept-desc" className="text-xs font-black text-slate-400 uppercase tracking-widest px-1">{t("common.description")}</label>
            <textarea 
              id="dept-desc"
              name="description" 
              defaultValue={editingDept?.description} 
              className="w-full px-5 py-4 bg-slate-50 border border-slate-100 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:border-primary-300 outline-none transition-all font-medium h-24 resize-none" 
              placeholder={t("departments.form.desc_placeholder")}
            />
          </div>

          <div className="space-y-2">
            <label htmlFor="dept-manager" className="text-xs font-black text-slate-400 uppercase tracking-widest px-1">{t("departments.manager_label")}</label>
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
            <label htmlFor="dept-pattern" className="text-xs font-black text-slate-400 uppercase tracking-widest px-1">{t("departments.default_pattern_label")}</label>
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

          <div className="pt-6 grid grid-cols-2 gap-5">
            <button 
              type="button" 
              onClick={() => setIsModalOpen(false)} 
              className="px-8 py-4 border border-slate-100 rounded-2xl font-bold text-slate-500 hover:bg-slate-50 transition-all"
            >
              {t("common.cancel")}
            </button>
            <button 
              type="submit" 
              data-testid="save-dept-btn"
              className="px-8 py-4 bg-slate-900 text-white rounded-2xl font-black hover:bg-slate-800 transition-all shadow-xl shadow-slate-200 hover:scale-[1.02] active:scale-95"
            >
              {editingDept ? t("common.update") : t("departments.modal.submit_btn")}
            </button>
          </div>
        </form>
      </Modal>
    </div>
  );
}
