import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { AuthorizedAgent } from "@/lib/api";
import { isAgentOnline } from "@/lib/utils";
import {
  Plus,
  ShieldCheck,
  ShieldAlert,
  ShieldX,
  Key,
  UserCheck,
  Globe,
  RefreshCcw,
  Edit2,
  Trash2,
  Activity,
  Eye,
  EyeOff,
  RotateCcw,
} from "lucide-react";
import AgentEditModal from "@/components/AgentEditModal";
import { useAuthorizedAgents } from "@/hooks/use-agents";
import { useToast } from "../components/Toast";

export default function AgentsPage() {
  const { t } = useTranslation();
  const toast = useToast();
  // Add a local timer to refresh the UI every 15s to update online status
  const [, setTick] = useState(0);
  useEffect(() => {
    const interval = setInterval(() => setTick((t) => t + 1), 15000);
    return () => clearInterval(interval);
  }, []);

  const {
    agents,
    isLoading,
    isFetching,
    refetch,
    createAgent,
    updateTrust,
    deleteAgent,
    updateAgent,
    includeInactive,
    setIncludeInactive,
  } = useAuthorizedAgents();

  const [showAddForm, setShowAddForm] = useState(false);
  const [newAgent, setNewAgent] = useState({ id: "", name: "", token: "" });

  // State for editing
  const [editingAgent, setEditingAgent] = useState<AuthorizedAgent | null>(
    null,
  );
  const [isEditModalOpen, setIsEditModalOpen] = useState(false);

  async function handleRestore(agent: AuthorizedAgent) {
    try {
      await updateAgent({ id: agent.id, agent: { is_active: true } });
      toast.success(t("agents.messages.restore_success", { name: agent.name }));
    } catch (e) {
      toast.error(t("agents.messages.restore_error"));
    }
  }

  async function handleAdd(e: React.FormEvent) {
    e.preventDefault();
    try {
      await createAgent(newAgent);
      toast.success(t("agents.messages.add_success"));
      setShowAddForm(false);
      setNewAgent({ id: "", name: "", token: "" });
    } catch (e) {
      toast.error(t("agents.messages.add_error"), t("common.system_error"));
    }
  }

  async function handleToggleTrust(agent: AuthorizedAgent) {
    const action = agent.is_trusted ? t("common.revoke") : t("common.approve");
    if (!confirm(t("agents.messages.confirm_trust", { action, id: agent.id })))
      return;

    try {
      await updateTrust({ id: agent.id, is_trusted: !agent.is_trusted });
      toast.success(t("agents.messages.trust_success", { action }));
    } catch (e) {
      toast.error(t("agents.messages.update_status_error"));
    }
  }

  async function handleDelete(agent: AuthorizedAgent) {
    if (
      !confirm(
        t("agents.messages.confirm_delete", { name: agent.name, id: agent.id }),
      )
    )
      return;

    try {
      await deleteAgent(agent.id);
      toast.success(t("agents.messages.delete_success", { name: agent.name }));
    } catch (e) {
      toast.error(t("agents.messages.delete_error"));
    }
  }

  function handleEdit(agent: AuthorizedAgent) {
    setEditingAgent(agent);
    setIsEditModalOpen(true);
  }

  function handleSave(updated: AuthorizedAgent) {
    setEditingAgent(updated);
  }

  const loading = isLoading || isFetching;

  return (
    <div className="p-8">
      <div className="max-w-6xl mx-auto">
        <div className="mb-8 flex items-center justify-between">
          <div>
            <h1 className="text-3xl font-bold text-slate-900 tracking-tight">
              {t("agents.title")}
            </h1>
            <p className="text-slate-500 mt-1">
              {t("agents.subtitle")}
            </p>
          </div>
          <div className="flex gap-3">
            <button
              onClick={() => setIncludeInactive(!includeInactive)}
              data-testid="toggle-archived-agents-btn"
              className={`px-4 py-2.5 rounded-2xl border font-bold flex items-center gap-2 transition-all ${includeInactive
                  ? "bg-slate-100 border-slate-300 text-slate-900 shadow-inner"
                  : "bg-white border-slate-200 text-slate-600 hover:bg-slate-50"
              }`}
              title={
                includeInactive ? t("agents.actions.hide_inactive") : t("agents.actions.show_inactive")
              }
            >
              {includeInactive ? (
                <Eye className="w-5 h-5" />
              ) : (
                <EyeOff className="w-5 h-5" />
              )}
              {includeInactive ? t("agents.actions.showing_archived") : t("agents.actions.show_archived")}
            </button>
            <button
              onClick={() => refetch()}
              data-testid="refresh-agents-btn"
              className="bg-white p-2.5 rounded-2xl border border-slate-200 hover:bg-slate-50 transition-colors text-slate-600"
            >
              <RefreshCcw
                className={`w-5 h-5 ${loading ? "animate-spin" : ""}`}
              />
            </button>
            <button
              onClick={() => setShowAddForm(!showAddForm)}
              data-testid="add-agent-btn"
              className="bg-slate-900 text-white px-6 py-2.5 rounded-2xl font-bold flex items-center gap-2 hover:bg-black transition-all shadow-lg shadow-slate-200"
            >
              <Plus className="w-5 h-5" />
              {t("agents.actions.declare_agent")}
            </button>
          </div>
        </div>

        {showAddForm && (
          <div className="mb-8 bg-white p-8 rounded-3xl border border-slate-100 shadow-xl animate-in fade-in slide-in-from-top-4 duration-300">
            <h2 className="text-xl font-bold mb-6 flex items-center gap-2 text-slate-800">
              <Plus className="w-5 h-5 text-blue-500" />
              {t("agents.form.new_title")}
            </h2>
            <form
              onSubmit={handleAdd}
              className="grid grid-cols-1 md:grid-cols-3 gap-6"
            >
              <div className="space-y-2">
                <label className="text-xs font-bold text-slate-400 uppercase tracking-widest ml-1">
                  {t("agents.form.id_label")}
                </label>
                <input
                  required
                  data-testid="agent-id-input"
                  placeholder={t("agents.form.id_placeholder")}
                  className="w-full bg-slate-50 border border-slate-200 rounded-xl px-4 py-3 focus:ring-2 focus:ring-primary-500 outline-none transition-all"
                  value={newAgent.id}
                  onChange={(e) =>
                    setNewAgent({ ...newAgent, id: e.target.value })
                  }
                />
              </div>
              <div className="space-y-2">
                <label className="text-xs font-bold text-slate-400 uppercase tracking-widest ml-1">
                  {t("agents.form.name_label")}
                </label>
                <input
                  required
                  data-testid="agent-name-input"
                  placeholder={t("agents.form.name_placeholder")}
                  className="w-full bg-slate-50 border border-slate-200 rounded-xl px-4 py-3 focus:ring-2 focus:ring-primary-500 outline-none transition-all"
                  value={newAgent.name}
                  onChange={(e) =>
                    setNewAgent({ ...newAgent, name: e.target.value })
                  }
                />
              </div>
              <div className="flex items-end gap-4">
                <div className="space-y-2 flex-1">
                  <label className="text-xs font-bold text-slate-400 uppercase tracking-widest ml-1">
                    {t("agents.form.token_label")}
                  </label>
                  <div className="relative">
                    <input
                      required
                      data-testid="agent-token-input"
                      placeholder={t("agents.form.token_placeholder")}
                      className="w-full bg-slate-50 border border-slate-200 rounded-xl px-4 py-3 focus:ring-2 focus:ring-primary-500 outline-none transition-all pr-10"
                      value={newAgent.token}
                      onChange={(e) =>
                        setNewAgent({ ...newAgent, token: e.target.value })
                      }
                    />
                    <Key className="absolute right-3 top-3.5 w-4 h-4 text-slate-300" />
                  </div>
                </div>
                <button
                  type="submit"
                  data-testid="save-new-agent-btn"
                  className="bg-primary-600 text-white px-8 py-3 rounded-xl font-bold hover:bg-primary-700 transition-all shadow-lg shadow-primary-100"
                >
                  {t("common.save")}
                </button>
              </div>
            </form>
          </div>
        )}

        <div className="bg-white rounded-3xl border border-slate-100 overflow-hidden shadow-sm">
          <table className="w-full text-left">
            <thead>
              <tr className="bg-slate-50 border-b border-slate-100">
                <th className="px-6 py-4 text-xs font-bold text-slate-400 uppercase tracking-widest">
                  {t("agents.table.identifier")}
                </th>
                <th className="px-6 py-4 text-xs font-bold text-slate-400 uppercase tracking-widest">
                  {t("agents.table.token")}
                </th>
                <th className="px-6 py-4 text-xs font-bold text-slate-400 uppercase tracking-widest">
                  {t("agents.table.last_ip")}
                </th>
                <th className="px-6 py-4 text-xs font-bold text-slate-400 uppercase tracking-widest">
                  {t("common.status")}
                </th>
                <th className="px-6 py-4 text-xs font-bold text-slate-400 uppercase tracking-widest text-right">
                  {t("common.actions")}
                </th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-50">
              {agents.map((agent) => (
                <tr
                  key={agent.id}
                  className={`hover:bg-slate-50/50 transition-colors group ${!agent.is_active ? "opacity-60 bg-slate-50/30" : ""}`}
                >
                  <td className="px-6 py-5">
                    <div className="flex items-center gap-3">
                      <div
                        className={`p-2 rounded-xl transition-all ${agent.is_active
                            ? "bg-slate-100 text-slate-500 group-hover:bg-white group-hover:shadow-sm"
                            : "bg-slate-200 text-slate-400"
                        }`}
                      >
                        <UserCheck className="w-5 h-5" />
                      </div>
                      <div>
                        <p
                          className={`font-bold ${agent.is_active ? "text-slate-900" : "text-slate-500"}`}
                        >
                          {agent.name}
                          {!agent.is_active && (
                            <span className="ml-2 text-[10px] bg-slate-200 text-slate-600 px-1.5 py-0.5 rounded uppercase tracking-tighter">
                              {t("agents.status.closed")}
                            </span>
                          )}
                        </p>
                        <p className="text-xs font-mono text-slate-400">
                          {agent.id}
                        </p>
                      </div>
                    </div>
                  </td>
                  <td className="px-6 py-5">
                    <div className="flex items-center gap-2 text-slate-400 font-mono text-xs bg-slate-50 px-2 py-1 rounded-lg w-fit">
                      <Key className="w-3 h-3" />
                      {agent.token.replace(/./g, "•").slice(0, 12)}
                    </div>
                  </td>
                  <td className="px-6 py-5">
                    <div className="flex items-center gap-2 text-slate-500 text-sm">
                      <Globe className="w-4 h-4 text-slate-300" />
                      {agent.last_ip || "---"}
                    </div>
                  </td>
                  <td className="px-6 py-5">
                    <div className="flex flex-col gap-2">
                      {isAgentOnline(agent.last_seen_at) ? (
                        <span className="flex items-center gap-1.5 text-primary-600 font-bold text-[10px] bg-primary-50 px-3 py-1 rounded-full w-fit uppercase tracking-wider animate-pulse">
                          <Activity className="w-3 h-3" />
                          {t("common.online").toUpperCase()}
                        </span>
                      ) : (
                        <span className="flex items-center gap-1.5 text-slate-400 font-bold text-[10px] bg-slate-100 px-3 py-1 rounded-full w-fit uppercase tracking-wider">
                          <Activity className="w-3 h-3" />
                          {t("common.offline").toUpperCase()}
                        </span>
                      )}

                      {agent.is_trusted ? (
                        <span className="flex items-center gap-1.5 text-emerald-600 font-bold text-[10px] bg-emerald-50 px-3 py-1 rounded-full w-fit uppercase tracking-wider">
                          <ShieldCheck className="w-3 h-3" />
                          {t("agents.status.trusted").toUpperCase()}
                        </span>
                      ) : (
                        <span className="flex items-center gap-1.5 text-amber-600 font-bold text-[10px] bg-amber-50 px-3 py-1 rounded-full w-fit uppercase tracking-wider">
                          <ShieldAlert className="w-3 h-3" />
                          {t("agents.status.pending").toUpperCase()}
                        </span>
                      )}
                    </div>
                  </td>
                  <td className="px-6 py-5 text-right">
                    <div className="flex items-center justify-end gap-2">
                      {agent.is_active ? (
                        <>
                          <button
                            onClick={() => handleEdit(agent)}
                            className="p-2 text-slate-400 hover:text-primary-600 hover:bg-primary-50 rounded-xl transition-all"
                            title={t("common.edit")}
                          >
                            <Edit2 className="w-4 h-4" />
                          </button>

                          <button
                            onClick={() => handleToggleTrust(agent)}
                            className={`px-3 py-1.5 rounded-xl text-[10px] font-black uppercase tracking-widest transition-all border ${agent.is_trusted
                                ? "text-rose-600 hover:bg-rose-50 border-rose-100"
                                : "bg-primary-600 text-white hover:bg-primary-700 border-transparent shadow-md shadow-primary-100"
                            }`}
                          >
                            {agent.is_trusted ? t("common.revoke") : t("common.trust")}
                          </button>

                          <button
                            onClick={() => handleDelete(agent)}
                            className="p-2 text-slate-400 hover:text-rose-600 hover:bg-rose-50 rounded-xl transition-all"
                            title={t("common.delete")}
                          >
                            <Trash2 className="w-4 h-4" />
                          </button>
                        </>
                      ) : (
                        <button
                          onClick={() => handleRestore(agent)}
                          className="flex items-center gap-1 px-4 py-2 bg-emerald-600 text-white text-[10px] font-bold uppercase tracking-widest rounded-xl hover:bg-emerald-700 transition-all shadow-md shadow-emerald-100"
                        >
                          <RotateCcw className="w-3 h-3" />
                          {t("common.restore")}
                        </button>
                      )}
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>

          {agents.length === 0 && !loading && (
            <div className="py-20 text-center text-slate-400">
              <ShieldX className="w-12 h-12 mx-auto mb-4 text-slate-200" />
              <p className="font-medium">{t("agents.messages.no_agents")}</p>
            </div>
          )}
        </div>
      </div>

      <AgentEditModal
        agent={editingAgent}
        isOpen={isEditModalOpen}
        onClose={() => setIsEditModalOpen(false)}
        onSave={handleSave}
      />
    </div>
  );
}