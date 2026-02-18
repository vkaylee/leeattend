import { useState } from "react";
import { useTranslation } from "react-i18next";
import {
  Cpu,
  Globe,
  Hash,
  MoreVertical,
  Edit2,
  Trash2,
  RefreshCw,
  Power,
  AlertTriangle,
  User,
  ClipboardList,
  Terminal,
  Activity,
  Clock
} from "lucide-react";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Device, DeviceStatus } from "@/lib/api";
import { cn } from "@/lib/utils";
import { useToast } from "@/components/ui/Toast";
import Modal from "@/components/ui/Modal";

interface DeviceListItemProps {
  device: Device;
  status?: DeviceStatus;
  parentAgentActive?: boolean;
  onUpdate: (id: string, data: Partial<Device>) => Promise<void>;
  onDelete: (id: string) => Promise<void>;
  onEdit?: (device: Device) => void;
  onRefresh?: (deviceId: string) => void;
}

export default function DeviceListItem({
  device,
  status,
  parentAgentActive = true,
  onUpdate,
  onDelete,
  onEdit,
  onRefresh
}: DeviceListItemProps) {
  const { t } = useTranslation();
  const toast = useToast();
  const [isToggling, setIsToggling] = useState(false);
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [showToggleModal, setShowToggleModal] = useState(false);
  const [showDeleteModal, setShowDeleteModal] = useState(false);

  async function handleRefresh() {
    if (!onRefresh) return;
    setIsRefreshing(true);
    try {
      await onRefresh(device.id);
    } finally {
      setTimeout(() => setIsRefreshing(false), 2000);
    }
  }

  async function handleToggleConfirm() {
    setShowToggleModal(false);
    setIsToggling(true);
    try {
      await onUpdate(device.id, { is_enabled: !device.is_enabled });
      toast.success(t(device.is_enabled ? "devices.toast.disabled" : "devices.toast.enabled", { name: device.name }));
    } catch (e) {
      console.error(e);
      toast.error(t("devices.toast.toggle_error"));
    } finally {
      setIsToggling(false);
    }
  }

  async function handleDeleteConfirm() {
    setShowDeleteModal(false);
    try {
      await onDelete(device.id);
      toast.success(t("devices.toast.delete_success"));
    } catch (e) {
      console.error(e);
    }
  }

  const isDimmed = !parentAgentActive || !device.is_enabled;

  return (
    <div
      data-testid={`device-item-${device.id}`}
      className={cn(
        "relative flex flex-col p-5 rounded-[1.5rem] transition-all group border-2",
        isDimmed 
          ? "bg-slate-50/50 dark:bg-slate-900/20 border-slate-100 dark:border-slate-800 opacity-60" 
          : "bg-white dark:bg-slate-900 border-transparent hover:border-primary-100 dark:hover:border-primary-900 shadow-sm",
      )}
    >
      {/* 1. TOP BAR: Name, IP, Status & Menu */}
      <div className="flex items-center justify-between mb-4 pb-4 border-b border-slate-50 dark:border-slate-800/50">
        <div className="flex items-center gap-3 min-w-0">
          <div className={cn(
            "w-10 h-10 rounded-xl flex items-center justify-center border shadow-inner shrink-0",
            status?.status === "online" 
              ? "bg-emerald-50 dark:bg-emerald-950/50 text-emerald-600 border-emerald-100 dark:border-emerald-800"
              : "bg-slate-100 dark:bg-slate-800 text-slate-400 border-slate-200 dark:border-slate-700"
          )}>
            <Cpu size={20} className={cn(status?.status === "online" && "animate-pulse")} />
          </div>
          <div className="min-w-0">
            <h3 className="font-black text-slate-700 dark:text-slate-200 text-sm uppercase tracking-tight truncate">
              {device.name}
            </h3>
            <div className="flex items-center gap-2 text-[10px] font-mono font-bold text-slate-400">
              <Globe size={10} /> {device.ip}
            </div>
          </div>
        </div>

        <div className="flex items-center gap-3">
          <span className={cn(
            "px-2.5 py-0.5 rounded-full text-[8px] font-black uppercase tracking-widest border",
            status?.status === "online"
              ? "bg-emerald-500 text-white border-transparent"
              : "bg-slate-100 dark:bg-slate-800 text-slate-500 border-slate-200 dark:border-slate-700"
          )}>
            {status?.status === "online" ? "Online" : "Offline"}
          </span>
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <button className="p-2 text-slate-400 hover:bg-slate-100 dark:hover:bg-slate-800 rounded-lg transition-colors">
                <MoreVertical size={16} />
              </button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end" className="w-52 p-2 rounded-xl shadow-2xl">
              <DropdownMenuItem className="gap-3 p-3 rounded-lg font-bold text-xs uppercase" onSelect={handleRefresh} disabled={isRefreshing}>
                <RefreshCw size={14} className={cn(isRefreshing && "animate-spin")} />
                {isRefreshing ? t("common.loading") : t("common.refresh")}
              </DropdownMenuItem>
              {onEdit && (
                <DropdownMenuItem className="gap-3 p-3 rounded-lg font-bold text-xs uppercase" onSelect={() => onEdit(device)}>
                  <Edit2 size={14} /> {t("devices.table.edit")}
                </DropdownMenuItem>
              )}
              <DropdownMenuItem className="gap-3 p-3 rounded-lg font-bold text-xs uppercase text-rose-600 focus:text-rose-600" onSelect={() => setShowDeleteModal(true)}>
                <Trash2 size={14} /> {t("devices.table.delete")}
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      </div>

      {/* 2. INFO WRAPPER: Responsive flex layout to prevent overflow */}
      <div className="flex flex-wrap gap-y-6 gap-x-8">
        {/* Users */}
        <div className="flex flex-col gap-1 min-w-[80px]">
          <span className="text-[8px] font-bold text-slate-400 uppercase tracking-widest">Users</span>
          <div className="flex items-center gap-1.5">
            <User size={12} className="text-slate-300" />
            <span className="text-sm font-black text-slate-700 dark:text-slate-200">{status?.info ? status.info.users_count : "--"}</span>
          </div>
        </div>

        {/* Logs */}
        <div className="flex flex-col gap-1 min-w-[80px]">
          <span className="text-[8px] font-bold text-slate-400 uppercase tracking-widest">Logs</span>
          <div className="flex items-center gap-1.5">
            <ClipboardList size={12} className="text-amber-400" />
            <span className="text-sm font-black text-amber-600">
              {!status ? "--" : (typeof status.info?.logs_count === 'number' && status.info.logs_count >= 0 ? status.info.logs_count : "N/A")}
            </span>
          </div>
        </div>

        {/* Device Time - Moved up in priority to avoid being the last item cut off */}
        <div className="flex flex-col gap-1 min-w-[150px]">
          <span className="text-[8px] font-bold text-slate-400 uppercase tracking-widest">Device Time</span>
          <div className="flex items-center gap-1.5 text-primary-600 dark:text-primary-400">
            <Clock size={12} />
            <span className="text-[10px] font-mono font-black tracking-tighter">{status?.info?.device_time || "--:--:--"}</span>
          </div>
        </div>

        {/* Serial Number */}
        <div className="flex flex-col gap-1 min-w-[130px]">
          <span className="text-[8px] font-bold text-slate-400 uppercase tracking-widest">Serial Number</span>
          <div className="flex items-center gap-1.5">
            <Terminal size={12} className="text-slate-300" />
            <span className="text-[10px] font-mono font-bold text-slate-600 dark:text-slate-400 truncate">{status?.info?.serial || "N/A"}</span>
          </div>
        </div>

        {/* MAC Address */}
        <div className="flex flex-col gap-1 min-w-[130px]">
          <span className="text-[8px] font-bold text-slate-400 uppercase tracking-widest">MAC Address</span>
          <div className="flex items-center gap-1.5">
            <Globe size={12} className="text-slate-300" />
            <span className="text-[10px] font-mono font-bold text-slate-600 dark:text-slate-400 truncate">{status?.info?.mac || "N/A"}</span>
          </div>
        </div>

        {/* HW & FW */}
        <div className="flex flex-col gap-1 min-w-[120px]">
          <span className="text-[8px] font-bold text-slate-400 uppercase tracking-widest">Hardware / FW</span>
          <div className="flex flex-wrap gap-1">
            <span className="text-[9px] font-black px-1.5 py-0.5 bg-slate-100 dark:bg-slate-800 rounded text-slate-500 border border-slate-200/50 dark:border-slate-700/50">{status?.info?.hw_name || "N/A"}</span>
            <span className="text-[9px] font-black px-1.5 py-0.5 bg-primary-50 dark:bg-primary-950 text-primary-600 rounded border border-primary-100/50">v{status?.info?.firmware || "?.?"}</span>
          </div>
        </div>
      </div>

      {/* 3. FOOTER: Quick Toggle */}
      <div className="mt-4 pt-4 border-t border-slate-50 dark:border-slate-800/50 flex items-center justify-between">
        <div className="text-[10px] font-bold text-slate-400">
          {status?.info ? (
            <span className="text-emerald-500 flex items-center gap-1.5">
              <Activity size={10} /> Dữ liệu đồng bộ lúc {status.info.last_sync}
            </span>
          ) : (
            <span className="italic">Đang đợi Agent phản hồi...</span>
          )}
        </div>
        <button
          onClick={() => setShowToggleModal(true)}
          disabled={isToggling}
          className={cn(
            "px-4 py-1.5 rounded-lg text-[9px] font-black uppercase tracking-widest transition-all border",
            device.is_enabled
              ? "text-rose-500 border-rose-100 dark:border-rose-900/50 hover:bg-rose-500 hover:text-white"
              : "bg-emerald-600 text-white border-transparent hover:bg-emerald-700"
          )}
        >
          {device.is_enabled ? "Tắt thiết bị" : "Bật thiết bị"}
        </button>
      </div>

      {/* Modals */}
      <Modal isOpen={showToggleModal} onClose={() => setShowToggleModal(false)} title={t(device.is_enabled ? "devices.modal.disable_title" : "devices.modal.enable_title")} icon={<Power size={20} />} maxWidth="sm">
        <p className="text-sm text-slate-600 dark:text-slate-300 mb-6">{t(device.is_enabled ? "devices.confirm_disable" : "devices.confirm_enable", { name: device.name })}</p>
        <div className="flex justify-end gap-3">
          <button onClick={() => setShowToggleModal(false)} className="px-4 py-2 text-xs font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400 rounded-xl transition-colors">{t("common.cancel")}</button>
          <button onClick={handleToggleConfirm} className={cn("px-4 py-2 text-xs font-bold uppercase tracking-widest rounded-xl transition-colors", device.is_enabled ? "bg-rose-600 text-white" : "bg-emerald-600 text-white")}>
            {device.is_enabled ? t("common.disable") : t("common.enable")}
          </button>
        </div>
      </Modal>

      <Modal isOpen={showDeleteModal} onClose={() => setShowDeleteModal(false)} title={t("devices.modal.delete_title")} icon={<AlertTriangle size={20} />} maxWidth="sm">
        <p className="text-sm text-slate-600 dark:text-slate-300 mb-6">{t("devices.toast.delete_confirm", { name: device.name })}</p>
        <div className="flex justify-end gap-3">
          <button onClick={() => setShowDeleteModal(false)} className="px-4 py-2 text-xs font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400 rounded-xl transition-colors">{t("common.cancel")}</button>
          <button onClick={handleDeleteConfirm} className="px-4 py-2 text-xs font-bold uppercase tracking-widest bg-rose-600 text-white rounded-xl transition-colors">{t("devices.table.delete")}</button>
        </div>
      </Modal>
    </div>
  );
}
