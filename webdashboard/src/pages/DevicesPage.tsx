import { useState } from "react";
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
import { Link } from "react-router-dom";
import { useDevices } from "@/hooks/use-devices";
import { useAuthorizedAgents } from "@/hooks/use-agents";
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

export default function DevicesPage() {
  const { t } = useTranslation();
  const toast = useToast();
  const { devices, isLoading: devicesLoading, createDevice, updateDevice, deleteDevice } = useDevices();
  const { agents: authorizedAgents, isLoading: agentsLoading } = useAuthorizedAgents();

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

  async function handleToggleDevice(device: { id: string; name: string; is_enabled: boolean }) {
    const confirmKey = device.is_enabled ? "devices.confirm_disable" : "devices.confirm_enable";
    if (!confirm(t(confirmKey, { name: device.name }))) return;

    setTogglingId(device.id);
    try {
      await updateDevice(device.id, { is_enabled: !device.is_enabled });
      toast.success(t(device.is_enabled ? "devices.toast.disabled" : "devices.toast.enabled", { name: device.name }));
    } catch (e) {
      console.error(e);
      toast.error(t("devices.toast.toggle_error"));
    } finally {
      setTogglingId(null);
    }
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
            <div key={device.id} className="bg-white dark:bg-slate-900 p-8 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 flex flex-col md:flex-row md:items-center gap-8 hover:shadow-2xl hover:shadow-primary-100/10 transition-all group relative overflow-hidden">
              <div className="bg-slate-50 dark:bg-slate-800 p-6 rounded-[2rem] group-hover:bg-primary-50 dark:group-hover:bg-primary-950 group-hover:rotate-3 transition-all border border-slate-100 dark:border-slate-800 shadow-inner shrink-0">
                <Cpu size={32} className="text-slate-500 dark:text-slate-400 group-hover:text-primary-600 dark:group-hover:text-primary-400 transition-colors" />
              </div>

              <div className="flex-1 grid grid-cols-1 md:grid-cols-4 gap-8">
                <div>
                  <p className="text-[9px] text-slate-500 dark:text-slate-400 font-bold uppercase tracking-[0.2em] mb-2">{t("devices.table.device")}</p>
                  <h3 className="font-bold text-slate-700 dark:text-slate-200 text-sm uppercase tracking-tight leading-none">{device.name}</h3>
                  <div className="flex items-center gap-2 mt-2">
                    <p className="text-[10px] font-mono font-bold text-slate-500 dark:text-slate-400 uppercase tracking-tighter opacity-70">{device.id}</p>
                    <span className="text-[8px] bg-slate-100 dark:bg-slate-700 px-1.5 py-0.5 rounded font-bold text-slate-400 dark:text-slate-500 uppercase tracking-widest">{device.brand}</span>
                  </div>
                </div>

                <div>
                  <p className="text-[9px] text-slate-500 dark:text-slate-400 font-bold uppercase tracking-[0.2em] mb-2">{t("devices.table.managed_by")}</p>
                  <div className="flex items-center gap-2.5 text-slate-700 dark:text-slate-200">
                    <Server size={14} className="text-primary-500 opacity-50" />
                    <span className="font-bold text-xs uppercase tracking-tight">{device.agent_id}</span>
                  </div>
                </div>

                <div>
                  <p className="text-[9px] text-slate-500 dark:text-slate-400 font-bold uppercase tracking-[0.2em] mb-2">{t("devices.table.connection")}</p>
                  <div className="flex items-center gap-3">
                    <div className="flex items-center gap-2 text-slate-600 dark:text-slate-300 bg-slate-50 dark:bg-slate-800 px-3 py-1.5 rounded-lg border border-slate-100 dark:border-slate-800 shadow-inner">
                      <Globe size={12} className="text-slate-300 dark:text-slate-600" />
                      <span className="text-[11px] font-bold font-mono tracking-tighter">{device.ip}</span>
                    </div>
                    <div className="flex items-center gap-1.5 text-slate-500 dark:text-slate-400 font-mono font-bold text-[11px] uppercase opacity-60">
                      <Hash size={12} /> {device.port}
                    </div>
                  </div>
                </div>

                <div className="flex items-center justify-end gap-4 md:justify-start">
                   <span className={cn("px-4 py-1.5 rounded-full text-[9px] font-bold uppercase tracking-widest border shadow-sm",
                    device.is_enabled ? "bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 border-emerald-100 dark:border-emerald-800" : "bg-slate-100 dark:bg-slate-700 text-slate-500 dark:text-slate-400 border-slate-200 dark:border-slate-700 opacity-60"
                  )}>
                    {device.is_enabled ? t("devices.table.active").toUpperCase() : t("devices.table.disabled").toUpperCase()}
                  </span>

                  <button
                    onClick={() => handleToggleDevice(device)}
                    disabled={togglingId === device.id}
                    className={cn(
                      "px-4 py-1.5 rounded-xl text-[9px] font-bold uppercase tracking-widest transition-all border shadow-sm active:scale-95",
                      togglingId === device.id && "opacity-50 cursor-not-allowed",
                      device.is_enabled
                        ? "text-rose-600 dark:text-rose-300 hover:bg-rose-50 dark:bg-rose-950 border-rose-100 dark:border-rose-800 bg-white dark:bg-slate-900"
                        : "bg-emerald-600 text-white hover:bg-emerald-700 border-transparent shadow-emerald-100"
                    )}
                  >
                    {togglingId === device.id ? (
                      <RefreshCw size={12} className="animate-spin" />
                    ) : device.is_enabled ? (
                      t("common.disable").toUpperCase()
                    ) : (
                      t("common.enable").toUpperCase()
                    )}
                  </button>
                </div>
              </div>

              <div className="flex items-center gap-4 shrink-0">
                <DropdownMenu>
                  <DropdownMenuTrigger asChild>
                    <button className="text-slate-300 dark:text-slate-600 hover:text-primary-600 dark:text-primary-400 p-3 hover:bg-white dark:hover:bg-slate-900 rounded-xl transition-all border border-transparent hover:border-slate-100 dark:border-slate-800 shadow-sm hover:shadow-md outline-none active:scale-90">
                      <MoreVertical size={20} />
                    </button>
                  </DropdownMenuTrigger>
                  <DropdownMenuContent align="end" className="w-64 p-3 rounded-[1.5rem] shadow-premium-2xl border-slate-100 dark:border-slate-800 bg-white dark:bg-slate-900/90 backdrop-blur-xl">
                    <DropdownMenuLabel className="text-[9px] font-bold uppercase tracking-[0.25em] text-slate-500 dark:text-slate-400 px-4 py-3 opacity-70">{t("devices.table.actions")}</DropdownMenuLabel>
                    <DropdownMenuItem className="gap-4 p-4 rounded-xl font-bold text-xs uppercase tracking-widest" onSelect={() => setEditingDevice(device)}>
                      <Edit2 size={16} className="text-slate-500 dark:text-slate-400" /> {t("devices.table.edit")}
                    </DropdownMenuItem>
                    <DropdownMenuItem className="gap-4 p-4 rounded-xl font-bold text-xs uppercase tracking-widest">
                      <RefreshCw size={16} className="text-slate-500 dark:text-slate-400" /> {t("devices.table.sync_time")}
                    </DropdownMenuItem>
                    <DropdownMenuItem className="gap-4 p-4 rounded-xl font-bold text-xs uppercase tracking-widest">
                      <Settings size={16} className="text-slate-500 dark:text-slate-400" /> {t("devices.table.advanced")}
                    </DropdownMenuItem>
                    <DropdownMenuSeparator className="my-2 bg-slate-50 dark:bg-slate-800" />
                    <DropdownMenuItem onSelect={() => handleDelete(device.id)} className="gap-4 p-4 rounded-xl font-bold text-xs uppercase tracking-widest text-rose-600 dark:text-rose-300 focus:text-rose-600 dark:text-rose-300 focus:bg-rose-50 dark:bg-rose-950">
                      <Trash2 size={16} /> {t("devices.table.delete")}
                    </DropdownMenuItem>
                  </DropdownMenuContent>
                </DropdownMenu>
              </div>
            </div>
          )
        )}
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
