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
    } catch (e) { }
  }

  async function handleAdd(e: React.FormEvent) {
    e.preventDefault();
    try {
      await createAgent(newAgent);
      toast.success(t("agents.messages.add_success"));
      setShowAddForm(false); setNewAgent({ id: "", name: "", token: "" });
    } catch (e) { }
  }

  async function handleToggleTrust(agent: AuthorizedAgent) {
    const action = agent.is_trusted ? t("common.revoke") : t("common.approve");
    if (!confirm(t("agents.messages.confirm_trust", { action, id: agent.id }))) return;
    try {
      await updateTrust({ id: agent.id, is_trusted: !agent.is_trusted });
      toast.success(t("agents.messages.trust_success", { action }));
    } catch (e) { }
  }

  async function handleDelete(agent: AuthorizedAgent) {
    if (!confirm(t("agents.messages.confirm_delete", { name: agent.name, id: agent.id }))) return;
    try {
      await deleteAgent(agent.id);
      toast.success(t("agents.messages.delete_success", { name: agent.name }));
    } catch (e) { }
  }

  function handleEdit(agent: AuthorizedAgent) { setEditingAgent(agent); setIsEditModalOpen(true); }
  function handleSave(updated: AuthorizedAgent) { setEditingAgent(updated); }

  const loading = isLoading || isFetching;

  return (
    <div className="space-y-10 animate-in fade-in duration-700">
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-6 px-2">
        <div className="flex items-center gap-4">
          <div className="w-12 h-12 rounded-[1.25rem] bg-primary-50 flex items-center justify-center text-primary-600 shadow-sm border border-primary-100/50"><Share2 size={24} /></div>
          <div>
            <h3 className="text-2xl font-black tracking-tight text-slate-900 uppercase leading-none">{t("agents.title")}</h3>
            <p className="text-[10px] font-black text-slate-400 uppercase tracking-widest mt-1.5 opacity-70">{t("agents.subtitle")}</p>
          </div>
        </div>
        <div className="flex items-center gap-4">
          <button onClick={() => setIncludeInactive(!includeInactive)} className={cn("flex items-center gap-2 px-6 py-2.5 rounded-2xl transition-all font-black text-[10px] uppercase tracking-widest border shadow-sm active:scale-95", includeInactive ? "bg-slate-900 border-slate-900 text-white" : "bg-white border-slate-200 text-slate-500 hover:bg-slate-50")}>
            {includeInactive ? <Eye size={14} /> : <EyeOff size={14} />} {includeInactive ? t("agents.actions.showing_archived").toUpperCase() : t("agents.actions.show_archived").toUpperCase()}
          </button>
          <button onClick={() => refetch()} className="bg-white p-3 rounded-2xl border border-slate-200 hover:bg-slate-50 transition-all text-slate-400 shadow-sm active:rotate-180 duration-500">
            <RefreshCcw className={cn("w-4 h-4", loading && "animate-spin text-primary-600")} />
          </button>
          <button onClick={() => setShowAddForm(!showAddForm)} className="flex items-center gap-3 bg-primary-600 text-white px-8 py-3 rounded-2xl font-black text-xs uppercase tracking-widest hover:bg-primary-700 transition-all shadow-xl shadow-primary-100 active:scale-95">
            <Plus size={18} /> {t("agents.actions.declare_agent").toUpperCase()}
          </button>
        </div>
      </div>

      {showAddForm && (
        <div className="bg-white p-10 rounded-[2.5rem] border-2 border-primary-100 shadow-xl shadow-primary-50/50 animate-in slide-in-from-top-4 duration-500 space-y-8">
          <div className="flex items-center gap-3 text-primary-700">
            <div className="w-8 h-8 rounded-xl bg-primary-50 flex items-center justify-center shadow-inner"><Plus size={16} /></div>
            <h4 className="text-[10px] font-black uppercase tracking-[0.2em]">{t("agents.form.new_title")}</h4>
          </div>
          <form onSubmit={handleAdd} className="grid grid-cols-1 md:grid-cols-3 gap-8">
            <div className="space-y-3 px-1">
              <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("agents.form.id_label")}</label>
              <input required placeholder={t("agents.form.id_placeholder")} className="w-full bg-slate-50 border border-slate-100 rounded-2xl px-5 py-4 focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all font-black text-sm uppercase tracking-tight shadow-sm" value={newAgent.id} onChange={(e) => setNewAgent({ ...newAgent, id: e.target.value })} />
            </div>
            <div className="space-y-3 px-1">
              <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("agents.form.name_label")}</label>
              <input required placeholder={t("agents.form.name_placeholder")} className="w-full bg-slate-50 border border-slate-100 rounded-2xl px-5 py-4 focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all font-black text-sm uppercase tracking-tight shadow-sm" value={newAgent.name} onChange={(e) => setNewAgent({ ...newAgent, name: e.target.value })} />
            </div>
            <div className="flex items-end gap-6">
              <div className="space-y-3 flex-1 px-1">
                <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("agents.form.token_label")}</label>
                <div className="relative group">
                  <input required placeholder={t("agents.form.token_placeholder")} className="w-full bg-slate-50 border border-slate-100 rounded-2xl px-5 py-4 focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all pr-14 font-mono font-bold text-sm tracking-tight shadow-sm" value={newAgent.token} onChange={(e) => setNewAgent({ ...newAgent, token: e.target.value })} />
                  <Key className="absolute right-5 top-1/2 -translate-y-1/2 w-5 h-5 text-slate-300 group-focus-within:text-primary-500 transition-colors" />
                </div>
              </div>
              <button type="submit" className="bg-slate-900 text-white px-10 py-4 rounded-2xl font-black text-xs uppercase tracking-widest hover:bg-black transition-all shadow-xl shadow-slate-200 h-[56px] active:scale-95">{t("common.save").toUpperCase()}</button>
            </div>
          </form>
        </div>
      )}

      <div className="bg-white rounded-[2.5rem] border border-slate-100 overflow-hidden shadow-sm">
        <table className="w-full text-left border-collapse">
          <thead>
            <tr className="bg-slate-50/50 border-b border-slate-100">
              <th className="px-8 py-5 text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("agents.table.identifier")}</th>
              <th className="px-8 py-5 text-[10px] font-black text-slate-400 uppercase tracking-widest text-center">{t("agents.table.last_ip")}</th>
              <th className="px-8 py-5 text-[10px] font-black text-slate-400 uppercase tracking-widest text-center">{t("common.status")}</th>
              <th className="px-8 py-5 border-b border-slate-100"></th>
            </tr>
          </thead>
          <tbody className="divide-y divide-slate-50">
            {isLoading ? (
              <tr><td colSpan={4} className="px-8 py-12 text-center text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50">{t("common.loading")}</td></tr>
            ) : agents.length === 0 ? (
              <tr><td colSpan={4} className="px-8 py-32 text-center bg-slate-50/30 flex flex-col items-center"><Monitor size={40} className="text-slate-200 mb-4" /><p className="text-[10px] font-black text-slate-300 uppercase tracking-widest">{t("agents.messages.no_agents")}</p></td></tr>
            ) : agents.map((agent) => (
              <tr key={agent.id} className={cn("hover:bg-slate-50/50 transition-all group", !agent.is_active && "opacity-60 bg-slate-50/30")}>
                <td className="px-8 py-6">
                  <div className="flex items-center gap-5">
                    <div className={cn("w-14 h-14 rounded-2xl flex items-center justify-center transition-all shadow-inner border", agent.is_active ? "bg-slate-50 text-slate-400 border-slate-100 group-hover:bg-white group-hover:text-primary-600 group-hover:border-primary-100 group-hover:shadow-md" : "bg-slate-100 text-slate-300 border-slate-200")}>
                      <Monitor size={28} />
                    </div>
                    <div>
                      <div className="flex items-center gap-3">
                        <p className={cn("font-black text-slate-900 text-sm tracking-tight uppercase leading-none", !agent.is_active && "text-slate-500 line-through")}>{agent.name}</p>
                        {!agent.is_active && <span className="text-[8px] bg-slate-200 text-slate-600 px-2 py-0.5 rounded-full font-black uppercase tracking-widest">{t("agents.status.closed")}</span>}
                      </div>
                      <p className="text-[10px] font-mono font-bold text-slate-400 uppercase tracking-tighter mt-1.5 opacity-70">{agent.id}</p>
                    </div>
                  </div>
                </td>
                <td className="px-8 py-6 text-center">
                  <div className="inline-flex items-center gap-2 px-3 py-1.5 bg-slate-50 rounded-lg border border-slate-100 shadow-inner text-slate-500 font-mono font-bold text-[11px] uppercase tracking-tighter">
                    <Globe size={12} className="text-slate-300" />
                    {agent.last_ip || "---.---.---.---"}
                  </div>
                </td>
                <td className="px-8 py-6 text-center">
                  <div className="flex flex-col items-center gap-2">
                    <span className={cn("flex items-center gap-1.5 font-black text-[9px] px-3 py-1 rounded-full uppercase tracking-widest border shadow-sm", isAgentOnline(agent.last_seen_at) ? "bg-emerald-50 text-emerald-600 border-emerald-100" : "bg-slate-50 text-slate-400 border-slate-100 opacity-60")}>
                      <div className={cn("w-1.5 h-1.5 rounded-full", isAgentOnline(agent.last_seen_at) ? "bg-emerald-500 animate-pulse" : "bg-slate-300")} /> {isAgentOnline(agent.last_seen_at) ? t("common.online").toUpperCase() : t("common.offline").toUpperCase()}
                    </span>
                    <span className={cn("flex items-center gap-1.5 font-black text-[9px] px-3 py-1 rounded-full uppercase tracking-widest border shadow-sm", agent.is_trusted ? "bg-blue-50 text-blue-600 border-blue-100" : "bg-amber-50 text-amber-600 border-amber-100")}>
                      {agent.is_trusted ? <ShieldCheck size={12} /> : <ShieldAlert size={12} />} {agent.is_trusted ? t("agents.status.trusted").toUpperCase() : t("agents.status.pending").toUpperCase()}
                    </span>
                  </div>
                </td>
                <td className="px-8 py-6 text-right">
                  <div className="flex items-center justify-end gap-3 opacity-0 group-hover:opacity-100 transition-all duration-300">
                    {agent.is_active ? (
                      <>
                        <button onClick={() => handleEdit(agent)} className="p-2.5 bg-white text-slate-400 hover:text-primary-600 rounded-xl shadow-md border border-slate-100 transition-all active:scale-90" title={t("common.edit")}><Edit2 size={18} /></button>
                        <button onClick={() => handleToggleTrust(agent)} className={cn("px-5 py-2.5 rounded-xl text-[9px] font-black uppercase tracking-[0.15em] transition-all border shadow-sm active:scale-95", agent.is_trusted ? "text-rose-600 hover:bg-rose-50 border-rose-100 bg-white" : "bg-primary-600 text-white hover:bg-primary-700 border-transparent shadow-primary-100")}>{agent.is_trusted ? t("common.revoke").toUpperCase() : t("common.approve").toUpperCase()}</button>
                        <button onClick={() => handleDelete(agent)} className="p-2.5 bg-white text-slate-400 hover:text-rose-600 rounded-xl shadow-md border border-slate-100 transition-all active:scale-90" title={t("common.delete")}><Trash2 size={18} /></button>
                      </>
                    ) : (
                      <button onClick={() => handleRestore(agent)} className="flex items-center gap-2 px-6 py-2.5 bg-emerald-600 text-white text-[9px] font-black uppercase tracking-widest rounded-xl hover:bg-emerald-700 transition-all shadow-lg shadow-emerald-100 active:scale-95"><RotateCcw size={14} /> {t("common.restore").toUpperCase()}</button>
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