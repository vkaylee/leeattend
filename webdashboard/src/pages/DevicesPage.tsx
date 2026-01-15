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

export default function DevicesPage() {
  const { t } = useTranslation();
  const toast = useToast();
  const {
    devices,
    isLoading: devicesLoading,
    createDevice,
    updateDevice,
    deleteDevice,
  } = useDevices();
  const { agents, isLoading: agentsLoading } = useAgentsStatus();

  const [showAddForm, setShowAddForm] = useState(false);
  const [editingDevice, setEditingDevice] = useState<any>(null);
  const [newDevice, setNewDevice] = useState({
    name: "",
    agent_id: "",
    ip: "",
    port: 4370,
  });

  async function handleAdd(e: React.FormEvent) {
    e.preventDefault();
    try {
      await createDevice(newDevice);
      toast.success(t("devices.toast.add_success", { name: newDevice.name }));
      setShowAddForm(false);
      setNewDevice({ name: "", agent_id: "", ip: "", port: 4370 });
    } catch (e) {
      toast.error(t("devices.toast.add_error"));
    }
  }

  async function handleUpdate(e: React.FormEvent) {
    e.preventDefault();
    try {
      await updateDevice(editingDevice.id, {
        name: editingDevice.name,
        agent_id: editingDevice.agent_id,
        ip: editingDevice.ip,
        port: editingDevice.port,
        is_enabled: editingDevice.is_enabled,
      });
      toast.success(t("devices.toast.update_success", { name: editingDevice.name }));
      setEditingDevice(null);
    } catch (e) {
      toast.error(t("devices.toast.update_error"));
    }
  }

  async function handleDelete(id: string) {
    if (!confirm(t("devices.toast.delete_confirm"))) return;
    try {
      await deleteDevice(id);
      toast.success(t("devices.toast.delete_success"));
    } catch (e) {
      toast.error(t("devices.toast.delete_error"));
    }
  }

  const loading = devicesLoading || agentsLoading;

  return (
    <div className="min-h-screen bg-slate-50 p-8 animate-in fade-in duration-500">
      <div className="max-w-6xl mx-auto">
        <div className="mb-8 flex items-center justify-between">
          <div className="flex items-center gap-4">
            <Link
              to="/"
              className="bg-white p-2 rounded-xl border border-slate-200 hover:bg-slate-50 transition-colors"
            >
              <ArrowLeft className="w-5 h-5 text-slate-600" />
            </Link>
            <h1 className="text-3xl font-bold text-slate-900 tracking-tight">
              {t("devices.title")}
            </h1>
          </div>
          <button
            onClick={() => setShowAddForm(!showAddForm)}
            className="bg-primary-600 text-white px-6 py-2.5 rounded-2xl font-bold flex items-center gap-2 hover:bg-primary-700 transition-colors shadow-lg shadow-primary-200"
          >
            <Plus className="w-5 h-5" />
            {t("devices.add_btn")}
          </button>
        </div>

      {/* Add/Edit Device Modal */}
      <Modal
        isOpen={showAddForm || !!editingDevice}
        onClose={() => { setShowAddForm(false); setEditingDevice(null); }}
        title={editingDevice ? t("devices.form.edit_device") : t("devices.form.new_device")}
        subtitle={t("devices.form.infrastructure")}
        icon={<Cpu className="w-6 h-6" />}
      >
        <form
          onSubmit={(e) => {
            if (editingDevice) {
              handleUpdate(e);
            } else {
              handleAdd(e);
            }
          }}
          className="space-y-6"
        >
          <div className="space-y-2">
            <label className="text-xs font-black text-slate-400 uppercase tracking-widest ml-1">
              {t("devices.form.clock_name")}
            </label>
            <input
              required
              className="w-full px-5 py-4 bg-slate-50 border border-slate-100 rounded-2xl focus:ring-4 focus:ring-primary-500/10 outline-none transition-all font-bold"
              placeholder={t("devices.form.clock_name_placeholder")}
              value={editingDevice ? editingDevice.name : newDevice.name}
              onChange={(e) =>
                editingDevice
                  ? setEditingDevice({ ...editingDevice, name: e.target.value })
                  : setNewDevice({ ...newDevice, name: e.target.value })
              }
            />
          </div>

          <div className="space-y-2">
            <label className="text-xs font-black text-slate-400 uppercase tracking-widest ml-1">
              {t("devices.form.managed_by")}
            </label>
            <Combobox
              options={agents.map((a) => ({
                value: a.agent_id,
                label: a.agent_id,
                icon: <Server className="w-4 h-4" />,
              }))}
              value={editingDevice ? editingDevice.agent_id : newDevice.agent_id}
              onChange={(val) =>
                editingDevice
                  ? setEditingDevice({ ...editingDevice, agent_id: val })
                  : setNewDevice({ ...newDevice, agent_id: val })
              }
              placeholder={t("devices.form.select_agent")}
              className="bg-slate-50 border-slate-100"
            />
          </div>

          <div className="grid grid-cols-3 gap-4">
            <div className="col-span-2 space-y-2">
              <label className="text-xs font-black text-slate-400 uppercase tracking-widest ml-1">
                {t("devices.form.ip_local")}
              </label>
              <input
                required
                className="w-full px-5 py-4 bg-slate-50 border border-slate-100 rounded-2xl focus:ring-4 focus:ring-primary-500/10 outline-none transition-all font-mono font-bold"
                placeholder="192.168.1.100"
                value={editingDevice ? editingDevice.ip : newDevice.ip}
                onChange={(e) =>
                  editingDevice
                    ? setEditingDevice({ ...editingDevice, ip: e.target.value })
                    : setNewDevice({ ...newDevice, ip: e.target.value })
                }
              />
            </div>
            <div className="space-y-2">
              <label className="text-xs font-black text-slate-400 uppercase tracking-widest ml-1">
                {t("devices.form.port")}
              </label>
              <input
                type="number"
                required
                className="w-full px-5 py-4 bg-slate-50 border border-slate-100 rounded-2xl focus:ring-4 focus:ring-primary-500/10 outline-none transition-all font-mono font-bold"
                value={editingDevice ? editingDevice.port : newDevice.port}
                onChange={(e) =>
                  editingDevice
                    ? setEditingDevice({
                        ...editingDevice,
                        port: parseInt(e.target.value),
                      })
                    : setNewDevice({
                        ...newDevice,
                        port: parseInt(e.target.value),
                      })
                }
              />
            </div>
          </div>

          <div className="pt-6 grid grid-cols-2 gap-5">
            <button
              type="button"
              onClick={() => {
                setShowAddForm(false);
                setEditingDevice(null);
              }}
              className="px-8 py-4 border border-slate-100 rounded-2xl font-bold text-slate-500 hover:bg-slate-50 transition-all"
            >
              {t("devices.form.cancel")}
            </button>
            <button
              type="submit"
              className="px-8 py-4 bg-slate-900 text-white rounded-2xl font-black hover:bg-slate-800 transition-all shadow-xl shadow-slate-200"
            >
              {editingDevice ? t("devices.form.update_device") : t("devices.form.save_device")}
            </button>
          </div>
        </form>
      </Modal>


        {loading ? (
          <div className="py-20 text-center">
            <div className="animate-spin w-10 h-10 border-4 border-blue-600 border-t-transparent rounded-full mx-auto"></div>
          </div>
        ) : (
          <div className="grid grid-cols-1 gap-4">
            {devices.map((device) => (
              <div
                key={device.id}
                className="bg-white p-6 rounded-3xl border border-slate-100 flex items-center gap-6 hover:shadow-md transition-all group"
              >
                <div className="bg-slate-50 p-4 rounded-2xl group-hover:bg-primary-50 transition-colors">
                  <Cpu className="w-8 h-8 text-slate-400 group-hover:text-blue-500 transition-colors" />
                </div>

                <div className="flex-1 grid grid-cols-1 md:grid-cols-4 gap-4">
                  <div>
                    <p className="text-xs text-slate-400 font-bold uppercase tracking-widest mb-1">
                      {t("devices.table.device")}
                    </p>
                    <h3 className="font-bold text-slate-800 text-lg">
                      {device.name}
                    </h3>
                    <p className="text-xs text-slate-400 font-mono mt-1">
                      {device.id}
                    </p>
                  </div>

                  <div>
                    <p className="text-xs text-slate-400 font-bold uppercase tracking-widest mb-1">
                      {t("devices.table.managed_by")}
                    </p>
                    <div className="flex items-center gap-2">
                      <Server className="w-4 h-4 text-slate-400" />
                      <span className="font-semibold text-slate-700">
                        {device.agent_id}
                      </span>
                    </div>
                  </div>

                  <div>
                    <p className="text-xs text-slate-400 font-bold uppercase tracking-widest mb-1">
                      {t("devices.table.connection")}
                    </p>
                    <div className="flex items-center gap-3">
                      <div className="flex items-center gap-1.5 text-slate-600 bg-slate-50 px-3 py-1 rounded-lg border border-slate-100">
                        <Globe className="w-3.5 h-3.5" />
                        <span className="text-sm font-bold">{device.ip}</span>
                      </div>
                      <div className="flex items-center gap-1 text-slate-400">
                        <Hash className="w-3.5 h-3.5" />
                        <span className="text-sm">{device.port}</span>
                      </div>
                    </div>
                  </div>

                  <div className="flex items-center justify-end gap-6">
                    <div className="flex items-center gap-2">
                      {device.is_enabled ? (
                        <div className="flex items-center gap-1.5 text-emerald-600 bg-emerald-50 px-3 py-1 rounded-full text-xs font-bold">
                          <ShieldCheck className="w-3.5 h-3.5" />
                          {t("devices.table.active")}
                        </div>
                      ) : (
                        <div className="flex items-center gap-1.5 text-slate-400 bg-slate-50 px-3 py-1 rounded-full text-xs font-bold">
                          <ShieldAlert className="w-3.5 h-3.5" />
                          {t("devices.table.disabled")}
                        </div>
                      )}
                    </div>

                    <DropdownMenu>
                      <DropdownMenuTrigger asChild>
                        <button className="text-slate-400 hover:text-slate-600 p-2 hover:bg-slate-100 rounded-xl transition-all outline-none">
                          <MoreVertical className="w-5 h-5" />
                        </button>
                      </DropdownMenuTrigger>
                      <DropdownMenuContent align="end" className="w-48">
                        <DropdownMenuLabel>{t("devices.table.actions")}</DropdownMenuLabel>
                        <DropdownMenuItem 
                          className="gap-2"
                          onSelect={() => setEditingDevice(device)}
                        >
                          <Edit2 className="w-4 h-4 text-slate-400" />
                          {t("devices.table.edit")}
                        </DropdownMenuItem>
                        <DropdownMenuItem className="gap-2">
                          <RefreshCw className="w-4 h-4 text-slate-400" />
                          {t("devices.table.sync_time")}
                        </DropdownMenuItem>
                        <DropdownMenuItem className="gap-2">
                          <Settings className="w-4 h-4 text-slate-400" />
                          {t("devices.table.advanced")}
                        </DropdownMenuItem>
                        <DropdownMenuSeparator />
                        <DropdownMenuItem
                          onSelect={() => handleDelete(device.id)}
                          className="text-rose-600 focus:text-rose-600 focus:bg-rose-50 gap-2 font-bold"
                        >
                          <Trash2 className="w-4 h-4" />
                          {t("devices.table.delete")}
                        </DropdownMenuItem>
                      </DropdownMenuContent>
                    </DropdownMenu>
                  </div>
                </div>
              </div>
            ))}

            {devices.length === 0 && (
              <div className="py-20 text-center bg-white rounded-3xl border border-dashed border-slate-200">
                <Cpu className="w-12 h-12 text-slate-200 mx-auto mb-4" />
                <p className="text-slate-400 font-medium">
                  {t("devices.table.no_data")}
                </p>
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
