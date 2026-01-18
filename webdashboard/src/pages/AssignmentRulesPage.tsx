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

  // Form State
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
      toast.success(t("rules.messages.create_success", "Rule created successfully"));
      closeModal();
      queryClient.invalidateQueries({ queryKey: ["assignment-rules"] });
      queryClient.invalidateQueries({ queryKey: ["schedules"] });
    },
    onError: (error: any) => toast.error(error.message)
  });

  const updateMutation = useMutation({
    mutationFn: ({ id, data }: { id: string, data: any }) => updateAssignmentRule(wsId, id, data),
    onSuccess: () => {
      toast.success(t("rules.messages.update_success", "Rule updated successfully"));
      closeModal();
      queryClient.invalidateQueries({ queryKey: ["assignment-rules"] });
      queryClient.invalidateQueries({ queryKey: ["schedules"] });
    },
    onError: (error: any) => toast.error(error.message)
  });

  const deleteMutation = useMutation({
    mutationFn: (id: string) => deleteAssignmentRule(wsId, id),
    onSuccess: () => {
      toast.success(t("rules.messages.delete_success", "Rule deleted"));
      queryClient.invalidateQueries({ queryKey: ["assignment-rules"] });
      queryClient.invalidateQueries({ queryKey: ["schedules"] });
    }
  });

  const openCreateModal = () => {
    setEditingRule(null);
    setName("");
    setPatternId("");
    setPriority(0);
    setConditions({});
    setIsCreateModalOpen(true);
  };

  const openEditModal = (rule: AssignmentRule) => {
    setEditingRule(rule);
    setName(rule.name);
    setPatternId(rule.pattern_id);
    setPriority(rule.priority);
    setConditions(rule.conditions);
    setIsCreateModalOpen(true);
  };

  const closeModal = () => {
    setIsCreateModalOpen(false);
    setEditingRule(null);
  };

  const handleSubmit = () => {
    if (!name || !patternId) {
      toast.warning(t("rules.validation.required_fields", "Name and Pattern are required"));
      return;
    }

    const payload = {
      name,
      pattern_id: patternId,
      priority,
      conditions
    };

    if (editingRule) {
      updateMutation.mutate({ id: editingRule.id, data: payload });
    } else {
      createMutation.mutate(payload);
    }
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex justify-between items-center">
        <div>
          <h1 className="text-3xl font-bold text-slate-900 tracking-tight">{t("rules.title", "Auto-Assignment Rules")}</h1>
          <p className="text-slate-500">{t("rules.subtitle", "Automatically assign patterns to employees based on conditions")}</p>
        </div>
        <button 
          onClick={openCreateModal}
          className="bg-primary-600 hover:bg-primary-700 text-white font-black px-6 py-3 rounded-2xl flex items-center gap-2 shadow-lg shadow-primary-100 transition-all active:scale-95 text-xs uppercase tracking-widest"
        >
          <Plus size={16} /> {t("rules.create_btn", "Add Rule")}
        </button>
      </div>

      {/* Rules List */}
      <div className="bg-white border border-slate-200 rounded-[32px] overflow-hidden shadow-sm">
        <table className="w-full text-left">
          <thead>
            <tr className="bg-slate-50/50 border-b border-slate-100">
              <th className="px-6 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("rules.table.priority", "Priority")}</th>
              <th className="px-6 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("rules.table.name", "Rule Name")}</th>
              <th className="px-6 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("rules.table.pattern", "Assigned Pattern")}</th>
              <th className="px-6 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("rules.table.conditions", "Conditions")}</th>
              <th className="px-6 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest text-right">{t("common.actions")}</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-slate-50">
            {rules?.map((rule) => (
              <tr key={rule.id} className="hover:bg-slate-50/50 transition-colors group">
                <td className="px-6 py-4">
                  <span className="bg-slate-100 text-slate-600 px-2 py-1 rounded-lg text-[10px] font-black tracking-tighter">
                    {rule.priority}
                  </span>
                </td>
                <td className="px-6 py-4">
                  <div className="font-bold text-slate-900">{rule.name}</div>
                  {!rule.is_active && <span className="text-[8px] font-black uppercase text-rose-500 tracking-widest">{t("common.inactive")}</span>}
                </td>
                <td className="px-6 py-4">
                  <div className="flex items-center gap-2">
                    <span className="bg-primary-50 text-primary-700 px-3 py-1 rounded-xl text-xs font-bold border border-primary-100">
                      {patterns?.find(p => p.id === rule.pattern_id)?.name || "Unknown Pattern"}
                    </span>
                  </div>
                </td>
                <td className="px-6 py-4">
                  <div className="flex flex-wrap gap-1">
                    {rule.conditions.department_id && (
                      <span className="bg-blue-50 text-blue-600 px-2 py-0.5 rounded-lg text-[10px] font-bold border border-blue-100">
                        Dept: {departments?.find(d => d.id === rule.conditions.department_id)?.name || "..."}
                      </span>
                    )}
                    {rule.conditions.tags?.map(tag => (
                      <span key={tag} className="bg-emerald-50 text-emerald-600 px-2 py-0.5 rounded-lg text-[10px] font-bold border border-emerald-100">
                        #{tag}
                      </span>
                    ))}
                    {!rule.conditions.department_id && (!rule.conditions.tags || rule.conditions.tags.length === 0) && (
                      <span className="text-slate-400 text-[10px] italic">{t("rules.match_all", "Global (Match All)")}</span>
                    )}
                  </div>
                </td>
                <td className="px-6 py-4 text-right">
                  <div className="flex justify-end gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
                    <button 
                      onClick={() => openEditModal(rule)}
                      className="p-2 hover:bg-primary-50 text-slate-400 hover:text-primary-600 rounded-xl transition-all"
                    >
                      <Settings2 size={16} />
                    </button>
                    <button 
                      onClick={() => {
                        if(confirm(t("common.confirm_delete"))) deleteMutation.mutate(rule.id);
                      }}
                      className="p-2 hover:bg-rose-50 text-slate-400 hover:text-rose-600 rounded-xl transition-all"
                    >
                      <Trash2 size={16} />
                    </button>
                  </div>
                </td>
              </tr>
            ))}
            {(!rules || rules.length === 0) && !rulesLoading && (
              <tr>
                <td colSpan={5} className="px-6 py-12 text-center text-slate-400 italic">
                  {t("rules.empty_list", "No assignment rules defined yet")}
                </td>
              </tr>
            )}
          </tbody>
        </table>
      </div>

      {/* Modal Tooltip Info */}
      <div className="bg-blue-50/50 border border-blue-100 rounded-2xl p-4 flex gap-3">
        <AlertCircle className="text-blue-500 shrink-0" size={20} />
        <div className="text-xs text-blue-700 leading-relaxed">
          <strong>{t("common.did_you_know", "Did you know?")}:</strong> {t("rules.info_priority", "Rules are evaluated from highest priority to lowest. The first rule that matches an employee's attributes will be applied. Global rules should have the lowest priority.")}
        </div>
      </div>

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
                placeholder="e.g., Factory Workers Night Shift"
                className="w-full bg-slate-50 border-none rounded-2xl px-5 py-3.5 font-bold text-slate-900 focus:ring-2 focus:ring-primary-500 transition-all"
              />
            </div>
            
            <div className="space-y-2">
              <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("rules.form.priority", "Priority (Higher = Stronger)")}</label>
              <input
                type="number"
                value={priority}
                onChange={(e) => setPriority(parseInt(e.target.value))}
                className="w-full bg-slate-50 border-none rounded-2xl px-5 py-3.5 font-bold text-slate-900 focus:ring-2 focus:ring-primary-500 transition-all"
              />
            </div>
          </div>

          <div className="space-y-2">
            <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("rules.form.pattern", "Select Pattern")}</label>
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
              <Filter size={14} /> {t("rules.conditions_title", "Match Conditions")}
            </h4>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div className="space-y-2">
                <label className="text-[10px] font-bold text-slate-500">{t("common.department", "Department")}</label>
                <select
                  value={conditions.department_id || ""}
                  onChange={(e) => setConditions({ ...conditions, department_id: e.target.value || undefined })}
                  className="w-full bg-white border border-slate-100 rounded-xl px-4 py-2 text-sm font-bold"
                >
                  <option value="">{t("common.all_departments", "All Departments")}</option>
                  {departments?.map(d => (
                    <option key={d.id} value={d.id}>{d.name}</option>
                  ))}
                </select>
              </div>

              <div className="space-y-2">
                <label className="text-[10px] font-bold text-slate-500">{t("common.tags", "Tags (Comma separated)")}</label>
                <input
                  type="text"
                  value={conditions.tags?.join(", ") || ""}
                  onChange={(e) => {
                    const tags = e.target.value.split(",").map(t => t.trim()).filter(t => t !== "");
                    setConditions({ ...conditions, tags: tags.length > 0 ? tags : undefined });
                  }}
                  placeholder="NightShift, Hanoi, VVIP"
                  className="w-full bg-white border border-slate-100 rounded-xl px-4 py-2 text-sm font-bold"
                />
              </div>
            </div>
            
            <p className="text-[10px] text-slate-400 italic">
              {t("rules.condition_help", "Leave empty to match all employees (within the scope of other conditions).")}
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
