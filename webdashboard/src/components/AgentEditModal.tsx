import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import {
  X,
  Save,
  Key,
  UserCheck,
  ShieldCheck,
  ShieldAlert,
  Power,
  Loader2,
  Cpu
} from "lucide-react";
import { AuthorizedAgent } from "@/lib/api";
import { useAuthorizedAgents } from "@/hooks/use-agents";
import { cn } from "@/lib/utils";

interface AgentEditModalProps {
  agent: AuthorizedAgent | null;
  isOpen: boolean;
  onClose: () => void;
  onSave: (updatedAgent: AuthorizedAgent) => void;
}

export default function AgentEditModal({ agent, isOpen, onClose, onSave }: AgentEditModalProps) {
  const { t } = useTranslation();
  const { updateAgent, isUpdating } = useAuthorizedAgents();
  
  // Initialize state directly from agent since parent uses key={agent?.id} to force remount
  const [name, setName] = useState(agent?.name || "");
  const [token, setToken] = useState(agent?.token || "");
  const [isTrusted, setIsTrusted] = useState(agent?.is_trusted || false);
  const [isActive, setIsActive] = useState(agent?.is_active ?? true);
  const [maxConcurrentWorkers, setMaxConcurrentWorkers] = useState(agent?.max_concurrent_workers || 2);

  if (!isOpen || !agent) return null;

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    try {
      const updated = await updateAgent({ 
        id: agent!.id, 
        agent: { 
          name, 
          token, 
          is_trusted: isTrusted, 
          is_active: isActive,
          max_concurrent_workers: maxConcurrentWorkers
        } 
      });
      onSave(updated || agent!); onClose();
    } catch (e) { console.error(e); }
  }

  return (
    <div className="fixed inset-0 z-[100] flex items-center justify-center p-4 bg-slate-900/60 backdrop-blur-md animate-in fade-in duration-300">
      <div className="w-full max-w-xl bg-white rounded-[2.5rem] shadow-premium-2xl border border-white overflow-hidden animate-in zoom-in-95 duration-300">
        <div className="flex items-center justify-between p-10 border-b border-slate-50 bg-slate-50/30">
          <div className="flex items-center gap-5">
            <div className="bg-primary-50 p-4 rounded-2xl text-primary-600 shadow-inner border border-primary-100/50">
              <UserCheck size={24} />
            </div>
            <div>
              <h2 className="text-xl font-bold text-slate-700 tracking-tight uppercase">
                {t("common.agents.edit_modal.title")}
              </h2>
              <p className="text-[10px] text-slate-500 font-bold uppercase tracking-widest mt-1 opacity-70">{t("common.agents.edit_modal.subtitle")}</p>
            </div>
          </div>
          <button onClick={onClose} className="p-3 bg-white hover:bg-slate-50 rounded-xl transition-all shadow-md border border-slate-100 text-slate-500 hover:text-rose-600 active:scale-90"><X size={20} /></button>
        </div>

        <form onSubmit={handleSubmit} className="p-10 space-y-10 animate-in fade-in duration-500">
          <div className="space-y-6">
            <div className="space-y-2 px-1">
              <label className="text-[10px] font-bold text-slate-500 uppercase tracking-widest ml-1">{t("common.agents.edit_modal.id_label")}</label>
              <div className="w-full bg-slate-100/50 border border-slate-200 rounded-2xl px-5 py-4 text-slate-500 font-mono font-bold shadow-inner opacity-60 uppercase text-xs">{agent.id}</div>
            </div>

            <div className="space-y-2 px-1">
              <label className="text-[10px] font-bold text-slate-500 uppercase tracking-widest ml-1">{t("common.agents.edit_modal.name_label")}</label>
              <input required placeholder={t("common.agents.edit_modal.name_placeholder")} className="w-full bg-slate-50 border border-slate-100 rounded-2xl px-5 py-4 focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all font-bold text-sm uppercase tracking-tight" value={name} onChange={(e) => setName(e.target.value)} />
            </div>

            <div className="grid grid-cols-2 gap-8">
              <div className="space-y-2 px-1">
                <label className="text-[10px] font-bold text-slate-500 uppercase tracking-widest ml-1">{t("common.agents.edit_modal.token_label")}</label>
                <div className="relative group">
                  <Key className="absolute left-5 top-1/2 -translate-y-1/2 text-slate-300 group-focus-within:text-primary-500 transition-colors" size={18} />
                  <input required placeholder={t("common.agents.edit_modal.token_placeholder")} className="w-full pl-12 pr-5 py-4 bg-slate-50 border border-slate-100 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all font-mono text-sm tracking-widest" value={token} onChange={(e) => setToken(e.target.value)} />
                </div>
              </div>
              <div className="space-y-2 px-1">
                <label className="text-[10px] font-bold text-slate-500 uppercase tracking-widest ml-1">{t("devices.form.strategy")}</label>
                <div className="relative group">
                  <input type="number" min="1" max="100" required className="w-full bg-slate-50 border border-slate-100 rounded-2xl px-5 py-4 focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all pr-12 font-mono font-bold text-sm text-slate-700" value={maxConcurrentWorkers} onChange={(e) => setMaxConcurrentWorkers(parseInt(e.target.value))} />
                  <Cpu className="absolute right-4 top-1/2 -translate-y-1/2 w-4 h-4 text-slate-300 group-focus-within:text-primary-500 transition-colors" />
                </div>
              </div>
            </div>

            <div className="grid grid-cols-2 gap-6 pt-2">
              <button type="button" onClick={() => setIsTrusted(!isTrusted)} className={cn("flex flex-col items-center gap-3 p-6 rounded-[2rem] border-2 transition-all font-bold text-[10px] uppercase tracking-widest active:scale-95 shadow-inner", isTrusted ? "bg-emerald-50 border-emerald-100 text-emerald-700 shadow-emerald-900/5" : "bg-slate-50 border-slate-100 text-slate-400")}>
                {isTrusted ? <ShieldCheck size={24} /> : <ShieldAlert size={24} />}
                {isTrusted ? t("common.agents.status.trusted") : t("common.agents.status.pending")}
              </button>

              <button type="button" onClick={() => setIsActive(!isActive)} className={cn("flex flex-col items-center gap-3 p-6 rounded-[2rem] border-2 transition-all font-bold text-[10px] uppercase tracking-widest active:scale-95 shadow-inner", isActive ? "bg-primary-50 border-primary-100 text-primary-700 shadow-primary-900/5" : "bg-rose-50 border-rose-100 text-rose-700 shadow-rose-900/5")}>
                <Power size={24} />
                {isActive ? t("common.status_active") : t("common.status_inactive")}
              </button>
            </div>
          </div>

          <div className="flex gap-4 pt-4 border-t border-slate-50">
            <button type="button" onClick={onClose} className="flex-1 px-8 py-4 bg-white border border-slate-200 rounded-2xl font-bold text-xs uppercase tracking-widest text-slate-500 hover:bg-slate-50 transition-all active:scale-95">{t("common.cancel").toUpperCase()}</button>
            <button type="submit" disabled={isUpdating} className="flex-1 bg-primary-600 text-white px-8 py-4 rounded-2xl font-bold uppercase tracking-widest text-xs hover:bg-primary-700 transition-all shadow-xl shadow-primary-100 active:scale-95 disabled:opacity-50 flex items-center justify-center gap-3">
              {isUpdating ? <Loader2 size={18} className="animate-spin" /> : <Save size={18} />}
              {isUpdating ? t("common.saving").toUpperCase() : t("common.save_changes").toUpperCase()}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
