import { Plus, Share2, Server, AlertCircle, Cpu, Lock, Save } from "lucide-react";
import { useState, useMemo } from "react";
import { useTranslation } from "react-i18next";

import AgentEditModal from "@/components/features/agents/agent-edit-modal";
import { Combobox } from "@/components/ui/combobox";
import Modal from "@/components/ui/modal";
import { useToast } from "@/components/ui/toast";
import { useAuthorizedAgents, useAgentsStatus } from "@/hooks/use-agents";
import { useDevices } from "@/hooks/use-devices";
import { AuthorizedAgent, Device, DeviceStatus } from "@/lib/api";
import { cn } from "@/lib/utils";

import AgentAccordionCard from "./agent-accordion-card";
import DeviceListItem from "./device-list-item";


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
    sendCommand
  } = useAuthorizedAgents();

  const { agents: agentsStatus, refetch: refetchStatus } = useAgentsStatus();

  // Create a map of agent_id -> online status for easy lookup
  const agentOnlineMap = useMemo(() => {
    const map: Record<string, boolean> = {};
    agentsStatus.forEach(a => {
      map[a.agent_id] = a.is_online;
    });
    return map;
  }, [agentsStatus]);

  // Create a flattened map of device_id -> status
  const deviceStatuses = useMemo(() => {
    const map: Record<string, DeviceStatus> = {};
 
    agentsStatus.forEach(a => {
      a.devices.forEach(d => {
        map[d.device_id.toLowerCase()] = d;
      });
    });
    return map;
  }, [agentsStatus]);

  async function handleRefreshDevice(agentId: string, deviceId: string) {
    try {
      
      await sendCommand({
        id: agentId,
        command: {
          method: "get_device_info",
          device_id: deviceId,
          data: {}
        }
      });
      toast.success(t("devices.toast.refresh_sent"));
      // Refresh status after a short delay
      setTimeout(refetchStatus, 2000);
    } catch (e: unknown) {
 
      console.error("Refresh failed:", e);
      const message = e instanceof Error ? e.message : "";
      toast.error(`${t("devices.toast.refresh_error")} ${message ? `(${message ?? ""})` : ""}`);
    }
  }

  const {
    devices,
    isLoading: devicesLoading,
    createDevice,
    updateDevice,
    deleteDevice
  } = useDevices();

  const [expandedAgents, setExpandedAgents] = useState<Set<string>>(new Set());
  const [showAddForm, setShowAddForm] = useState(false);
  const [showAddDeviceForm, setShowAddDeviceForm] = useState(false);
  const [newAgent, setNewAgent] = useState({ id: "", name: "", token: "" });
  const [newDevice, setNewDevice] = useState({ name: "", agent_id: "", ip: "", port: 4370, brand: "zkteco", password: "", timezone: "UTC" });
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
    setExpandedAgents(previous => {
      const next = new Set(previous);
      if (next.has(agentId)) next.delete(agentId);
      else next.add(agentId);
      return next;
    });
  };

  const expandAll = () => { setExpandedAgents(new Set(agents.map(a => a.id))); };
  const collapseAll = () => { setExpandedAgents(new Set()); };

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

  async function handleAddDevice(e: React.FormEvent) {
    e.preventDefault();
    try {
      await createDevice(newDevice);
      toast.success(t("devices.toast.add_success", { name: newDevice.name }));
      setShowAddDeviceForm(false);
      setNewDevice({ name: "", agent_id: "", ip: "", port: 4370, brand: "zkteco", password: "", timezone: "UTC" });
    } catch (e) {
      console.error(e);
      toast.error(t("devices.toast.add_error"));
    }
  }

  const handleEditAgent = (agent: AuthorizedAgent) => { setEditingAgent(agent); };
  const handleSaveAgent = (updated: AuthorizedAgent) => {
    toast.success(t("agents.messages.update_success", { name: updated.name }));
    void refetchAgents();
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
    <div className="space-y-8 animate-in fade-in duration-500" data-testid="connectivity-topology">
      {/* Header */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-6 px-2">
        <div className="flex items-center gap-4">
          <div className="w-12 h-12 rounded-[1.25rem] bg-primary-50 dark:bg-primary-950 flex items-center justify-center text-primary-600 dark:text-primary-400 shadow-sm border border-primary-100 dark:border-primary-800/50">
            <Share2 size={24} />
          </div>
          <div>
            <h1 className="text-3xl font-bold tracking-tight text-slate-700 dark:text-slate-200 leading-tight">
              {t("common.settings.categories.connectivity")}
            </h1>
            <p className="text-slate-500 dark:text-slate-400 mt-1 font-medium">
              {t("common.settings.categories.connectivity_desc")}
            </p>
          </div>
        </div>

        <div className="flex items-center gap-3">
          {agents.length > 0 && (
            <>
              <button
                className="px-4 py-2 text-xs font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400 hover:text-primary-600 dark:text-primary-400 transition-colors"
                data-testid="expand-all-btn"
                onClick={expandAll}
              >
                {t("common.expand_all")}
              </button>
              <button
                className="px-4 py-2 text-xs font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400 hover:text-primary-600 dark:text-primary-400 transition-colors"
                data-testid="collapse-all-btn"
                onClick={collapseAll}
              >
                {t("common.collapse_all")}
              </button>
              <div className="w-px h-6 bg-slate-200 dark:bg-slate-600" />
            </>
          )}
          <button
            className={cn(
              "flex items-center gap-2 px-6 py-3 rounded-2xl font-bold text-[10px] uppercase tracking-widest transition-all border shadow-sm active:scale-95",
              includeInactive
                ? "bg-slate-900 text-white border-transparent"
                : "bg-white dark:bg-slate-900 text-slate-500 dark:text-slate-400 border-slate-200 dark:border-slate-700 hover:bg-slate-50 dark:hover:bg-slate-800"
            )}
            data-testid="toggle-inactive-btn"
            onClick={() => { setIncludeInactive(!includeInactive); }}
          >
            {includeInactive ? t("common.status_inactive") : t("common.status_active")}
          </button>
          <button
            className="flex items-center gap-3 bg-white dark:bg-slate-900 text-slate-600 dark:text-slate-300 px-6 py-3 rounded-2xl font-bold text-xs uppercase tracking-widest hover:border-primary-200 dark:hover:border-primary-800 transition-all border border-slate-200 dark:border-slate-700 active:scale-95"
            data-testid="add-agent-btn"
            onClick={() => { setShowAddForm(true); }}
          >
            <Plus size={18} /> {t("agents.add_btn")}
          </button>
          <button
            className="flex items-center gap-3 bg-primary-600 text-white px-8 py-3 rounded-2xl font-bold text-xs uppercase tracking-widest hover:bg-primary-700 transition-all shadow-xl shadow-primary-100 dark:shadow-black/30 active:scale-95"
            data-testid="global-add-device-btn"
            onClick={() => {
              if (agents.length > 0) {
                setNewDevice({ ...newDevice, agent_id: agents[0].id });
                setShowAddDeviceForm(true);
              } else {
                toast.error(t("agents.messages.no_agents"));
              }
            }}
          >
            <Plus size={18} /> {t("devices.add_btn")}
          </button>
        </div>
      </div>

      {/* Add Agent Form */}
      {!!showAddForm && (
        <form className="bg-white dark:bg-slate-900 border-2 border-primary-100 dark:border-primary-800 p-10 rounded-[2.5rem] shadow-xl shadow-primary-50/50 animate-in slide-in-from-top-4 duration-500" data-testid="add-agent-form" onSubmit={handleAddAgent}>
          <div className="flex items-center gap-3 mb-8 text-primary-700 dark:text-primary-300">
            <div className="w-8 h-8 rounded-xl bg-primary-50 dark:bg-primary-950 flex items-center justify-center shadow-inner"><Plus size={16} /></div>
            <h4 className="text-[10px] font-bold uppercase tracking-[0.2em]">{t("agents.add_form_title")}</h4>
          </div>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-8">
            <div className="space-y-3 px-1">
              <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{t("agents.id_label")}</label>
              <input className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-4 focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-mono font-bold text-sm tracking-tight" data-testid="agent-id-input" onChange={(e) => { setNewAgent({ ...newAgent, id: e.target.value }); }} placeholder="agent-001" required value={newAgent.id} />
            </div>
            <div className="space-y-3 px-1">
              <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{t("agents.name_label")}</label>
              <input className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-4 focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-bold text-sm tracking-tight" data-testid="agent-name-input" onChange={(e) => { setNewAgent({ ...newAgent, name: e.target.value }); }} placeholder="Office Main" required value={newAgent.name} />
            </div>
            <div className="space-y-3 px-1">
              <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{t("agents.token_label")}</label>
              <input className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-4 focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-mono font-bold text-sm" data-testid="agent-token-input" onChange={(e) => { setNewAgent({ ...newAgent, token: e.target.value }); }} placeholder="Connection token" required value={newAgent.token} />
            </div>
          </div>
          <div className="mt-10 flex justify-end gap-4">
            <button className="px-8 py-4 rounded-2xl font-bold text-xs uppercase tracking-widest text-slate-500 dark:text-slate-400 hover:bg-slate-50 dark:hover:bg-slate-800 transition-all" data-testid="agent-form-cancel" onClick={() => { setShowAddForm(false); }} type="button">{t("common.cancel")}</button>
            <button className="bg-primary-600 text-white px-12 py-4 rounded-2xl font-bold text-xs uppercase tracking-widest hover:bg-primary-700 transition-all shadow-xl shadow-primary-100 dark:shadow-black/30 active:scale-95" data-testid="agent-form-submit" type="submit">{t("agents.save_btn")}</button>
          </div>
        </form>
      )}

      {/* Agent Topology */}
      {isLoading ? (
        <div className="space-y-4" data-testid="agents-loading">
          {Array.from({ length: 3 }).map((_, index) => (
            <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 p-8 rounded-[2.5rem] shadow-sm animate-pulse h-28" key={index} />
          ))}
        </div>
      ) : agents.length === 0 ? (
        <div className="py-32 text-center border-2 border-dashed border-slate-100 dark:border-slate-800 rounded-[3rem] bg-slate-50 dark:bg-slate-800/30 flex flex-col items-center animate-in zoom-in-95 duration-500" data-testid="empty-agents-state">
          <div className="w-24 h-24 bg-white dark:bg-slate-900 rounded-full flex items-center justify-center shadow-sm mb-6 border border-slate-100 dark:border-slate-800">
            <Server className="text-slate-200 dark:text-slate-700" size={32} />
          </div>
          <p className="text-slate-500 dark:text-slate-400 font-bold uppercase tracking-widest text-[10px] opacity-60 mb-2">
            {t("agents.table.no_agents")}
          </p>
          <p className="text-slate-400 dark:text-slate-500 text-xs max-w-md mb-8">
            {t("connectivity.empty_guide")}
          </p>
          <button
            className="px-10 py-4 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-2xl font-bold text-[10px] uppercase tracking-[0.2em] text-slate-600 dark:text-slate-300 hover:border-primary-200 dark:border-primary-800 hover:text-primary-600 dark:text-primary-400 transition-all shadow-sm active:scale-95"
            data-testid="empty-add-agent-btn"
            onClick={() => { setShowAddForm(true); }}
          >
            {t("agents.add_btn")}
          </button>
        </div>
      ) : (
        <div className="space-y-4" data-testid="agents-list">
          {agents.map(agent => (
            <AgentAccordionCard
              agent={agent}
              devices={groupedDevices[agent.id] || []}
              deviceStatuses={deviceStatuses}
              isExpanded={expandedAgents.has(agent.id)}
              isOnline={agentOnlineMap[agent.id] || false}
              key={agent.id}
              onAddDevice={(agentId) => {
                setNewDevice({ ...newDevice, agent_id: agentId });
                setShowAddDeviceForm(true);
              }}
              onDelete={handleDeleteAgent}
              onDeleteDevice={handleDeleteDevice}
              onEdit={handleEditAgent}
              onEditDevice={setEditingDevice}
              onRefreshDevice={(deviceId) => handleRefreshDevice(agent.id, deviceId)}
              onToggleExpand={() => { toggleExpanded(agent.id); }}
              onToggleTrust={handleToggleTrust}
              onUpdateDevice={handleUpdateDevice}
            />
          ))}
        </div>
      )}

      {/* Unassigned Devices Section */}
      {unassignedDevices.length > 0 && (
        <div className="mt-8 animate-in fade-in duration-500" data-testid="unassigned-devices-section">
          <div className="flex items-center gap-3 mb-4 px-2">
            <AlertCircle className="text-amber-500" size={16} />
            <h3 className="text-xs font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400">
              {t("devices.unassigned_title")} ({unassignedDevices.length})
            </h3>
          </div>
          <div className="bg-amber-50 dark:bg-amber-950/50 border border-amber-100 dark:border-amber-800 rounded-[2.5rem] p-6 space-y-3" data-testid="unassigned-devices-list">
            {unassignedDevices.map(device => (
              <DeviceListItem
                device={device}
                key={device.id}
                onDelete={handleDeleteDevice}
                onEdit={setEditingDevice}
                onUpdate={handleUpdateDevice}
                parentAgentActive={false}
              />
            ))}
          </div>
        </div>
      )}

      {/* Edit Agent Modal */}
      <AgentEditModal
        agent={editingAgent}
        isOpen={!!editingAgent}
        key={editingAgent?.id}
        onClose={() => { setEditingAgent(null); }}
        onSave={handleSaveAgent}
      />

      {/* Add/Edit Device Modal */}
      <Modal
        icon={<Cpu size={24} />}
        isOpen={showAddDeviceForm || !!editingDevice}
        maxWidth="lg"
        onClose={() => { setShowAddDeviceForm(false); setEditingDevice(null); }}
        subtitle={t("devices.form.infrastructure")}
        title={editingDevice ? t("devices.form.edit_device") : t("devices.form.new_device")}
      >
        <form className="space-y-8 pt-4 animate-in fade-in duration-500" data-testid={editingDevice ? "edit-device-form" : "add-device-form"} onSubmit={editingDevice ? handleSaveDevice : handleAddDevice}>
          <div className="space-y-3 px-1">
            <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{t("devices.form.clock_name")}</label>
            <input className="w-full px-5 py-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-bold text-sm tracking-tight shadow-sm" data-testid="device-name-input" onChange={(e) => { editingDevice ? setEditingDevice({ ...editingDevice, name: e.target.value }) : setNewDevice({ ...newDevice, name: e.target.value }); }} placeholder={t("devices.form.clock_name_placeholder")} required value={editingDevice ? editingDevice.name : newDevice.name}
            />
          </div>

          <div className="grid grid-cols-2 gap-8">
            <div className="space-y-3 px-1">
              <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{t("devices.form.managed_by")}</label>
              {/* Show agent name as read-only (device always belongs to an agent) */}
              <div className="w-full px-5 py-4 bg-slate-100 dark:bg-slate-700 border border-slate-200 dark:border-slate-700 rounded-2xl font-bold text-sm uppercase tracking-tight text-slate-600 dark:text-slate-300 flex items-center gap-2">
                <Server className="text-slate-400 dark:text-slate-500" size={14} />
                {agents.find(a => a.id === (editingDevice ? editingDevice.agent_id : newDevice.agent_id))?.name ?? "—"}
              </div>
            </div>
            <div className="space-y-3 px-1">
              <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{t("devices.form.brand")}</label>
              <Combobox
                className="bg-slate-50 dark:bg-slate-800 border-slate-100 dark:border-slate-800 h-14"
                data-testid="device-brand-select"
                onChange={(value) => { editingDevice ? setEditingDevice({ ...editingDevice, brand: value }) : setNewDevice({ ...newDevice, brand: value }); }}
                options={brandOptions}
                placeholder={t("devices.form.brand_placeholder")}
                value={editingDevice ? editingDevice.brand : newDevice.brand}
              />
            </div>
          </div>

          <div className="grid grid-cols-3 gap-8">
            <div className="col-span-2 space-y-3 px-1">
              <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{t("devices.form.ip_local")}</label>
              <input
                className="w-full px-5 py-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-mono font-bold text-sm tracking-tight"
                data-testid="device-ip-input"
                onChange={(e) => { editingDevice ? setEditingDevice({ ...editingDevice, ip: e.target.value }) : setNewDevice({ ...newDevice, ip: e.target.value }); }}
                placeholder="192.168.1.100"
                required
                value={editingDevice ? editingDevice.ip : newDevice.ip}
              />
            </div>
            <div className="space-y-3 px-1">
              <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{t("devices.form.port")}</label>
              <input
                className="w-full px-5 py-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-mono font-bold text-sm text-center tracking-tighter"
                data-testid="device-port-input"
                onChange={(e) => { editingDevice ? setEditingDevice({ ...editingDevice, port: parseInt(e.target.value) }) : setNewDevice({ ...newDevice, port: parseInt(e.target.value) }); }}
                required
                type="number"
                value={editingDevice ? editingDevice.port : newDevice.port}
              />
            </div>
          </div>

          <div className="grid grid-cols-2 gap-8">
            <div className="space-y-3 px-1">
              <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{t("devices.form.password")}</label>
              <div className="relative">
                <Lock className="absolute left-4 top-1/2 -translate-y-1/2 text-slate-400" size={16} />
                <input
                  className="w-full pl-11 pr-5 py-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-mono font-bold text-sm tracking-tight"
                  data-testid="device-password-input"
                  onChange={(e) => { editingDevice ? setEditingDevice({ ...editingDevice, password: e.target.value }) : setNewDevice({ ...newDevice, password: e.target.value }); }}
                  placeholder={t("devices.form.password_placeholder")}
                  type="password"
                  value={editingDevice ? (editingDevice.password ?? "") : newDevice.password}
                />
              </div>
            </div>
            {/* Timezone field hidden from UI — workspace timezone is used for all log processing.
               The timezone value defaults to "UTC" in the data model for backward compatibility. */}
          </div>

          <div className="flex gap-4 pt-6">
            <button
              className="flex-1 px-8 py-4 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-2xl font-bold text-xs uppercase tracking-widest text-slate-500 dark:text-slate-400 hover:bg-slate-50 dark:hover:bg-slate-800 transition-all active:scale-95"
              data-testid="device-form-cancel"
              onClick={() => { setShowAddDeviceForm(false); setEditingDevice(null); }}
              type="button"
            >
              {t("devices.form.cancel")}
            </button>
            <button
              className="flex-1 bg-primary-600 text-white px-8 py-4 rounded-2xl font-bold uppercase tracking-widest text-xs hover:bg-primary-700 transition-all shadow-xl shadow-primary-100 dark:shadow-black/30 active:scale-95 flex items-center justify-center gap-3"
              data-testid="device-form-submit"
              type="submit"
            >
              {editingDevice ? <Save size={18} /> : <Plus size={18} />}
              {editingDevice ? t("devices.form.update_device") : t("devices.form.save_device")}
            </button>
          </div>
        </form>
      </Modal>
    </div>
  );
}
