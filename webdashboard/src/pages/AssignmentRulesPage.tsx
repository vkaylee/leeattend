import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { 
  Plus, 
  Trash2, 
  GripVertical,
  Settings2,
  Filter,
  Check,
  X,
  AlertCircle
} from "lucide-react";
import { 
  fetchAssignmentRules, 
  createAssignmentRule, 
  deleteAssignmentRule,
  updateAssignmentRule,
  fetchWorkingPatterns,
  fetchDepartments,
  getCurrentWorkspaceId,
  AssignmentRule,
  AssignmentRuleConditions
} from "../lib/api";
import { useToast } from "../components/Toast";
import Modal from "@/components/ui/Modal";
import { Combobox } from "@/components/ui/Combobox";

export default function AssignmentRulesPage() {
  const { t } = useTranslation();
  const wsId = getCurrentWorkspaceId() || "";
  const queryClient = useQueryClient();
  const toast = useToast();

  const [isModalOpen, setIsCreateModalOpen] = useState(false);
  const [editingRule, setEditingRule] = useState<AssignmentRule | null>(null);

  const [name, setName] = useState("");
  const [patternId, setPatternId] = useState("");
  const [priority, setPriority] = useState(0);
  const [conditions, setConditions] = useState<AssignmentRuleConditions>({});

  const { data: rules, isLoading: rulesLoading } = useQuery({
    queryKey: ["assignment-rules", wsId],
    queryFn: () => fetchAssignmentRules(wsId),
    enabled: !!wsId,
  });

  const { data: patterns } = useQuery({
    queryKey: ["working-patterns", wsId],
    queryFn: () => fetchWorkingPatterns(wsId),
    enabled: !!wsId,
  });

  const { data: departments } = useQuery({
    queryKey: ["departments", wsId],
    queryFn: () => fetchDepartments(wsId),
    enabled: !!wsId,
  });

  const createMutation = useMutation({
    mutationFn: (data: any) => createAssignmentRule(wsId, data),
    onSuccess: () => {
      toast.success(t("rules.messages.create_success"));
      closeModal();
      queryClient.invalidateQueries({ queryKey: ["assignment-rules"] });
    },
  });

  const updateMutation = useMutation({
    mutationFn: ({ id, data }: { id: string, data: any }) => updateAssignmentRule(wsId, id, data),
    onSuccess: () => {
      toast.success(t("rules.messages.update_success"));
      closeModal();
      queryClient.invalidateQueries({ queryKey: ["assignment-rules"] });
    },
  });

  const deleteMutation = useMutation({
    mutationFn: (id: string) => deleteAssignmentRule(wsId, id),
    onSuccess: () => {
      toast.success(t("rules.messages.delete_success"));
      queryClient.invalidateQueries({ queryKey: ["assignment-rules"] });
    }
  });

  const openCreateModal = () => {
    setEditingRule(null); setName(""); setPatternId(""); setPriority(0); setConditions({});
    setIsCreateModalOpen(true);
  };

  const openEditModal = (rule: AssignmentRule) => {
    setEditingRule(rule); setName(rule.name); setPatternId(rule.pattern_id);
    setPriority(rule.priority); setConditions(rule.conditions);
    setIsCreateModalOpen(true);
  };

  const closeModal = () => { setIsCreateModalOpen(false); setEditingRule(null); };

  const handleSubmit = () => {
    if (!name || !patternId) return;
    const payload = { name, pattern_id: patternId, priority, conditions };
    if (editingRule) updateMutation.mutate({ id: editingRule.id, data: payload });
    else createMutation.mutate(payload);
  };

  return (
    <div className="space-y-8 animate-in fade-in duration-500">
      {/* Header */}
      <div className="flex justify-between items-center px-2">
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-2xl bg-primary-50 flex items-center justify-center text-primary-600 shadow-sm border border-primary-100/50">
            <Zap size={20} />
          </div>
          <div>
            <h3 className="text-lg font-bold text-slate-800 tracking-tight">{t("rules.title")}</h3>
            <p className="text-xs text-slate-500 font-medium">{t("rules.subtitle")}</p>
          </div>
        </div>
        <button 
          onClick={openCreateModal}
          className="flex items-center gap-2 bg-primary-600 text-white px-6 py-2.5 rounded-2xl hover:bg-primary-700 transition-all shadow-lg shadow-primary-100 font-black text-[10px] uppercase tracking-widest active:scale-95"
        >
          <Plus size={14} /> {t("rules.create_btn")}
        </button>
      </div>

      <div className="bg-white border border-slate-100 rounded-[32px] overflow-hidden shadow-sm">
        <table className="w-full text-left border-collapse">
          <thead>
            <tr className="bg-slate-50/50 border-b border-slate-100">
              <th className="px-6 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("rules.table.priority")}</th>
              <th className="px-6 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("rules.table.name")}</th>
              <th className="px-6 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("rules.table.pattern")}</th>
              <th className="px-6 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("rules.table.conditions")}</th>
              <th className="px-6 py-4 text-right"></th>
            </tr>
          </thead>
          <tbody className="divide-y divide-slate-50">
            {rules?.map((rule) => (
              <tr key={rule.id} className="hover:bg-slate-50/50 transition-colors group">
                <td className="px-6 py-5">
                  <span className="bg-slate-100 text-slate-600 px-3 py-1 rounded-xl text-[10px] font-black tracking-tight border border-slate-200">
                    P{rule.priority}
                  </span>
                </td>
                <td className="px-6 py-5">
                  <div className="font-black text-slate-900 text-sm tracking-tight uppercase">{rule.name}</div>
                  {!rule.is_active && <span className="text-[8px] font-black uppercase text-rose-500 tracking-widest bg-rose-50 px-1.5 py-0.5 rounded mt-1 inline-block">{t("common.inactive")}</span>}
                </td>
                <td className="px-6 py-5">
                  <span className="bg-primary-50 text-primary-700 px-3 py-1.5 rounded-xl text-[10px] font-black uppercase tracking-widest border border-primary-100 shadow-sm">
                    {patterns?.find(p => p.id === rule.pattern_id)?.name || "Unknown"}
                  </span>
                </td>
                <td className="px-6 py-5">
                  <div className="flex flex-wrap gap-2">
                    {rule.conditions.department_id && (
                      <span className="bg-blue-50 text-blue-600 px-2.5 py-1 rounded-lg text-[9px] font-black uppercase tracking-widest border border-blue-100 flex items-center gap-1.5">
                        <Building2 size={10} /> {departments?.find(d => d.id === rule.conditions.department_id)?.name}
                      </span>
                    )}
                    {rule.conditions.tags?.map(tag => (
                      <span key={tag} className="bg-emerald-50 text-emerald-600 px-2.5 py-1 rounded-lg text-[9px] font-black uppercase tracking-widest border border-emerald-100">
                        #{tag}
                      </span>
                    ))}
                    {!rule.conditions.department_id && (!rule.conditions.tags || rule.conditions.tags.length === 0) && (
                      <span className="text-slate-300 text-[10px] font-black uppercase tracking-widest italic">{t("rules.match_all")}</span>
                    )}
                  </div>
                </td>
                <td className="px-6 py-5 text-right">
                  <div className="flex justify-end gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
                    <button onClick={() => openEditModal(rule)} className="p-2.5 bg-white text-slate-400 hover:text-primary-600 rounded-xl shadow-md border border-slate-100 transition-all"><Settings2 size={16} /></button>
                    <button onClick={() => { if(confirm(t("common.confirm_delete"))) deleteMutation.mutate(rule.id); }} className="p-2.5 bg-white text-slate-400 hover:text-rose-600 rounded-xl shadow-md border border-slate-100 transition-all"><Trash2 size={16} /></button>
                  </div>
                </td>
              </tr>
            ))}
            {(!rules || rules.length === 0) && !rulesLoading && (
              <tr><td colSpan={5} className="px-6 py-20 text-center text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50">{t("rules.empty_list")}</td></tr>
            )}
          </tbody>
        </table>
      </div>

      <div className="p-6 bg-blue-50/50 border border-blue-100 rounded-[2rem] flex gap-4">
        <div className="w-10 h-10 rounded-xl bg-blue-100 flex items-center justify-center text-blue-600 shrink-0"><AlertCircle size={20} /></div>
        <div className="text-xs text-blue-800 leading-relaxed font-medium">
          <p className="font-black uppercase tracking-widest mb-1 text-blue-900">{t("common.did_you_know")}</p>
          <p className="opacity-80">{t("rules.info_priority")}</p>
        </div>
      </div>

      <Modal isOpen={isModalOpen} onClose={closeModal} title={editingRule ? t("rules.edit_title") : t("rules.create_title")} subtitle={t("rules.form_subtitle")} icon={<Zap size={24} />} maxWidth="2xl">
        <div className="space-y-8 pt-4">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="space-y-2">
              <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("rules.form.name")}</label>
              <input type="text" value={name} onChange={(e) => setName(e.target.value)} placeholder={t("rules.form.name_placeholder")} className="w-full bg-slate-50 border border-slate-100 rounded-2xl px-5 py-4 focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all font-black text-sm uppercase tracking-tight" />
            </div>
            <div className="space-y-2">
              <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("rules.form.priority")}</label>
              <input type="number" value={priority} onChange={(e) => setPriority(parseInt(e.target.value))} className="w-full bg-slate-50 border border-slate-100 rounded-2xl px-5 py-4 focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all font-mono font-black text-sm text-center" />
            </div>
          </div>

          <div className="space-y-2">
            <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("rules.form.pattern")}</label>
            <select value={patternId} onChange={(e) => setPatternId(e.target.value)} className="w-full bg-slate-50 border border-slate-100 rounded-2xl px-5 py-4 focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all font-black text-sm uppercase tracking-tight">
              <option value="">{t("patterns.form.select_pattern_placeholder")}</option>
              {patterns?.map(p => (<option key={p.id} value={p.id}>{p.name}</option>))}
            </select>
          </div>

          <div className="bg-slate-50 rounded-[2.5rem] p-8 border border-slate-100 space-y-6">
            <h4 className="text-[10px] font-black text-slate-400 uppercase tracking-[0.2em] flex items-center gap-2"><Filter size={14} /> {t("rules.conditions_title")}</h4>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              <div className="space-y-2">
                <label className="text-[10px] font-black text-slate-500 uppercase tracking-widest ml-1">{t("common.department")}</label>
                <select value={conditions.department_id || ""} onChange={(e) => setConditions({ ...conditions, department_id: e.target.value || undefined })} className="w-full bg-white border border-slate-200 rounded-xl px-4 py-3 text-sm font-black uppercase tracking-tight shadow-sm"><option value="">{t("common.all_departments")}</option>{departments?.map(d => (<option key={d.id} value={d.id}>{d.name}</option>))}</select>
              </div>
              <div className="space-y-2">
                <label className="text-[10px] font-black text-slate-500 uppercase tracking-widest ml-1">{t("common.tags")}</label>
                <input type="text" value={conditions.tags?.join(", ") || ""} onChange={(e) => { const tags = e.target.value.split(",").map(t => t.trim()).filter(t => t !== ""); setConditions({ ...conditions, tags: tags.length > 0 ? tags : undefined }); }} placeholder={t("rules.form.tags_placeholder")} className="w-full bg-white border border-slate-200 rounded-xl px-4 py-3 text-sm font-bold shadow-sm" />
              </div>
            </div>
            <p className="text-[10px] text-slate-400 italic font-medium px-1">{t("rules.condition_help")}</p>
          </div>

          <div className="flex gap-4 pt-2">
            <button onClick={closeModal} className="flex-1 px-8 py-4 rounded-2xl font-black text-xs uppercase tracking-widest text-slate-500 hover:bg-slate-100 transition-all">{t("common.cancel")}</button>
            <button onClick={handleSubmit} disabled={createMutation.isPending || updateMutation.isPending} className="flex-1 bg-primary-600 hover:bg-primary-700 text-white font-black px-8 py-4 rounded-2xl flex items-center justify-center gap-3 transition-all active:scale-95 disabled:opacity-50 shadow-lg shadow-primary-100 text-xs uppercase tracking-widest">{editingRule ? t("common.save_changes") : t("common.create")}</button>
          </div>
        </div>
      </Modal>
    </div>
  );
}

      {/* Create/Edit Modal */}
      <Modal
        isOpen={isModalOpen}
        onClose={closeModal}
        title={editingRule ? t("rules.edit_title", "Edit Rule") : t("rules.create_title", "Create New Rule")}
        subtitle={t("rules.form_subtitle", "Define when this pattern should be applied")}
        maxWidth="2xl"
      >
        <div className="space-y-6 pt-4">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div className="space-y-2">
              <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("rules.form.name", "Rule Name")}</label>
              <input
                type="text"
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder={t("rules.form.name_placeholder")}
                className="w-full bg-slate-50 border-none rounded-2xl px-5 py-3.5 font-bold text-slate-900 focus:ring-2 focus:ring-primary-500 transition-all"
              />
            </div>
            
            <div className="space-y-2">
              <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("rules.form.priority")}</label>
              <input
                type="number"
                value={priority}
                onChange={(e) => setPriority(parseInt(e.target.value))}
                className="w-full bg-slate-50 border-none rounded-2xl px-5 py-3.5 font-bold text-slate-900 focus:ring-2 focus:ring-primary-500 transition-all"
              />
            </div>
          </div>

          <div className="space-y-2">
            <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("rules.form.pattern")}</label>
            <select
              value={patternId}
              onChange={(e) => setPatternId(e.target.value)}
              className="w-full bg-slate-50 border-none rounded-2xl px-5 py-3.5 font-bold text-slate-900 focus:ring-2 focus:ring-primary-500 transition-all"
            >
              <option value="">{t("patterns.form.select_pattern_placeholder")}</option>
              {patterns?.map(p => (
                <option key={p.id} value={p.id}>{p.name}</option>
              ))}
            </select>
          </div>

          <div className="bg-slate-50 rounded-[32px] p-6 border border-slate-100 space-y-4">
            <h4 className="text-xs font-black text-slate-500 uppercase tracking-widest flex items-center gap-2">
              <Filter size={14} /> {t("rules.conditions_title")}
            </h4>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div className="space-y-2">
                <label className="text-[10px] font-bold text-slate-500">{t("common.department")}</label>
                <select
                  value={conditions.department_id || ""}
                  onChange={(e) => setConditions({ ...conditions, department_id: e.target.value || undefined })}
                  className="w-full bg-white border border-slate-100 rounded-xl px-4 py-2 text-sm font-bold"
                >
                  <option value="">{t("common.all_departments")}</option>
                  {departments?.map(d => (
                    <option key={d.id} value={d.id}>{d.name}</option>
                  ))}
                </select>
              </div>

              <div className="space-y-2">
                <label className="text-[10px] font-bold text-slate-500">{t("common.tags")}</label>
                <input
                  type="text"
                  value={conditions.tags?.join(", ") || ""}
                  onChange={(e) => {
                    const tags = e.target.value.split(",").map(t => t.trim()).filter(t => t !== "");
                    setConditions({ ...conditions, tags: tags.length > 0 ? tags : undefined });
                  }}
                  placeholder={t("rules.form.tags_placeholder")}
                  className="w-full bg-white border border-slate-100 rounded-xl px-4 py-2 text-sm font-bold"
                />
              </div>
            </div>
            
            <p className="text-[10px] text-slate-400 italic">
              {t("rules.condition_help")}
            </p>
          </div>

          <div className="flex gap-3 pt-2">
            <button
              onClick={closeModal}
              className="flex-1 px-6 py-4 rounded-2xl font-black text-xs uppercase tracking-widest text-slate-500 hover:bg-slate-100 transition-all"
            >
              {t("common.cancel")}
            </button>
            <button
              onClick={handleSubmit}
              disabled={createMutation.isPending || updateMutation.isPending}
              className="flex-1 bg-primary-600 hover:bg-primary-700 text-white font-black px-6 py-4 rounded-2xl flex items-center justify-center gap-2 shadow-lg shadow-primary-100 transition-all active:scale-95 disabled:opacity-50 text-xs uppercase tracking-widest"
            >
              <Check className="w-4 h-4" /> {editingRule ? t("common.save_changes") : t("common.create")}
            </button>
          </div>
        </div>
      </Modal>
    </div>
  );
}
