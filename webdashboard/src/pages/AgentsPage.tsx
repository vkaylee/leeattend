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
    } catch (e) { toast.error(t("agents.messages.restore_error")); }
  }

  async function handleAdd(e: React.FormEvent) {
    e.preventDefault();
    try {
      await createAgent(newAgent);
      toast.success(t("agents.messages.add_success"));
      setShowAddForm(false);
      setNewAgent({ id: "", name: "", token: "" });
    } catch (e) { toast.error(t("agents.messages.add_error")); }
  }

  async function handleToggleTrust(agent: AuthorizedAgent) {
    const action = agent.is_trusted ? t("common.revoke") : t("common.approve");
    if (!confirm(t("agents.messages.confirm_trust", { action, id: agent.id }))) return;
    try {
      await updateTrust({ id: agent.id, is_trusted: !agent.is_trusted });
      toast.success(t("agents.messages.trust_success", { action }));
    } catch (e) { toast.error(t("common.error")); }
  }

  async function handleDelete(agent: AuthorizedAgent) {
    if (!confirm(t("agents.messages.confirm_delete", { name: agent.name, id: agent.id }))) return;
    try {
      await deleteAgent(agent.id);
      toast.success(t("agents.messages.delete_success", { name: agent.name }));
    } catch (e) { toast.error(t("agents.messages.delete_error")); }
  }

  function handleEdit(agent: AuthorizedAgent) { setEditingAgent(agent); setIsEditModalOpen(true); }
  function handleSave(updated: AuthorizedAgent) { setEditingAgent(updated); }

  const loading = isLoading || isFetching;

  return (
    <div className="space-y-8 animate-in fade-in duration-500">
      {/* Header */}
      <div className="flex justify-between items-center px-2">
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-2xl bg-primary-50 flex items-center justify-center text-primary-600 shadow-sm border border-primary-100/50">
            <Share2 size={20} />
          </div>
          <div>
            <h3 className="text-lg font-bold text-slate-800 tracking-tight">{t("agents.title")}</h3>
            <p className="text-xs text-slate-500 font-medium">{t("agents.subtitle")}</p>
          </div>
        </div>
        <div className="flex gap-3">
          <button onClick={() => setIncludeInactive(!includeInactive)} className={cn("flex items-center gap-2 px-5 py-2.5 rounded-2xl transition-all font-black text-[10px] uppercase tracking-widest border", includeInactive ? "bg-slate-100 border-slate-300 text-slate-900" : "bg-white border-slate-200 text-slate-600 hover:bg-slate-50")}>
            {includeInactive ? <Eye size={14} /> : <EyeOff size={14} />} {includeInactive ? t("agents.actions.showing_archived") : t("agents.actions.show_archived")}
          </button>
          <button onClick={() => refetch()} className="bg-white p-2.5 rounded-2xl border border-slate-200 hover:bg-slate-50 transition-colors text-slate-600 shadow-sm">
            <RefreshCcw className={cn("w-4 h-4", loading && "animate-spin")} />
          </button>
          <button onClick={() => setShowAddForm(!showAddForm)} className="flex items-center gap-2 bg-primary-600 text-white px-6 py-2.5 rounded-2xl hover:bg-primary-700 transition-all shadow-lg shadow-primary-100 font-black text-[10px] uppercase tracking-widest active:scale-95">
            <Plus size={14} /> {t("agents.actions.declare_agent")}
          </button>
        </div>
      </div>

      {showAddForm && (
        <div className="bg-white p-8 rounded-[32px] border-2 border-primary-100 shadow-xl shadow-primary-50/50 animate-in slide-in-from-top-4 duration-300">
          <div className="flex items-center gap-2 mb-6 text-primary-700">
            <Plus className="w-4 h-4" />
            <h4 className="text-xs font-black uppercase tracking-widest">{t("agents.form.new_title")}</h4>
          </div>
          <form onSubmit={handleAdd} className="grid grid-cols-1 md:grid-cols-3 gap-6">
            <div className="space-y-2">
              <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("agents.form.id_label")}</label>
              <input required placeholder={t("agents.form.id_placeholder")} className="w-full bg-slate-50 border border-slate-100 rounded-xl px-4 py-3 text-sm focus:ring-2 focus:ring-primary-500 outline-none transition-all font-bold" value={newAgent.id} onChange={(e) => setNewAgent({ ...newAgent, id: e.target.value })} />
            </div>
            <div className="space-y-2">
              <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("agents.form.name_label")}</label>
              <input required placeholder={t("agents.form.name_placeholder")} className="w-full bg-slate-50 border border-slate-100 rounded-xl px-4 py-3 text-sm focus:ring-2 focus:ring-primary-500 outline-none transition-all font-bold" value={newAgent.name} onChange={(e) => setNewAgent({ ...newAgent, name: e.target.value })} />
            </div>
            <div className="flex items-end gap-4">
              <div className="space-y-2 flex-1">
                <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("agents.form.token_label")}</label>
                <div className="relative group">
                  <input required placeholder={t("agents.form.token_placeholder")} className="w-full bg-slate-50 border border-slate-100 rounded-xl px-4 py-3 text-sm focus:ring-2 focus:ring-primary-500 outline-none transition-all pr-10 font-bold" value={newAgent.token} onChange={(e) => setNewAgent({ ...newAgent, token: e.target.value })} />
                  <Key className="absolute right-3 top-3.5 w-4 h-4 text-slate-300 group-focus-within:text-primary-500 transition-colors" />
                </div>
              </div>
              <button type="submit" className="bg-primary-600 text-white px-8 py-3 rounded-xl font-black text-[10px] uppercase tracking-widest hover:bg-primary-700 transition-all shadow-lg shadow-primary-100 h-[46px]">{t("common.save")}</button>
            </div>
          </form>
        </div>
      )}

      <div className="bg-white rounded-[32px] border border-slate-100 overflow-hidden shadow-sm">
        <table className="w-full text-left border-collapse">
          <thead>
            <tr className="bg-slate-50/50 border-b border-slate-100">
              <th className="px-6 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("agents.table.identifier")}</th>
              <th className="px-6 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("agents.table.last_ip")}</th>
              <th className="px-6 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("common.status")}</th>
              <th className="px-6 py-4 text-right"></th>
            </tr>
          </thead>
          <tbody className="divide-y divide-slate-50">
            {isLoading ? (
              <tr><td colSpan={4} className="px-6 py-12 text-center text-slate-400 font-bold italic">{t("common.loading")}</td></tr>
            ) : agents.length === 0 ? (
              <tr><td colSpan={4} className="px-6 py-20 text-center text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50">{t("agents.messages.no_agents")}</td></tr>
            ) : agents.map((agent) => (
              <tr key={agent.id} className={cn("hover:bg-slate-50/50 transition-colors group", !agent.is_active && "opacity-60 bg-slate-50/30")}>
                <td className="px-6 py-5">
                  <div className="flex items-center gap-4">
                    <div className={cn("w-12 h-12 rounded-2xl flex items-center justify-center transition-all shadow-inner border", agent.is_active ? "bg-slate-50 text-slate-400 border-slate-100 group-hover:bg-white group-hover:text-primary-600 group-hover:border-primary-100 group-hover:shadow-md" : "bg-slate-100 text-slate-300 border-slate-200")}>
                      <Monitor size={24} />
                    </div>
                    <div>
                      <div className="flex items-center gap-2">
                        <p className={cn("font-black text-slate-900 text-sm tracking-tight uppercase", !agent.is_active && "text-slate-500 line-through")}>{agent.name}</p>
                        {!agent.is_active && <span className="text-[8px] bg-slate-200 text-slate-600 px-1.5 py-0.5 rounded font-black uppercase tracking-tighter">{t("agents.status.closed")}</span>}
                      </div>
                      <p className="text-[10px] font-mono font-bold text-slate-400 uppercase tracking-tighter">{agent.id}</p>
                    </div>
                  </div>
                </td>
                <td className="px-6 py-5">
                  <div className="flex items-center gap-2 text-slate-500 font-bold text-xs">
                    <Globe className="w-3.5 h-3.5 text-slate-300" />
                    {agent.last_ip || "---"}
                  </div>
                </td>
                <td className="px-6 py-5">
                  <div className="flex flex-col gap-2">
                    <span className={cn("flex items-center gap-1.5 font-black text-[9px] px-2.5 py-1 rounded-full w-fit uppercase tracking-tighter border", isAgentOnline(agent.last_seen_at) ? "bg-emerald-50 text-emerald-600 border-emerald-100 animate-pulse" : "bg-slate-100 text-slate-400 border-slate-200")}>
                      <Activity className="w-3 h-3" /> {isAgentOnline(agent.last_seen_at) ? t("common.online") : t("common.offline")}
                    </span>
                    <span className={cn("flex items-center gap-1.5 font-black text-[9px] px-2.5 py-1 rounded-full w-fit uppercase tracking-tighter border", agent.is_trusted ? "bg-blue-50 text-blue-600 border-blue-100" : "bg-amber-50 text-amber-600 border-amber-100")}>
                      {agent.is_trusted ? <ShieldCheck className="w-3 h-3" /> : <ShieldAlert className="w-3 h-3" />} {agent.is_trusted ? t("agents.status.trusted") : t("agents.status.pending")}
                    </span>
                  </div>
                </td>
                <td className="px-6 py-5 text-right">
                  <div className="flex items-center justify-end gap-2 opacity-0 group-hover:opacity-100 transition-all">
                    {agent.is_active ? (
                      <>
                        <button onClick={() => handleEdit(agent)} className="p-2.5 bg-white text-slate-400 hover:text-primary-600 rounded-xl shadow-md border border-slate-100 transition-all"><Edit2 size={16} /></button>
                        <button onClick={() => handleToggleTrust(agent)} className={cn("px-4 py-2 rounded-xl text-[9px] font-black uppercase tracking-widest transition-all border shadow-sm active:scale-95", agent.is_trusted ? "text-rose-600 hover:bg-rose-50 border-rose-100 bg-white" : "bg-primary-600 text-white hover:bg-primary-700 border-transparent shadow-primary-100")}>{agent.is_trusted ? t("common.revoke") : t("common.trust")}</button>
                        <button onClick={() => handleDelete(agent)} className="p-2.5 bg-white text-slate-400 hover:text-rose-600 rounded-xl shadow-md border border-slate-100 transition-all"><Trash2 size={16} /></button>
                      </>
                    ) : (
                      <button onClick={() => handleRestore(agent)} className="flex items-center gap-2 px-5 py-2 bg-emerald-600 text-white text-[9px] font-black uppercase tracking-widest rounded-xl hover:bg-emerald-700 transition-all shadow-lg shadow-emerald-100"><RotateCcw size={12} /> {t("common.restore")}</button>
                    )}
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      <AgentEditModal agent={editingAgent} isOpen={isEditModalOpen} onClose={() => setIsEditModalOpen(false)} onSave={handleSave} />
    </div>
  );
}
      <AgentEditModal
        agent={editingAgent}
        isOpen={isEditModalOpen}
        onClose={() => setIsEditModalOpen(false)}
        onSave={handleSave}
      />
    </div>
  );
}