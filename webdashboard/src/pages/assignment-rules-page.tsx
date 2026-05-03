import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { 
  Plus, 
  Trash2, 
  Pencil,
  Filter,
  Save,
  AlertCircle,
  Building2,
  Zap
} from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { Combobox } from "@/components/ui/combobox";
import Modal from "@/components/ui/modal";
import StatusBadge from "@/components/ui/status-badge";

import { useToast } from "../components/ui/toast";
import { useConfirm } from "../hooks/use-confirm";
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

export default function AssignmentRulesPage() {
  const { t } = useTranslation();
  const wsId = getCurrentWorkspaceId() ?? "";
  const queryClient = useQueryClient();
  const toast = useToast();
  const { confirm, ConfirmationDialog } = useConfirm();

  const [isModalOpen, setIsCreateModalOpen] = useState(false);
  const [editingRule, setEditingRule] = useState<AssignmentRule | null>(null);

  const [name, setName] = useState("");
  const [patternId, setPatternId] = useState("");
  const [priority, setPriority] = useState(0);
  const [conditions, setConditions] = useState<AssignmentRuleConditions>({});

  const { data: rules, isLoading: rulesLoading } = useQuery({ queryKey: ["assignment-rules", wsId], queryFn: () => fetchAssignmentRules(wsId), enabled: !!wsId });
  const { data: patterns } = useQuery({ queryKey: ["working-patterns", wsId], queryFn: () => fetchWorkingPatterns(wsId), enabled: !!wsId });
  const { data: departments } = useQuery({ queryKey: ["departments", wsId], queryFn: () => fetchDepartments(wsId), enabled: !!wsId });

  const createMutation = useMutation({
    mutationFn: (data: Parameters<typeof createAssignmentRule>[1]) => createAssignmentRule(wsId, data),
    onSuccess: () => { toast.success(t("rules.messages.create_success")); closeModal(); void queryClient.invalidateQueries({ queryKey: ["assignment-rules"] }); },
  });

  const updateMutation = useMutation({
    mutationFn: ({ id, data }: { id: string, data: Parameters<typeof updateAssignmentRule>[2] }) => updateAssignmentRule(wsId, id, data),
    onSuccess: () => { toast.success(t("rules.messages.update_success")); closeModal(); void queryClient.invalidateQueries({ queryKey: ["assignment-rules"] }); },
  });

  const deleteMutation = useMutation({
    mutationFn: (id: string) => deleteAssignmentRule(wsId, id),
    onSuccess: () => { toast.success(t("rules.messages.delete_success")); void queryClient.invalidateQueries({ queryKey: ["assignment-rules"] }); }
  });

  const openCreateModal = () => { setName(""); setPatternId(""); setPriority(0); setConditions({}); setIsCreateModalOpen(true); };
  const openEditModal = (rule: AssignmentRule) => { setEditingRule(rule); setName(rule.name); setPatternId(rule.pattern_id); setPriority(rule.priority); setConditions(rule.conditions); setIsCreateModalOpen(true); };
  const closeModal = () => { setIsCreateModalOpen(false); setEditingRule(null); };

  const handleSubmit = () => {
    if (!name || !patternId) return;
    const payload = { name, pattern_id: patternId, priority, conditions };
    if (editingRule) updateMutation.mutate({ id: editingRule.id, data: payload });
    else createMutation.mutate(payload);
  };

  return (
    <div className="space-y-10 animate-in fade-in duration-700 pb-20">
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-8 px-2">
        <div className="space-y-1">
          <h1 className="text-4xl font-bold tracking-tighter text-slate-700 dark:text-slate-200 uppercase leading-none" data-testid="assignment-rules-title">{t("rules.title")}</h1>
          <p className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] opacity-60">{t("rules.subtitle")}</p>
        </div>
        <Button icon={<Plus size={18} />} onClick={openCreateModal}>{t("rules.create_btn").toUpperCase()}</Button>
      </div>

      <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-[3rem] overflow-hidden shadow-sm dark:shadow-black/10 animate-in slide-in-from-bottom-8 duration-700">
        <table className="w-full text-left border-collapse">
          <thead>
            <tr className="bg-slate-50 dark:bg-slate-800/50 dark:bg-slate-800/50 border-b border-slate-100 dark:border-slate-800">
              <th className="px-10 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em]">{t("rules.table.priority")}</th>
              <th className="px-8 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em]">{t("rules.table.name")}</th>
              <th className="px-8 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em]">{t("rules.table.pattern")}</th>
              <th className="px-8 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em]">{t("rules.table.conditions")}</th>
              <th className="px-10 py-6 border-b border-slate-100 dark:border-slate-800"></th>
            </tr>
          </thead>
          <tbody className="divide-y divide-slate-50 dark:divide-slate-800">
            {rules?.map((rule) => (
              <tr className="hover:bg-slate-50 dark:hover:bg-slate-800/50 dark:hover:bg-slate-800/50 transition-all group" key={rule.id}>
                <td className="px-10 py-6"><span className="bg-slate-100 dark:bg-slate-700 text-slate-600 dark:text-slate-300 px-3 py-1.5 rounded-xl text-[10px] font-bold tracking-tight border border-slate-200 dark:border-slate-700 shadow-inner font-mono uppercase">P{rule.priority}</span></td>
                <td className="px-8 py-6">
                  <div className="font-bold text-slate-700 dark:text-slate-200 text-sm tracking-tight uppercase leading-none mb-1.5">{rule.name}</div>
                  {!rule.is_active && <StatusBadge variant="danger">{t("common.inactive").toUpperCase()}</StatusBadge>}
                </td>
                <td className="px-8 py-6"><span className="bg-primary-50 dark:bg-primary-950 text-primary-600 dark:text-primary-400 px-3 py-1.5 rounded-xl text-[10px] font-bold uppercase tracking-widest border border-primary-100 dark:border-primary-800 shadow-sm dark:shadow-black/10">{patterns?.find(p => p.id === rule.pattern_id)?.name.toUpperCase() ?? "UNKNOWN"}</span></td>
                <td className="px-8 py-6">
                  <div className="flex flex-wrap gap-2.5">
                    {!!rule.conditions.department_id && (<span className="bg-blue-50 dark:bg-blue-950 text-blue-600 dark:text-blue-300 px-3 py-1.5 rounded-[10px] text-[9px] font-bold uppercase tracking-widest border border-blue-100 dark:border-blue-800 flex items-center gap-2 shadow-sm dark:shadow-black/10"><Building2 className="opacity-50" size={12} /> {departments?.find(d => d.id === rule.conditions.department_id)?.name.toUpperCase()}</span>)}
                    {rule.conditions.tags?.map(tag => (<span className="bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 px-3 py-1.5 rounded-[10px] text-[9px] font-bold uppercase tracking-widest border border-emerald-100 dark:border-emerald-800 shadow-sm dark:shadow-black/10" key={tag}>#{tag.toUpperCase()}</span>))}
                    {!rule.conditions.department_id && (!rule.conditions.tags || rule.conditions.tags.length === 0) && (
                      <span className="text-slate-300 dark:text-slate-600 text-[10px] font-bold uppercase tracking-[0.2em] italic opacity-60">{t("rules.match_all")}</span>
                    )}
                  </div>
                </td>
                <td className="px-10 py-6 text-right">
                  <div className="flex justify-end gap-3 opacity-0 group-hover:opacity-100 transition-all duration-300 translate-x-2 group-hover:translate-x-0">
                    <button className="p-3 bg-white dark:bg-slate-800 text-slate-500 dark:text-slate-400 hover:text-primary-600 dark:hover:text-primary-400 rounded-xl shadow-md dark:shadow-black/20 border border-slate-100 dark:border-slate-700 transition-all active:scale-90" onClick={() => { openEditModal(rule); }} title={t("common.edit")}><Pencil size={18} /></button>
                    <button className="p-3 bg-white dark:bg-slate-800 text-slate-500 dark:text-slate-400 hover:text-rose-600 dark:hover:text-rose-400 rounded-xl shadow-md dark:shadow-black/20 border border-slate-100 dark:border-slate-700 transition-all active:scale-90" onClick={async () => { const ok = await confirm({ title: t("common.confirm_delete"), message: "Are you sure you want to delete this rule?", variant: "danger" }); if(ok) deleteMutation.mutate(rule.id); }} title={t("common.delete")}><Trash2 size={18} /></button>
                  </div>
                </td>
              </tr>
            ))}
            {(!rules || rules.length === 0) && !rulesLoading && (
              <tr><td className="px-10 py-32 text-center bg-slate-50 dark:bg-slate-800/30 dark:bg-slate-800/30 flex flex-col items-center" colSpan={5}><Zap className="text-slate-200 dark:text-slate-700 mb-4" size={40} /><p className="text-[10px] font-bold text-slate-300 dark:text-slate-600 uppercase tracking-widest">{t("rules.empty_list")}</p></td></tr>
            )}
          </tbody>
        </table>
      </div>

      <div className="p-8 bg-blue-50 dark:bg-blue-950/50 dark:bg-blue-950/50 border border-blue-100 dark:border-blue-900 rounded-[2.5rem] flex gap-6 shadow-inner animate-in fade-in slide-in-from-bottom-4 duration-1000">
        <div className="w-12 h-12 rounded-2xl bg-white dark:bg-slate-800 flex items-center justify-center text-blue-600 dark:text-blue-400 shadow-sm dark:shadow-black/10 border border-blue-100 dark:border-blue-900 shrink-0"><AlertCircle size={24} /></div>
        <div className="text-xs text-blue-900 dark:text-blue-200 leading-relaxed font-medium pt-1">
          <p className="font-bold uppercase tracking-[0.2em] mb-2 text-blue-950 dark:text-blue-100 opacity-80">{t("common.did_you_know")}</p>
          <p className="opacity-70 font-bold uppercase tracking-tight">{t("rules.info_priority")}</p>
        </div>
      </div>

      {/* Create/Edit Modal */}
      <Modal
        isOpen={isModalOpen}
        maxWidth="2xl"
        onClose={closeModal}
        subtitle={t("rules.form_subtitle", "Define when this pattern should be applied")}
        title={editingRule ? t("rules.edit_title", "Edit Rule") : t("rules.create_title", "Create New Rule")}
      >
        <div className="space-y-6 pt-4">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div className="space-y-2">
              <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">{t("rules.form.name", "Rule Name")}</label>
              <input
                className="w-full bg-slate-50 dark:bg-slate-800 border-none rounded-2xl px-5 py-3.5 font-bold text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-primary-500 transition-all"
                onChange={(e) => { setName(e.target.value); }}
                placeholder={t("rules.form.name_placeholder")}
                type="text"
                value={name}
              />
            </div>

            <div className="space-y-2">
              <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">{t("rules.form.priority")}</label>
              <input
                className="w-full bg-slate-50 dark:bg-slate-800 border-none rounded-2xl px-5 py-3.5 font-bold text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-primary-500 transition-all"
                onChange={(e) => { setPriority(parseInt(e.target.value)); }}
                type="number"
                value={priority}
              />
            </div>
          </div>

          <div className="space-y-2">
            <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">{t("rules.form.pattern")}</label>
            <Combobox
              className="w-full bg-slate-50 dark:bg-slate-800 border-none rounded-2xl py-[5px] font-bold text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-primary-500 transition-all"
              data-testid="pattern-combobox"
              onChange={setPatternId}
              options={(patterns ?? []).map((p) => ({ value: p.id, label: p.name }))}
              placeholder={t("patterns.form.select_pattern_placeholder")}
              value={patternId}
            />
          </div>

          <div className="bg-slate-50 dark:bg-slate-800 rounded-[32px] p-6 border border-slate-100 dark:border-slate-800 space-y-4">
            <h4 className="text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest flex items-center gap-2">
              <Filter size={14} /> {t("rules.conditions_title")}
            </h4>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div className="space-y-2">
                <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400">{t("common.department")}</label>
                <Combobox
                  className="w-full bg-white dark:bg-slate-900 border-slate-100 dark:border-slate-700 rounded-xl text-sm font-bold text-slate-700 dark:text-slate-200"
                  onChange={(value) => { setConditions({ ...conditions, department_id: value || undefined }); }}
                  options={(departments ?? []).map((d) => ({ value: d.id, label: d.name }))}
                  placeholder={t("common.all_departments")}
                  value={conditions.department_id ?? ""}
                />
              </div>

              <div className="space-y-2">
                <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400">{t("common.tags")}</label>
                <input
                  className="w-full bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-700 rounded-xl px-4 py-2 text-sm font-bold text-slate-700 dark:text-slate-200"
                  onChange={(e) => {
                    const tags = e.target.value.split(",").map(t => t.trim()).filter(t => t !== "");
                    setConditions({ ...conditions, tags: tags.length > 0 ? tags : undefined });
                  }}
                  placeholder={t("rules.form.tags_placeholder")}
                  type="text"
                  value={conditions.tags?.join(", ") ?? ""}
                />
              </div>
            </div>

            <p className="text-[10px] text-slate-500 dark:text-slate-400 italic">
              {t("rules.condition_help")}
            </p>
          </div>

          <div className="flex gap-3 pt-2">
            <Button className="flex-1" onClick={closeModal} size="lg" variant="ghost">
              {t("common.cancel")}
            </Button>
            <Button className="flex-1" icon={<Save className="w-4 h-4" />} loading={createMutation.isPending || updateMutation.isPending} onClick={handleSubmit} size="lg">
              {editingRule ? t("common.save_changes") : t("common.create")}
            </Button>
          </div>
        </div>
      </Modal>
      <ConfirmationDialog />
    </div>
  );
}
