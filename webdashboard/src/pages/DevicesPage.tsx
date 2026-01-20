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
} from "lucide-react";
import { Link } from "react-router-dom";
import { useDevices } from "@/hooks/use-devices";
import { useAgentsStatus } from "@/hooks/use-agents";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { useToast } from "../components/Toast";
import { Combobox } from "@/components/ui/Combobox";
import Modal from "@/components/ui/Modal";
import { cn } from "@/lib/utils";

export default function DevicesPage() {
  const { t } = useTranslation();
  const toast = useToast();
  const { devices, isLoading: devicesLoading, createDevice, updateDevice, deleteDevice } = useDevices();
  const { agents, isLoading: agentsLoading } = useAgentsStatus();

  const [showAddForm, setShowAddForm] = useState(false);
  const [editingDevice, setEditingDevice] = useState<any>(null);
  const [newDevice, setNewDevice] = useState({ name: "", agent_id: "", ip: "", port: 4370 });

  async function handleAdd(e: React.FormEvent) {
    e.preventDefault();
    try {
      await createDevice(newDevice);
      toast.success(t("devices.toast.add_success", { name: newDevice.name }));
      setShowAddForm(false);
      setNewDevice({ name: "", agent_id: "", ip: "", port: 4370 });
    } catch (e) { toast.error(t("devices.toast.add_error")); }
  }

  async function handleUpdate(e: React.FormEvent) {
    e.preventDefault();
    try {
      await updateDevice(editingDevice.id, { name: editingDevice.name, agent_id: editingDevice.agent_id, ip: editingDevice.ip, port: editingDevice.port, is_enabled: editingDevice.is_enabled });
      toast.success(t("devices.toast.update_success", { name: editingDevice.name }));
      setEditingDevice(null);
    } catch (e) { toast.error(t("devices.toast.update_error")); }
  }

  async function handleDelete(id: string) {
    if (!confirm(t("devices.toast.delete_confirm"))) return;
    try {
      await deleteDevice(id);
      toast.success(t("devices.toast.delete_success"));
    } catch (e) { toast.error(t("devices.toast.delete_error")); }
  }

  const loading = devicesLoading || agentsLoading;

  return (
    <div className="space-y-8 animate-in fade-in duration-500">
      {/* Header */}
      <div className="flex justify-between items-center px-2">
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-2xl bg-primary-50 flex items-center justify-center text-primary-600 shadow-sm border border-primary-100/50">
            <HardDrive size={20} />
          </div>
          <div>
            <h3 className="text-lg font-bold text-slate-800 tracking-tight">{t("devices.title")}</h3>
            <p className="text-xs text-slate-500 font-medium">{t("devices.form.infrastructure")}</p>
          </div>
        </div>
        <button onClick={() => setShowAddForm(true)} className="flex items-center gap-2 bg-primary-600 text-white px-6 py-2.5 rounded-2xl hover:bg-primary-700 transition-all shadow-lg shadow-primary-100 font-black text-[10px] uppercase tracking-widest active:scale-95">
          <Plus size={14} /> {t("devices.add_btn")}
        </button>
      </div>

      <div className="grid grid-cols-1 gap-6">
        {loading ? (
          Array.from({ length: 3 }).map((_, i) => (
            <div key={i} className="bg-white border border-slate-100 p-6 rounded-[32px] shadow-sm animate-pulse h-24" />
          ))
        ) : devices.length === 0 ? (
          <div className="py-24 text-center border-2 border-dashed border-slate-100 rounded-[40px] bg-slate-50/30 flex flex-col items-center">
            <div className="w-20 h-20 bg-white rounded-full flex items-center justify-center shadow-sm mb-4">
              <Cpu size={32} className="text-slate-300" />
            </div>
            <h3 className="text-lg font-bold text-slate-900">{t("devices.table.no_data")}</h3>
            <p className="text-slate-400 text-sm max-w-xs mt-1">{t("devices.messages.empty_desc", { defaultValue: "Bắt đầu bằng cách kết nối máy chấm công đầu tiên." })}</p>
          </div>
        ) : (
          devices.map((device) => (
            <div key={device.id} className="bg-white p-6 rounded-[2.5rem] border border-slate-100 flex items-center gap-8 hover:shadow-xl hover:shadow-primary-100/10 transition-all group relative">
              <div className="bg-slate-50 p-5 rounded-3xl group-hover:bg-primary-50 transition-colors border border-slate-100 shadow-inner">
                <Cpu size={32} className="text-slate-400 group-hover:text-primary-600 transition-colors" />
              </div>

              <div className="flex-1 grid grid-cols-1 md:grid-cols-3 gap-8">
                <div>
                  <p className="text-[9px] text-slate-400 font-black uppercase tracking-widest mb-1.5">{t("devices.table.device")}</p>
                  <h3 className="font-black text-slate-900 text-lg uppercase tracking-tight leading-tight">{device.name}</h3>
                  <p className="text-[10px] font-mono font-bold text-slate-400 uppercase tracking-tighter mt-1">{device.id}</p>
                </div>

                <div>
                  <p className="text-[9px] text-slate-400 font-black uppercase tracking-widest mb-1.5">{t("devices.table.managed_by")}</p>
                  <div className="flex items-center gap-2 text-slate-700">
                    <Server size={14} className="text-primary-500" />
                    <span className="font-black text-xs uppercase tracking-tight">{device.agent_id}</span>
                  </div>
                </div>

                <div>
                  <p className="text-[9px] text-slate-400 font-black uppercase tracking-widest mb-1.5">{t("devices.table.connection")}</p>
                  <div className="flex items-center gap-3">
                    <div className="flex items-center gap-1.5 text-slate-600 bg-slate-50 px-2.5 py-1 rounded-lg border border-slate-100 shadow-inner">
                      <Globe size={12} className="text-slate-300" />
                      <span className="text-xs font-black font-mono">{device.ip}</span>
                    </div>
                    <div className="flex items-center gap-1 text-slate-400 font-mono font-bold text-xs uppercase">
                      <Hash size={12} /> {device.port}
                    </div>
                  </div>
                </div>
              </div>

              <div className="flex items-center gap-6">
                <span className={cn("px-4 py-1.5 rounded-full text-[9px] font-black uppercase tracking-widest border", 
                  device.is_enabled ? "bg-emerald-50 text-emerald-600 border-emerald-100" : "bg-slate-100 text-slate-400 border-slate-200"
                )}>
                  {device.is_enabled ? t("devices.table.active") : t("devices.table.disabled")}
                </span>

                <DropdownMenu>
                  <DropdownMenuTrigger asChild>
                    <button className="text-slate-300 hover:text-primary-600 p-2 hover:bg-primary-50 rounded-xl transition-all outline-none border border-transparent hover:border-primary-100 shadow-sm hover:shadow-md">
                      <MoreVertical size={20} />
                    </button>
                  </DropdownMenuTrigger>
                  <DropdownMenuContent align="end" className="w-56 p-2 rounded-2xl shadow-xl border-slate-100">
                    <DropdownMenuLabel className="text-[10px] font-black uppercase tracking-widest text-slate-400 px-3 py-2">{t("devices.table.actions")}</DropdownMenuLabel>
                    <DropdownMenuItem className="gap-3 p-3 rounded-xl font-bold text-sm" onSelect={() => setEditingDevice(device)}>
                      <Edit2 size={16} className="text-slate-400" /> {t("devices.table.edit")}
                    </DropdownMenuItem>
                    <DropdownMenuItem className="gap-3 p-3 rounded-xl font-bold text-sm">
                      <RefreshCw size={16} className="text-slate-400" /> {t("devices.table.sync_time")}
                    </DropdownMenuItem>
                    <DropdownMenuItem className="gap-3 p-3 rounded-xl font-bold text-sm">
                      <Settings size={16} className="text-slate-400" /> {t("devices.table.advanced")}
                    </DropdownMenuItem>
                    <DropdownMenuSeparator className="my-2" />
                    <DropdownMenuItem onSelect={() => handleDelete(device.id)} className="gap-3 p-3 rounded-xl font-black text-sm text-rose-600 focus:text-rose-600 focus:bg-rose-50">
                      <Trash2 size={16} /> {t("devices.table.delete")}
                    </DropdownMenuItem>
                  </DropdownMenuContent>
                </DropdownMenu>
              </div>
            </div>
          ))
        )}
      </div>

      <Modal isOpen={showAddForm || !!editingDevice} onClose={() => { setShowAddForm(false); setEditingDevice(null); }} title={editingDevice ? t("devices.form.edit_device") : t("devices.form.new_device")} subtitle={t("devices.form.infrastructure")} icon={<Cpu size={24} />} maxWidth="lg">
        <form onSubmit={editingDevice ? handleUpdate : handleAdd} className="space-y-6 pt-4">
          <div className="space-y-2">
            <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("devices.form.clock_name")}</label>
            <input required className="w-full px-5 py-4 bg-slate-50 border border-slate-100 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all font-black text-sm uppercase tracking-tight" placeholder={t("devices.form.clock_name_placeholder")} value={editingDevice ? editingDevice.name : newDevice.name} onChange={(e) => editingDevice ? setEditingDevice({ ...editingDevice, name: e.target.value }) : setNewDevice({ ...newDevice, name: e.target.value })} />
          </div>
          <div className="space-y-2">
            <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("devices.form.managed_by")}</label>
            <Combobox options={agents.map((a) => ({ value: a.agent_id, label: a.agent_id, icon: <Server size={14} /> }))} value={editingDevice ? editingDevice.agent_id : newDevice.agent_id} onChange={(val) => editingDevice ? setEditingDevice({ ...editingDevice, agent_id: val }) : setNewDevice({ ...newDevice, agent_id: val })} placeholder={t("devices.form.select_agent")} className="bg-slate-50 border-slate-100" />
          </div>
          <div className="grid grid-cols-3 gap-6">
            <div className="col-span-2 space-y-2">
              <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("devices.form.ip_local")}</label>
              <input required className="w-full px-5 py-4 bg-slate-50 border border-slate-100 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all font-mono font-black text-sm" placeholder="192.168.1.100" value={editingDevice ? editingDevice.ip : newDevice.ip} onChange={(e) => editingDevice ? setEditingDevice({ ...editingDevice, ip: e.target.value }) : setNewDevice({ ...newDevice, ip: e.target.value })} />
            </div>
            <div className="space-y-2">
              <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("devices.form.port")}</label>
              <input type="number" required className="w-full px-5 py-4 bg-slate-50 border border-slate-100 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all font-mono font-black text-sm text-center" value={editingDevice ? editingDevice.port : newDevice.port} onChange={(e) => editingDevice ? setEditingDevice({ ...editingDevice, port: parseInt(e.target.value) }) : setNewDevice({ ...newDevice, port: parseInt(e.target.value) })} />
            </div>
          </div>
          <div className="pt-6 grid grid-cols-2 gap-5">
            <button type="button" onClick={() => { setShowAddForm(false); setEditingDevice(null); }} className="px-8 py-4 border border-slate-100 rounded-2xl font-black text-xs uppercase tracking-widest text-slate-500 hover:bg-slate-50 transition-all">{t("devices.form.cancel")}</button>
            <button type="submit" className="px-8 py-4 bg-slate-900 text-white rounded-2xl font-black text-xs uppercase tracking-widest hover:bg-black transition-all shadow-xl shadow-slate-200">{editingDevice ? t("devices.form.update_device") : t("devices.form.save_device")}</button>
          </div>
        </form>
      </Modal>
    </div>
  );
}
      </div>
    </div>
  );
}
