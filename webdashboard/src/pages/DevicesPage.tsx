import { useState, useMemo } from "react";
import { useTranslation } from "react-i18next";
import {
  Plus,
  Trash2,
  Server,
  Cpu,
  Globe,
  Hash,
  ShieldCheck,
  ShieldAlert,
  ArrowLeft,
  MoreVertical,
  Settings,
  RefreshCw,
  Edit2,
  X,
  HardDrive,
  Cpu as BrandIcon
} from "lucide-react";
import { useDevices } from "@/hooks/use-devices";
import { useAuthorizedAgents, useAgentsStatus } from "@/hooks/use-agents";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { useToast } from "../components/ui/Toast";
import { Combobox } from "@/components/ui/Combobox";
import Modal from "@/components/ui/Modal";
import { cn } from "@/lib/utils";
import { DeviceStatus } from "@/lib/api";
import DeviceListItem from "@/components/features/connectivity/DeviceListItem";

export default function DevicesPage() {
  const { t } = useTranslation();
  const toast = useToast();
  const { devices, isLoading: devicesLoading, createDevice, updateDevice, deleteDevice } = useDevices();
  const { agents: authorizedAgents, isLoading: agentsLoading, sendCommand } = useAuthorizedAgents();
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
      setTimeout(refetchStatus, 2000);
    } catch (e: any) {
      console.error("Refresh failed:", e);
      toast.error(`${t("devices.toast.refresh_error")}`);
    }
  }

  const [showAddForm, setShowAddForm] = useState(false);
  const [editingDevice, setEditingDevice] = useState<any>(null);
  const [newDevice, setNewDevice] = useState({ name: "", agent_id: "", ip: "", port: 4370, brand: "zkteco" });
  const [togglingId, setTogglingId] = useState<string | null>(null);

  async function handleAdd(e: React.FormEvent) {
    e.preventDefault();
    try {
      await createDevice(newDevice);
      toast.success(t("devices.toast.add_success", { name: newDevice.name }));
      setShowAddForm(false); setNewDevice({ name: "", agent_id: "", ip: "", port: 4370, brand: "zkteco" });
    } catch (e) { console.error(e); }
  }

  async function handleUpdate(e: React.FormEvent) {
    e.preventDefault();
    try {
      await updateDevice(editingDevice.id, { name: editingDevice.name, agent_id: editingDevice.agent_id, ip: editingDevice.ip, port: editingDevice.port, brand: editingDevice.brand, is_enabled: editingDevice.is_enabled });
      toast.success(t("devices.toast.update_success", { name: editingDevice.name }));
      setEditingDevice(null);
    } catch (e) { console.error(e); }
  }

  async function handleDelete(id: string) {
    if (!confirm(t("devices.toast.delete_confirm"))) return;
    try {
      await deleteDevice(id);
      toast.success(t("devices.toast.delete_success"));
    } catch (e) { console.error(e); }
  }

  const loading = devicesLoading || agentsLoading;

  const brandOptions = [
    { value: "zkteco", label: "ZKTeco (Real)", icon: <BrandIcon size={14} /> },
    { value: "mock", label: "Mock (Development)", icon: <BrandIcon size={14} /> }
  ];

  return (
    <div className="space-y-10 animate-in fade-in duration-700">
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-6 px-2">
        <div className="flex items-center gap-4">
          <div className="w-12 h-12 rounded-[1.25rem] bg-primary-50 dark:bg-primary-950 flex items-center justify-center text-primary-600 dark:text-primary-400 shadow-sm border border-primary-100 dark:border-primary-800/50"><HardDrive size={24} /></div>
          <div>
            <h3 className="text-2xl font-bold tracking-tight text-slate-700 dark:text-slate-200 uppercase leading-none">{t("devices.title")}</h3>
            <p className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest mt-1.5 opacity-70">{t("devices.form.infrastructure")}</p>
          </div>
        </div>
        <button onClick={() => setShowAddForm(true)} className="flex items-center gap-3 bg-primary-600 text-white px-8 py-3 rounded-2xl font-bold text-xs uppercase tracking-widest hover:bg-primary-700 transition-all shadow-xl shadow-primary-100 dark:shadow-black/30 active:scale-95"><Plus size={18} /> {t("devices.add_btn").toUpperCase()}</button>
      </div>

      <div className="grid grid-cols-1 gap-6">
        {loading ? (
          Array.from({ length: 3 }).map((_, i) => (
            <div key={i} className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 p-8 rounded-[2.5rem] shadow-sm animate-pulse h-32" />
          ))
        ) : devices.length === 0 ? (
          <div className="py-32 text-center border-2 border-dashed border-slate-100 dark:border-slate-800 rounded-[3rem] bg-slate-50 dark:bg-slate-800/30 flex flex-col items-center animate-in zoom-in-95 duration-500">
            <div className="w-24 h-24 bg-white dark:bg-slate-900 rounded-full flex items-center justify-center shadow-sm mb-6 border border-slate-100 dark:border-slate-800">
              <HardDrive size={32} className="text-slate-200 dark:text-slate-700" />
            </div>
            <p className="text-slate-500 dark:text-slate-400 font-bold uppercase tracking-widest text-[10px] opacity-60 mb-8">{t("devices.messages.empty_desc")}</p>
            <button onClick={() => setShowAddForm(true)} className="px-10 py-4 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-2xl font-bold text-[10px] uppercase tracking-[0.2em] text-slate-600 dark:text-slate-300 hover:border-primary-200 dark:border-primary-800 hover:text-primary-600 dark:text-primary-400 transition-all shadow-sm active:scale-95">{t("devices.add_btn")}</button>
          </div>
        ) : devices.map((device) => (
          <DeviceListItem
            key={device.id}
            device={device}
            status={deviceStatuses[device.id.toLowerCase()]}
            parentAgentActive={agentOnlineMap[device.agent_id] || false}
            onUpdate={(id, data) => updateDevice(id, data)}
            onDelete={(id) => deleteDevice(id)}
            onEdit={setEditingDevice}
            onRefresh={(deviceId) => handleRefreshDevice(device.agent_id, deviceId)}
          />
        ))}
      </div>

      <Modal isOpen={showAddForm || !!editingDevice} onClose={() => { setShowAddForm(false); setEditingDevice(null); }} title={editingDevice ? t("devices.form.edit_device") : t("devices.form.new_device")} subtitle={t("devices.form.infrastructure")} icon={<Cpu size={24} />} maxWidth="lg">
        <form onSubmit={editingDevice ? handleUpdate : handleAdd} className="space-y-8 pt-4 animate-in fade-in duration-500">
          <div className="space-y-3 px-1">
            <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{t("devices.form.clock_name")}</label>
            <input required className="w-full px-5 py-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-bold text-sm uppercase tracking-tight shadow-sm" placeholder={t("devices.form.clock_name_placeholder")} value={editingDevice ? editingDevice.name : newDevice.name} onChange={(e) => editingDevice ? setEditingDevice({ ...editingDevice, name: e.target.value }) : setNewDevice({ ...newDevice, name: e.target.value })} />
          </div>

          <div className="grid grid-cols-2 gap-8">
            <div className="space-y-3 px-1">
              <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{t("devices.form.managed_by")}</label>
              <Combobox options={authorizedAgents.map((a) => ({ value: a.id, label: `${a.name} (${a.id})`, icon: <Server size={14} /> }))} value={editingDevice ? editingDevice.agent_id : newDevice.agent_id} onChange={(val) => editingDevice ? setEditingDevice({ ...editingDevice, agent_id: val }) : setNewDevice({ ...newDevice, agent_id: val })} placeholder={t("devices.form.select_agent")} className="bg-slate-50 dark:bg-slate-800 border-slate-100 dark:border-slate-800 h-14" />
            </div>
            <div className="space-y-3 px-1">
              <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{t("devices.form.brand")}</label>
              <Combobox options={brandOptions} value={editingDevice ? editingDevice.brand : newDevice.brand} onChange={(val) => editingDevice ? setEditingDevice({ ...editingDevice, brand: val }) : setNewDevice({ ...newDevice, brand: val })} placeholder={t("devices.form.brand_placeholder")} className="bg-slate-50 dark:bg-slate-800 border-slate-100 dark:border-slate-800 h-14" />
            </div>
          </div>

          <div className="grid grid-cols-3 gap-8">
            <div className="col-span-2 space-y-3 px-1">
              <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{t("devices.form.ip_local")}</label>
              <input required className="w-full px-5 py-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-mono font-bold text-sm tracking-tight" placeholder="192.168.1.100" value={editingDevice ? editingDevice.ip : newDevice.ip} onChange={(e) => editingDevice ? setEditingDevice({ ...editingDevice, ip: e.target.value }) : setNewDevice({ ...newDevice, ip: e.target.value })} />
            </div>
            <div className="space-y-3 px-1">
              <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{t("devices.form.port")}</label>
              <input type="number" required className="w-full px-5 py-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-mono font-bold text-sm text-center tracking-tighter" value={editingDevice ? editingDevice.port : newDevice.port} onChange={(e) => editingDevice ? setEditingDevice({ ...editingDevice, port: parseInt(e.target.value) }) : setNewDevice({ ...newDevice, port: parseInt(e.target.value) })} />
            </div>
          </div>

          <div className="flex gap-4 pt-6">
            <button type="button" onClick={() => { setShowAddForm(false); setEditingDevice(null); }} className="flex-1 px-8 py-4 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-2xl font-bold text-xs uppercase tracking-widest text-slate-500 dark:text-slate-400 hover:bg-slate-50 dark:hover:bg-slate-800 transition-all active:scale-95">{t("devices.form.cancel").toUpperCase()}</button>
            <button type="submit" className="flex-1 bg-primary-600 text-white px-8 py-4 rounded-2xl font-bold uppercase tracking-widest text-xs hover:bg-primary-700 transition-all shadow-xl shadow-primary-100 dark:shadow-black/30 active:scale-95 flex items-center justify-center gap-3">
              {editingDevice ? <Edit2 size={18} /> : <Plus size={18} />}
              {editingDevice ? t("devices.form.update_device").toUpperCase() : t("devices.form.save_device").toUpperCase()}
            </button>
          </div>
        </form>
      </Modal>
    </div>
  );
}
