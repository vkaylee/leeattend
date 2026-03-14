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
  Cpu as BrandIcon,
  Lock,
  Clock
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
import Button from "@/components/ui/Button";
import { LABEL_BASE } from "@/components/ui/FormField";
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
  const [newDevice, setNewDevice] = useState({ name: "", agent_id: "", ip: "", port: 4370, brand: "zkteco", password: "", timezone: "UTC" });
  const [togglingId, setTogglingId] = useState<string | null>(null);

  async function handleAdd(e: React.FormEvent) {
    e.preventDefault();
    try {
      await createDevice(newDevice);
      toast.success(t("devices.toast.add_success", { name: newDevice.name }));
      setShowAddForm(false); setNewDevice({ name: "", agent_id: "", ip: "", port: 4370, brand: "zkteco", password: "", timezone: "UTC" });
    } catch (e) { console.error(e); }
  }

  async function handleUpdate(e: React.FormEvent) {
    e.preventDefault();
    try {
      await updateDevice(editingDevice.id, { name: editingDevice.name, agent_id: editingDevice.agent_id, ip: editingDevice.ip, port: editingDevice.port, brand: editingDevice.brand, is_enabled: editingDevice.is_enabled, password: editingDevice.password || undefined, timezone: editingDevice.timezone });
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
        <Button data-testid="add-device-btn" onClick={() => setShowAddForm(true)} icon={<Plus size={18} />}>{t("devices.add_btn").toUpperCase()}</Button>
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
            <Button variant="outline" size="sm" onClick={() => setShowAddForm(true)}>{t("devices.add_btn")}</Button>
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

      <Modal isOpen={showAddForm || !!editingDevice} onClose={() => { setShowAddForm(false); setEditingDevice(null); }} title={editingDevice ? t("devices.form.edit_device") : t("devices.form.new_device")} subtitle={t("devices.form.infrastructure")} icon={<Cpu size={24} />} maxWidth="lg" testId={editingDevice ? "edit-device-form" : "add-device-form"}>
        <form onSubmit={editingDevice ? handleUpdate : handleAdd} className="space-y-8 pt-4 animate-in fade-in duration-500">
          <div className="space-y-3 px-1">
            <label className={LABEL_BASE}>{t("devices.form.clock_name")}</label>
            <input data-testid="device-name-input" required className="w-full px-5 py-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-bold text-sm uppercase tracking-tight shadow-sm" placeholder={t("devices.form.clock_name_placeholder")} value={editingDevice ? editingDevice.name : newDevice.name} onChange={(e) => editingDevice ? setEditingDevice({ ...editingDevice, name: e.target.value }) : setNewDevice({ ...newDevice, name: e.target.value })} />
          </div>

          <div className="grid grid-cols-2 gap-8">
            <div className="space-y-3 px-1">
              <label className={LABEL_BASE}>{t("devices.form.managed_by")}</label>
              <Combobox data-testid="device-agent-select" options={authorizedAgents.map((a) => ({ value: a.id, label: `${a.name} (${a.id})`, icon: <Server size={14} /> }))} value={editingDevice ? editingDevice.agent_id : newDevice.agent_id} onChange={(val) => editingDevice ? setEditingDevice({ ...editingDevice, agent_id: val }) : setNewDevice({ ...newDevice, agent_id: val })} placeholder={t("devices.form.select_agent")} className="bg-slate-50 dark:bg-slate-800 border-slate-100 dark:border-slate-800 h-14" />
            </div>
            <div className="space-y-3 px-1">
              <label className={LABEL_BASE}>{t("devices.form.brand")}</label>
              <Combobox data-testid="device-brand-select" options={brandOptions} value={editingDevice ? editingDevice.brand : newDevice.brand} onChange={(val) => editingDevice ? setEditingDevice({ ...editingDevice, brand: val }) : setNewDevice({ ...newDevice, brand: val })} placeholder={t("devices.form.brand_placeholder")} className="bg-slate-50 dark:bg-slate-800 border-slate-100 dark:border-slate-800 h-14" />
            </div>
          </div>

          <div className="grid grid-cols-3 gap-8">
            <div className="col-span-2 space-y-3 px-1">
              <label className={LABEL_BASE}>{t("devices.form.ip_local")}</label>
              <input data-testid="device-ip-input" required className="w-full px-5 py-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-mono font-bold text-sm tracking-tight" placeholder="192.168.1.100" value={editingDevice ? editingDevice.ip : newDevice.ip} onChange={(e) => editingDevice ? setEditingDevice({ ...editingDevice, ip: e.target.value }) : setNewDevice({ ...newDevice, ip: e.target.value })} />
            </div>
            <div className="space-y-3 px-1">
              <label className={LABEL_BASE}>{t("devices.form.port")}</label>
              <input data-testid="device-port-input" type="number" required className="w-full px-5 py-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-mono font-bold text-sm text-center tracking-tighter" value={editingDevice ? editingDevice.port : newDevice.port} onChange={(e) => editingDevice ? setEditingDevice({ ...editingDevice, port: parseInt(e.target.value) }) : setNewDevice({ ...newDevice, port: parseInt(e.target.value) })} />
            </div>
          </div>

          <div className="grid grid-cols-2 gap-8">
            <div className="space-y-3 px-1">
              <label className={LABEL_BASE}>{t("devices.form.password")}</label>
              <div className="relative">
                <Lock size={16} className="absolute left-4 top-1/2 -translate-y-1/2 text-slate-400" />
                <input data-testid="device-password-input" type="password" className="w-full pl-11 pr-5 py-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-mono font-bold text-sm tracking-tight" placeholder={t("devices.form.password_placeholder")} value={editingDevice ? (editingDevice.password || "") : newDevice.password} onChange={(e) => editingDevice ? setEditingDevice({ ...editingDevice, password: e.target.value }) : setNewDevice({ ...newDevice, password: e.target.value })} />
              </div>
            </div>
            <div className="space-y-3 px-1">
              <label className={LABEL_BASE}>{t("devices.form.timezone")}</label>
              <Combobox options={[
                { value: "UTC", label: "UTC", icon: <Clock size={14} /> },
                { value: "America/New_York", label: "America/New_York (EST)", icon: <Clock size={14} /> },
                { value: "America/Chicago", label: "America/Chicago (CST)", icon: <Clock size={14} /> },
                { value: "America/Denver", label: "America/Denver (MST)", icon: <Clock size={14} /> },
                { value: "America/Los_Angeles", label: "America/Los_Angeles (PST)", icon: <Clock size={14} /> },
                { value: "America/Sao_Paulo", label: "America/Sao_Paulo (BRT)", icon: <Clock size={14} /> },
                { value: "Europe/London", label: "Europe/London (GMT)", icon: <Clock size={14} /> },
                { value: "Europe/Berlin", label: "Europe/Berlin (CET)", icon: <Clock size={14} /> },
                { value: "Asia/Tokyo", label: "Asia/Tokyo (JST)", icon: <Clock size={14} /> },
                { value: "Asia/Shanghai", label: "Asia/Shanghai (CST)", icon: <Clock size={14} /> },
                { value: "Asia/Ho_Chi_Minh", label: "Asia/Ho_Chi_Minh (ICT)", icon: <Clock size={14} /> },
                { value: "Australia/Sydney", label: "Australia/Sydney (AEST)", icon: <Clock size={14} /> }
              ]} value={editingDevice ? (editingDevice.timezone || "UTC") : newDevice.timezone} onChange={(val) => editingDevice ? setEditingDevice({ ...editingDevice, timezone: val }) : setNewDevice({ ...newDevice, timezone: val })} placeholder={t("devices.form.timezone_placeholder")} className="bg-slate-50 dark:bg-slate-800 border-slate-100 dark:border-slate-800 h-14" data-testid="device-timezone-select" />
            </div>
          </div>

          <div className="flex gap-4 pt-6">
            <Button data-testid="device-form-cancel" variant="outline" size="lg" className="flex-1" onClick={() => { setShowAddForm(false); setEditingDevice(null); }}>{t("devices.form.cancel").toUpperCase()}</Button>
            <Button data-testid="device-form-submit" size="lg" type="submit" className="flex-1" icon={editingDevice ? <Edit2 size={18} /> : <Plus size={18} />}>
              {editingDevice ? t("devices.form.update_device").toUpperCase() : t("devices.form.save_device").toUpperCase()}
            </Button>
          </div>
        </form>
      </Modal>
    </div>
  );
}
