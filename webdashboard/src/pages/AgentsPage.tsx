import React, { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { 
  Plus, 
  Trash2, 
  Server, 
  ShieldCheck, 
  ShieldAlert, 
  RefreshCw, 
  Search, 
  RotateCcw,
  Edit2,
  HardDrive
} from "lucide-react";
import { Link } from "react-router-dom";
import { useAuthorizedAgents } from "@/hooks/use-agents";
import { useToast } from "@/components/ui/Toast";
import { cn } from "@/lib/utils";
import Button from "@/components/ui/Button";
import StatusBadge from "@/components/ui/StatusBadge";
import { LABEL_BASE } from "@/components/ui/FormField";
import AgentEditModal from "@/components/features/agents/AgentEditModal";
import { AuthorizedAgent } from "@/lib/api";

export default function AgentsPage() {
  const { t } = useTranslation();
  const toast = useToast();
  const [, setTick] = useState(0);
  useEffect(() => {
    const interval = setInterval(() => setTick((t) => t + 1), 15000);
    return () => clearInterval(interval);
  }, []);

  const { agents, isLoading, isFetching, refetch, createAgent, updateTrust, deleteAgent, updateAgent, includeInactive, setIncludeInactive } = useAuthorizedAgents();
  const [showAddForm, setShowAddForm] = useState(false);
  const [newAgent, setNewAgent] = useState({ id: "", name: "", token: "" });
  const [editingAgent, setEditingAgent] = useState<AuthorizedAgent | null>(null);
  const [isEditModalOpen, setIsEditModalOpen] = useState(false);

  async function handleRestore(agent: AuthorizedAgent) {
    try {
      await updateAgent({ id: agent.id, agent: { is_active: true } });
      toast.success(t("agents.messages.restore_success", { name: agent.name }));
    } catch (e) { console.error(e); }
  }

  async function handleAdd(e: React.FormEvent) {
    e.preventDefault();
    try {
      await createAgent(newAgent);
      toast.success(t("agents.messages.add_success", { name: newAgent.name }));
      setShowAddForm(false); setNewAgent({ id: "", name: "", token: "" });
    } catch (e) { console.error(e); }
  }

  async function handleToggleTrust(agent: AuthorizedAgent) {
    if (!confirm(t(agent.is_trusted ? "agents.messages.confirm_revoke" : "agents.messages.confirm_approve", { name: agent.name }))) return;
    try {
      await updateTrust({ id: agent.id, is_trusted: !agent.is_trusted });
      toast.success(t(agent.is_trusted ? "agents.messages.revoke_success" : "agents.messages.approve_success", { name: agent.name }));
    } catch (e) { console.error(e); }
  }

  async function handleDelete(agent: AuthorizedAgent) {
    if (!confirm(t("agents.messages.confirm_delete", { name: agent.name }))) return;
    try {
      await deleteAgent(agent.id);
      toast.success(t("agents.messages.delete_success"));
    } catch (e) { console.error(e); }
  }

  const handleEdit = (agent: AuthorizedAgent) => { setEditingAgent(agent); setIsEditModalOpen(true); };
  const handleSave = (updated: AuthorizedAgent) => { toast.success(t("agents.messages.update_success", { name: updated.name })); refetch(); };

  return (
    <div className="space-y-10 animate-in fade-in duration-700 pb-20">
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-8 px-2">
        <div className="flex items-center gap-5">
          <div className="w-14 h-14 rounded-2xl bg-primary-50 dark:bg-primary-950 flex items-center justify-center text-primary-600 dark:text-primary-400 shadow-inner border border-primary-100 dark:border-primary-800/50"><Server size={32} /></div>
          <div>
            <h3 className="text-4xl font-bold tracking-tighter text-slate-700 dark:text-slate-200 uppercase leading-none">{t("agents.title")}</h3>
            <p className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] mt-2 opacity-60">{t("agents.subtitle")}</p>
          </div>
        </div>
        <div className="flex items-center gap-4">
          <button onClick={() => setIncludeInactive(!includeInactive)} className={cn("flex items-center gap-2 px-6 py-3 rounded-2xl font-bold text-[10px] uppercase tracking-widest transition-all border shadow-sm active:scale-95", includeInactive ? "bg-slate-900 text-white border-transparent" : "bg-white dark:bg-slate-900 text-slate-500 dark:text-slate-400 border-slate-200 dark:border-slate-700 hover:bg-slate-50 dark:hover:bg-slate-800")}>{includeInactive ? t("common.status_inactive").toUpperCase() : t("common.status_active").toUpperCase()}</button>
          <Button onClick={() => setShowAddForm(true)} icon={<Plus size={18} />}>{t("agents.add_btn").toUpperCase()}</Button>
        </div>
      </div>

      {showAddForm && (
        <form onSubmit={handleAdd} className="bg-white dark:bg-slate-900 border-2 border-primary-100 dark:border-primary-800 p-10 rounded-[2.5rem] shadow-xl shadow-primary-50/50 animate-in slide-in-from-top-4 duration-500">
          <div className="flex items-center gap-3 mb-8 text-primary-700 dark:text-primary-300">
            <div className="w-8 h-8 rounded-xl bg-primary-50 dark:bg-primary-950 flex items-center justify-center shadow-inner"><Plus size={16} /></div>
            <h4 className="text-[10px] font-bold uppercase tracking-[0.2em]">{t("agents.add_form_title")}</h4>
          </div>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-8">
            <div className="space-y-3 px-1"><label className={LABEL_BASE}>{t("agents.id_label")}</label><input required placeholder="agent-001" className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-4 focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-mono font-bold text-sm uppercase tracking-tight" value={newAgent.id} onChange={(e) => setNewAgent({ ...newAgent, id: e.target.value })} /></div>
            <div className="space-y-3 px-1"><label className={LABEL_BASE}>{t("agents.name_label")}</label><input required placeholder="Văn phòng chính" className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-4 focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-bold text-sm uppercase tracking-tight" value={newAgent.name} onChange={(e) => setNewAgent({ ...newAgent, name: e.target.value })} /></div>
            <div className="space-y-3 px-1"><label className={LABEL_BASE}>{t("agents.token_label")}</label><input required placeholder="Mật mã kết nối" className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-4 focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-mono font-bold text-sm" value={newAgent.token} onChange={(e) => setNewAgent({ ...newAgent, token: e.target.value })} /></div>
          </div>
          <div className="mt-10 flex justify-end gap-4">
            <Button variant="ghost" size="lg" onClick={() => setShowAddForm(false)}>{t("common.cancel")}</Button>
            <Button size="lg" type="submit">{t("agents.save_btn")}</Button>
          </div>
        </form>
      )}

      <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-[3rem] overflow-hidden shadow-sm">
        <table className="w-full text-left border-collapse">
          <thead className="bg-slate-50 dark:bg-slate-800/50 border-b border-slate-100 dark:border-slate-800">
            <tr>
              <th className="px-10 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em]">{t("agents.table.agent")}</th>
              <th className="px-8 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em]">{t("agents.table.status")}</th>
              <th className="px-8 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] text-center">{t("agents.table.last_seen")}</th>
              <th className="px-10 py-6 border-b border-slate-100 dark:border-slate-800"></th>
            </tr>
          </thead>
          <tbody className="divide-y divide-slate-50 dark:divide-slate-800">
            {isLoading ? (
              <tr><td colSpan={4} className="px-10 py-20 text-center text-slate-500 dark:text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50 animate-pulse">{t("common.loading")}</td></tr>
            ) : agents.length === 0 ? (
              <tr><td colSpan={4} className="px-10 py-32 text-center text-slate-500 dark:text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50 flex flex-col items-center gap-4">
                <HardDrive size={40} className="text-slate-200 dark:text-slate-700" />
                {t("agents.table.no_agents")}
              </td></tr>
            ) : agents.map((agent) => (
              <tr key={agent.id} className={cn("hover:bg-slate-50 dark:hover:bg-slate-800/50 transition-all group", !agent.is_active && "opacity-50 grayscale bg-slate-50 dark:bg-slate-800/30")}>
                <td className="px-10 py-6">
                  <div className="flex items-center gap-5">
                    <div className={cn("w-14 h-14 rounded-2xl flex items-center justify-center font-bold shadow-inner border transition-all group-hover:scale-110", agent.is_active ? "bg-primary-50 dark:bg-primary-950 text-primary-600 dark:text-primary-400 border-primary-100 dark:border-primary-800/50" : "bg-slate-100 dark:bg-slate-700 text-slate-500 dark:text-slate-400 border-slate-200 dark:border-slate-700")}>
                      <Server size={24} />
                    </div>
                    <div>
                      <div className="font-bold text-slate-700 dark:text-slate-200 text-sm tracking-tight uppercase leading-none mb-1.5">{agent.name}</div>
                      <div className="text-[10px] text-slate-500 dark:text-slate-400 font-mono font-bold uppercase opacity-70">ID: {agent.id}</div>
                    </div>
                  </div>
                </td>
                <td className="px-8 py-6">
                  <div className="flex flex-col gap-2">
                    <StatusBadge variant={agent.is_trusted ? "success" : "warning"}>{agent.is_trusted ? t("common.agents.status.trusted").toUpperCase() : t("common.agents.status.pending").toUpperCase()}</StatusBadge>
                    {!agent.is_active && <StatusBadge variant="neutral">{t("common.status_inactive").toUpperCase()}</StatusBadge>}
                  </div>
                </td>
                <td className="px-8 py-6">
                  <div className="flex flex-col items-center gap-1.5">
                    <div className="flex items-center gap-2 text-slate-500 dark:text-slate-400 text-[10px] font-mono font-bold uppercase bg-slate-50 dark:bg-slate-800 px-3 py-1.5 rounded-lg border border-slate-100 dark:border-slate-800 shadow-inner">
                      <RefreshCw size={12} className={cn(isFetching && "animate-spin")} />
                      {agent.last_seen_at ? new Date(agent.last_seen_at).toLocaleString() : t("common.never")}
                    </div>
                  </div>
                </td>
                <td className="px-10 py-6 text-right">
                  <div className="flex items-center justify-end gap-3 opacity-0 group-hover:opacity-100 transition-all duration-300">
                    {agent.is_active ? (
                      <>
                        <button onClick={() => handleEdit(agent)} className="p-3 bg-white dark:bg-slate-900 text-slate-500 dark:text-slate-400 hover:text-primary-600 dark:text-primary-400 rounded-xl shadow-md border border-slate-100 dark:border-slate-800 transition-all active:scale-90" title={t("common.edit")}><Edit2 size={18} /></button>
                        <button onClick={() => handleToggleTrust(agent)} className={cn("px-5 py-2.5 rounded-xl text-[9px] font-bold uppercase tracking-[0.15em] transition-all border shadow-sm active:scale-95", agent.is_trusted ? "text-rose-600 dark:text-rose-300 hover:bg-rose-50 dark:bg-rose-950 border-rose-100 dark:border-rose-800 bg-white dark:bg-slate-900" : "bg-primary-600 text-white hover:bg-primary-700 border-transparent shadow-primary-100")}>{agent.is_trusted ? t("common.revoke").toUpperCase() : t("common.approve").toUpperCase()}</button>
                        <button onClick={() => handleDelete(agent)} className="p-3 bg-white dark:bg-slate-900 text-slate-500 dark:text-slate-400 hover:text-rose-600 dark:text-rose-300 rounded-xl shadow-md border border-slate-100 dark:border-slate-800 transition-all active:scale-90" title={t("common.delete")}><Trash2 size={18} /></button>
                      </>
                    ) : (
                      <button onClick={() => handleRestore(agent)} className="flex items-center gap-2 px-6 py-3 bg-emerald-600 text-white text-[9px] font-bold uppercase tracking-widest rounded-xl hover:bg-emerald-700 transition-all shadow-lg shadow-emerald-100 active:scale-95"><RotateCcw size={14} /> {t("common.restore").toUpperCase()}</button>
                    )}
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      <AgentEditModal key={editingAgent?.id} agent={editingAgent} isOpen={isEditModalOpen} onClose={() => setIsEditModalOpen(false)} onSave={handleSave} />
    </div>
  );
}