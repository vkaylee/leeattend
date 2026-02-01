import { useState, useMemo } from "react";
import { useTranslation } from "react-i18next";
import { Plus, Share2, Server, HardDrive, AlertCircle, Cpu, Edit2 } from "lucide-react";
import { useAuthorizedAgents } from "@/hooks/use-agents";
import { useDevices } from "@/hooks/use-devices";
import { cn } from "@/lib/utils";
import { useToast } from "@/components/ui/Toast";
import { AuthorizedAgent, Device } from "@/lib/api";
import AgentAccordionCard from "./AgentAccordionCard";
import DeviceListItem from "./DeviceListItem";
import AgentEditModal from "@/components/features/agents/AgentEditModal";
import Modal from "@/components/ui/Modal";
import { Combobox } from "@/components/ui/Combobox";

export default function ConnectivityTopology() {
  const { t } = useTranslation();
  const toast = useToast();

  const {
    agents,
    isLoading: agentsLoading,
    refetch: refetchAgents,
    includeInactive,
    setIncludeInactive,
    createAgent,
    updateTrust,
    deleteAgent,
    updateAgent
  } = useAuthorizedAgents();

  const {
    devices,
    isLoading: devicesLoading,
    updateDevice,
    deleteDevice
  } = useDevices();

  const [expandedAgents, setExpandedAgents] = useState<Set<string>>(new Set());
  const [showAddForm, setShowAddForm] = useState(false);
  const [newAgent, setNewAgent] = useState({ id: "", name: "", token: "" });
  const [editingAgent, setEditingAgent] = useState<AuthorizedAgent | null>(null);
  const [editingDevice, setEditingDevice] = useState<Device | null>(null);

  // Group devices by agent_id
  const { groupedDevices, unassignedDevices } = useMemo(() => {
    const grouped: Record<string, Device[]> = {};
    const unassigned: Device[] = [];

    for (const device of devices) {
      const agentExists = agents.some(a => a.id === device.agent_id);
      if (agentExists) {
        if (!grouped[device.agent_id]) grouped[device.agent_id] = [];
        grouped[device.agent_id].push(device);
      } else {
        unassigned.push(device);
      }
    }

    return { groupedDevices: grouped, unassignedDevices: unassigned };
  }, [agents, devices]);

  const toggleExpanded = (agentId: string) => {
    setExpandedAgents(prev => {
      const next = new Set(prev);
      if (next.has(agentId)) next.delete(agentId);
      else next.add(agentId);
      return next;
    });
  };

  const expandAll = () => setExpandedAgents(new Set(agents.map(a => a.id)));
  const collapseAll = () => setExpandedAgents(new Set());

  async function handleAddAgent(e: React.FormEvent) {
    e.preventDefault();
    try {
      await createAgent(newAgent);
      toast.success(t("agents.messages.add_success", { name: newAgent.name }));
      setShowAddForm(false);
      setNewAgent({ id: "", name: "", token: "" });
    } catch (e) {
      console.error(e);
    }
  }

  async function handleToggleTrust(agent: AuthorizedAgent) {
    await updateTrust({ id: agent.id, is_trusted: !agent.is_trusted });
  }

  async function handleDeleteAgent(agent: AuthorizedAgent) {
    await deleteAgent(agent.id);
  }

  async function handleUpdateDevice(id: string, data: Partial<Device>) {
    await updateDevice(id, data);
  }

  async function handleDeleteDevice(id: string) {
    await deleteDevice(id);
  }

  const handleEditAgent = (agent: AuthorizedAgent) => setEditingAgent(agent);
  const handleSaveAgent = (updated: AuthorizedAgent) => {
    toast.success(t("agents.messages.update_success", { name: updated.name }));
    refetchAgents();
  };

  async function handleSaveDevice(e: React.FormEvent) {
    e.preventDefault();
    if (!editingDevice) return;
    try {
      await updateDevice(editingDevice.id, {
        name: editingDevice.name,
        ip: editingDevice.ip,
        port: editingDevice.port,
        brand: editingDevice.brand,
        agent_id: editingDevice.agent_id,
      });
      toast.success(t("devices.toast.update_success", { name: editingDevice.name }));
      setEditingDevice(null);
    } catch (e) {
      console.error(e);
      toast.error(t("devices.toast.update_error"));
    }
  }

  const brandOptions = [
    { value: "zkteco", label: "ZKTeco", icon: <Cpu size={14} /> },
    { value: "mock", label: "Mock (Dev)", icon: <Cpu size={14} /> }
  ];

  const isLoading = agentsLoading || devicesLoading;

  return (
    <div data-testid="connectivity-topology" className="space-y-8 animate-in fade-in duration-500">
      {/* Header */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-6 px-2">
        <div className="flex items-center gap-4">
          <div className="w-12 h-12 rounded-[1.25rem] bg-primary-50 flex items-center justify-center text-primary-600 shadow-sm border border-primary-100/50">
            <Share2 size={24} />
          </div>
          <div>
            <h1 className="text-3xl font-bold tracking-tight text-slate-700 leading-tight">
              {t("common.settings.categories.connectivity")}
            </h1>
            <p className="text-slate-500 mt-1 font-medium">
              {t("common.settings.categories.connectivity_desc")}
            </p>
          </div>
        </div>

        <div className="flex items-center gap-3">
          {agents.length > 0 && (
            <>
              <button
                data-testid="expand-all-btn"
                onClick={expandAll}
                className="px-4 py-2 text-xs font-bold uppercase tracking-widest text-slate-500 hover:text-primary-600 transition-colors"
              >
                {t("common.expand_all")}
              </button>
              <button
                data-testid="collapse-all-btn"
                onClick={collapseAll}
                className="px-4 py-2 text-xs font-bold uppercase tracking-widest text-slate-500 hover:text-primary-600 transition-colors"
              >
                {t("common.collapse_all")}
              </button>
              <div className="w-px h-6 bg-slate-200" />
            </>
          )}
          <button
            data-testid="toggle-inactive-btn"
            onClick={() => setIncludeInactive(!includeInactive)}
            className={cn(
              "flex items-center gap-2 px-6 py-3 rounded-2xl font-bold text-[10px] uppercase tracking-widest transition-all border shadow-sm active:scale-95",
              includeInactive
                ? "bg-slate-900 text-white border-transparent"
                : "bg-white text-slate-500 border-slate-200 hover:bg-slate-50"
            )}
          >
            {includeInactive ? t("common.status_inactive") : t("common.status_active")}
          </button>
          <button
            data-testid="add-agent-btn"
            onClick={() => setShowAddForm(true)}
            className="flex items-center gap-3 bg-primary-600 text-white px-8 py-3 rounded-2xl font-bold text-xs uppercase tracking-widest hover:bg-primary-700 transition-all shadow-xl shadow-primary-100 active:scale-95"
          >
            <Plus size={18} /> {t("agents.add_btn")}
          </button>
        </div>
      </div>

      {/* Add Agent Form */}
      {showAddForm && (
        <form data-testid="add-agent-form" onSubmit={handleAddAgent} className="bg-white border-2 border-primary-100 p-10 rounded-[2.5rem] shadow-xl shadow-primary-50/50 animate-in slide-in-from-top-4 duration-500">
          <div className="flex items-center gap-3 mb-8 text-primary-700">
            <div className="w-8 h-8 rounded-xl bg-primary-50 flex items-center justify-center shadow-inner"><Plus size={16} /></div>
            <h4 className="text-[10px] font-bold uppercase tracking-[0.2em]">{t("agents.add_form_title")}</h4>
          </div>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-8">
            <div className="space-y-3 px-1">
              <label className="text-[10px] font-bold text-slate-500 uppercase tracking-widest ml-1">{t("agents.id_label")}</label>
              <input data-testid="agent-id-input" required placeholder="agent-001" className="w-full bg-slate-50 border border-slate-100 rounded-2xl px-5 py-4 focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all font-mono font-bold text-sm uppercase tracking-tight" value={newAgent.id} onChange={(e) => setNewAgent({ ...newAgent, id: e.target.value })} />
            </div>
            <div className="space-y-3 px-1">
              <label className="text-[10px] font-bold text-slate-500 uppercase tracking-widest ml-1">{t("agents.name_label")}</label>
              <input data-testid="agent-name-input" required placeholder="Office Main" className="w-full bg-slate-50 border border-slate-100 rounded-2xl px-5 py-4 focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all font-bold text-sm uppercase tracking-tight" value={newAgent.name} onChange={(e) => setNewAgent({ ...newAgent, name: e.target.value })} />
            </div>
            <div className="space-y-3 px-1">
              <label className="text-[10px] font-bold text-slate-500 uppercase tracking-widest ml-1">{t("agents.token_label")}</label>
              <input data-testid="agent-token-input" required placeholder="Connection token" className="w-full bg-slate-50 border border-slate-100 rounded-2xl px-5 py-4 focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all font-mono font-bold text-sm" value={newAgent.token} onChange={(e) => setNewAgent({ ...newAgent, token: e.target.value })} />
            </div>
          </div>
          <div className="mt-10 flex justify-end gap-4">
            <button data-testid="agent-form-cancel" type="button" onClick={() => setShowAddForm(false)} className="px-8 py-4 rounded-2xl font-bold text-xs uppercase tracking-widest text-slate-500 hover:bg-slate-50 transition-all">{t("common.cancel")}</button>
            <button data-testid="agent-form-submit" type="submit" className="bg-primary-600 text-white px-12 py-4 rounded-2xl font-bold text-xs uppercase tracking-widest hover:bg-primary-700 transition-all shadow-xl shadow-primary-100 active:scale-95">{t("agents.save_btn")}</button>
          </div>
        </form>
      )}

      {/* Agent Topology */}
      {isLoading ? (
        <div className="space-y-4">
          {Array.from({ length: 3 }).map((_, i) => (
            <div key={i} className="bg-white border border-slate-100 p-8 rounded-[2.5rem] shadow-sm animate-pulse h-28" />
          ))}
        </div>
      ) : agents.length === 0 ? (
        <div data-testid="empty-agents-state" className="py-32 text-center border-2 border-dashed border-slate-100 rounded-[3rem] bg-slate-50/30 flex flex-col items-center animate-in zoom-in-95 duration-500">
          <div className="w-24 h-24 bg-white rounded-full flex items-center justify-center shadow-sm mb-6 border border-slate-100">
            <Server size={32} className="text-slate-200" />
          </div>
          <p className="text-slate-500 font-bold uppercase tracking-widest text-[10px] opacity-60 mb-2">
            {t("agents.table.no_agents")}
          </p>
          <p className="text-slate-400 text-xs max-w-md mb-8">
            {t("connectivity.empty_guide")}
          </p>
          <button
            data-testid="empty-add-agent-btn"
            onClick={() => setShowAddForm(true)}
            className="px-10 py-4 bg-white border border-slate-200 rounded-2xl font-bold text-[10px] uppercase tracking-[0.2em] text-slate-600 hover:border-primary-200 hover:text-primary-600 transition-all shadow-sm active:scale-95"
          >
            {t("agents.add_btn")}
          </button>
        </div>
      ) : (
        <div data-testid="agents-list" className="space-y-4">
          {agents.map(agent => (
            <AgentAccordionCard
              key={agent.id}
              agent={agent}
              devices={groupedDevices[agent.id] || []}
              isExpanded={expandedAgents.has(agent.id)}
              onToggleExpand={() => toggleExpanded(agent.id)}
              onEdit={handleEditAgent}
              onToggleTrust={handleToggleTrust}
              onDelete={handleDeleteAgent}
              onUpdateDevice={handleUpdateDevice}
              onDeleteDevice={handleDeleteDevice}
              onEditDevice={setEditingDevice}
            />
          ))}
        </div>
      )}

      {/* Unassigned Devices Section */}
      {unassignedDevices.length > 0 && (
        <div data-testid="unassigned-devices-section" className="mt-8 animate-in fade-in duration-500">
          <div className="flex items-center gap-3 mb-4 px-2">
            <AlertCircle size={16} className="text-amber-500" />
            <h3 className="text-xs font-bold uppercase tracking-widest text-slate-500">
              {t("devices.unassigned_title")} ({unassignedDevices.length})
            </h3>
          </div>
          <div data-testid="unassigned-devices-list" className="bg-amber-50/50 border border-amber-100 rounded-[2.5rem] p-6 space-y-3">
            {unassignedDevices.map(device => (
              <DeviceListItem
                key={device.id}
                device={device}
                parentAgentActive={false}
                onUpdate={handleUpdateDevice}
                onDelete={handleDeleteDevice}
                onEdit={setEditingDevice}
              />
            ))}
          </div>
        </div>
      )}

      {/* Edit Agent Modal */}
      <AgentEditModal
        key={editingAgent?.id}
        agent={editingAgent}
        isOpen={!!editingAgent}
        onClose={() => setEditingAgent(null)}
        onSave={handleSaveAgent}
      />

      {/* Edit Device Modal */}
      <Modal
        isOpen={!!editingDevice}
        onClose={() => setEditingDevice(null)}
        title={t("devices.form.edit_device")}
        subtitle={t("devices.form.infrastructure")}
        icon={<Cpu size={24} />}
        maxWidth="lg"
      >
        {editingDevice && (
          <form data-testid="edit-device-form" onSubmit={handleSaveDevice} className="space-y-8 pt-4 animate-in fade-in duration-500">
            <div className="space-y-3 px-1">
              <label className="text-[10px] font-bold text-slate-500 uppercase tracking-widest ml-1">{t("devices.form.clock_name")}</label>
              <input
                data-testid="device-name-input"
                required
                className="w-full px-5 py-4 bg-slate-50 border border-slate-100 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all font-bold text-sm uppercase tracking-tight shadow-sm"
                placeholder={t("devices.form.clock_name_placeholder")}
                value={editingDevice.name}
                onChange={(e) => setEditingDevice({ ...editingDevice, name: e.target.value })}
              />
            </div>

            <div className="grid grid-cols-2 gap-8">
              <div className="space-y-3 px-1">
                <label className="text-[10px] font-bold text-slate-500 uppercase tracking-widest ml-1">{t("devices.form.managed_by")}</label>
                <Combobox
                  data-testid="device-agent-select"
                  options={agents.map((a) => ({ value: a.id, label: `${a.name} (${a.id})`, icon: <Server size={14} /> }))}
                  value={editingDevice.agent_id}
                  onChange={(val) => setEditingDevice({ ...editingDevice, agent_id: val })}
                  placeholder={t("devices.form.select_agent")}
                  className="bg-slate-50 border-slate-100 h-14"
                />
              </div>
              <div className="space-y-3 px-1">
                <label className="text-[10px] font-bold text-slate-500 uppercase tracking-widest ml-1">{t("devices.form.brand")}</label>
                <Combobox
                  data-testid="device-brand-select"
                  options={brandOptions}
                  value={editingDevice.brand}
                  onChange={(val) => setEditingDevice({ ...editingDevice, brand: val })}
                  placeholder={t("devices.form.brand_placeholder")}
                  className="bg-slate-50 border-slate-100 h-14"
                />
              </div>
            </div>

            <div className="grid grid-cols-3 gap-8">
              <div className="col-span-2 space-y-3 px-1">
                <label className="text-[10px] font-bold text-slate-500 uppercase tracking-widest ml-1">{t("devices.form.ip_local")}</label>
                <input
                  data-testid="device-ip-input"
                  required
                  className="w-full px-5 py-4 bg-slate-50 border border-slate-100 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all font-mono font-bold text-sm tracking-tight"
                  placeholder="192.168.1.100"
                  value={editingDevice.ip}
                  onChange={(e) => setEditingDevice({ ...editingDevice, ip: e.target.value })}
                />
              </div>
              <div className="space-y-3 px-1">
                <label className="text-[10px] font-bold text-slate-500 uppercase tracking-widest ml-1">{t("devices.form.port")}</label>
                <input
                  data-testid="device-port-input"
                  type="number"
                  required
                  className="w-full px-5 py-4 bg-slate-50 border border-slate-100 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all font-mono font-bold text-sm text-center tracking-tighter"
                  value={editingDevice.port}
                  onChange={(e) => setEditingDevice({ ...editingDevice, port: parseInt(e.target.value) })}
                />
              </div>
            </div>

            <div className="flex gap-4 pt-6">
              <button
                data-testid="device-form-cancel"
                type="button"
                onClick={() => setEditingDevice(null)}
                className="flex-1 px-8 py-4 bg-white border border-slate-200 rounded-2xl font-bold text-xs uppercase tracking-widest text-slate-500 hover:bg-slate-50 transition-all active:scale-95"
              >
                {t("devices.form.cancel")}
              </button>
              <button
                data-testid="device-form-submit"
                type="submit"
                className="flex-1 bg-primary-600 text-white px-8 py-4 rounded-2xl font-bold uppercase tracking-widest text-xs hover:bg-primary-700 transition-all shadow-xl shadow-primary-100 active:scale-95 flex items-center justify-center gap-3"
              >
                <Edit2 size={18} />
                {t("devices.form.update_device")}
              </button>
            </div>
          </form>
        )}
      </Modal>
    </div>
  );
}
