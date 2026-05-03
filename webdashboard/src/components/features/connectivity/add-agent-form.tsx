import { Plus } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

interface AddAgentFormProps {
  newAgent: { id: string; name: string; token: string };
  onCancel: () => void;
  onChange: (field: string, value: string) => void;
  onSubmit: (e: React.SyntheticEvent) => void;
}

export function AddAgentForm({
  newAgent,
  onCancel,
  onChange,
  onSubmit,
}: AddAgentFormProps) {
  const { t } = useTranslation();

  return (
    <form className="bg-white dark:bg-slate-900 border-2 border-primary-100 dark:border-primary-800 p-10 rounded-[2.5rem] shadow-xl shadow-primary-50/50 animate-in slide-in-from-top-4 duration-500" data-testid="add-agent-form" onSubmit={onSubmit}>
      <div className="flex items-center gap-3 mb-8 text-primary-700 dark:text-primary-300">
        <div className="w-8 h-8 rounded-xl bg-primary-50 dark:bg-primary-950 flex items-center justify-center shadow-inner"><Plus size={16} /></div>
        <h4 className="text-[10px] font-bold uppercase tracking-[0.2em]">{t("agents.add_form_title")}</h4>
      </div>
      <div className="grid grid-cols-1 md:grid-cols-3 gap-8">
        <div className="space-y-3 px-1">
          <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{t("agents.id_label")}</label>
          <input className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-4 focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-mono font-bold text-sm tracking-tight" data-testid="agent-id-input" onChange={(e) => { onChange("id", e.target.value); }} placeholder="agent-001" required value={newAgent.id} />
        </div>
        <div className="space-y-3 px-1">
          <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{t("agents.name_label")}</label>
          <input className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-4 focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-bold text-sm tracking-tight" data-testid="agent-name-input" onChange={(e) => { onChange("name", e.target.value); }} placeholder="Office Main" required value={newAgent.name} />
        </div>
        <div className="space-y-3 px-1">
          <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{t("agents.token_label")}</label>
          <input className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-4 focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-mono font-bold text-sm" data-testid="agent-token-input" onChange={(e) => { onChange("token", e.target.value); }} placeholder="Connection token" required value={newAgent.token} />
        </div>
      </div>
      <div className="mt-10 flex justify-end gap-4">
        <button className="px-8 py-4 rounded-2xl font-bold text-xs uppercase tracking-widest text-slate-500 dark:text-slate-400 hover:bg-slate-50 dark:hover:bg-slate-800 transition-all" data-testid="agent-form-cancel" onClick={onCancel} type="button">{t("common.cancel")}</button>
        <button className="bg-primary-600 text-white px-12 py-4 rounded-2xl font-bold text-xs uppercase tracking-widest hover:bg-primary-700 transition-all shadow-xl shadow-primary-100 dark:shadow-black/30 active:scale-95" data-testid="agent-form-submit" type="submit">{t("agents.save_btn")}</button>
      </div>
    </form>
  );
}
