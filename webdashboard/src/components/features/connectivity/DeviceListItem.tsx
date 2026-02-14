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
  AlertTriangle
} from "lucide-react";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Device } from "@/lib/api";
import { cn } from "@/lib/utils";
import { useToast } from "@/components/ui/Toast";
import Modal from "@/components/ui/Modal";

interface DeviceListItemProps {
  device: Device;
  parentAgentActive?: boolean;
  onUpdate: (id: string, data: Partial<Device>) => Promise<void>;
  onDelete: (id: string) => Promise<void>;
  onEdit?: (device: Device) => void;
}

export default function DeviceListItem({
  device,
  parentAgentActive = true,
  onUpdate,
  onDelete,
  onEdit
}: DeviceListItemProps) {
  const { t } = useTranslation();
  const toast = useToast();
  const [isToggling, setIsToggling] = useState(false);
  const [showToggleModal, setShowToggleModal] = useState(false);
  const [showDeleteModal, setShowDeleteModal] = useState(false);

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
        "flex items-center gap-6 p-4 rounded-2xl transition-all group",
        "bg-slate-50 dark:bg-slate-800/50 hover:bg-slate-100 dark:hover:bg-slate-700/50 border border-transparent hover:border-slate-200 dark:hover:border-slate-700",
        isDimmed && "opacity-50"
      )}
    >
      {/* Device icon */}
      <div className={cn(
        "w-10 h-10 rounded-xl flex items-center justify-center",
        "bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 shadow-inner",
        "group-hover:bg-primary-50 dark:group-hover:bg-primary-950 group-hover:text-primary-600 dark:group-hover:text-primary-400 transition-colors"
      )}>
        <Cpu size={18} className="text-slate-400 dark:text-slate-500 group-hover:text-primary-600 dark:group-hover:text-primary-400" />
      </div>

      {/* Device info */}
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2">
          <span data-testid={`device-name-${device.id}`} className="font-bold text-slate-700 dark:text-slate-200 text-sm uppercase tracking-tight truncate">
            {device.name}
          </span>
          <span data-testid={`device-brand-${device.id}`} className="text-[8px] bg-slate-200 dark:bg-slate-600 px-1.5 py-0.5 rounded font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest shrink-0">
            {device.brand}
          </span>
        </div>
      </div>

      {/* Connection info */}
      <div className="flex items-center gap-3 shrink-0">
        <div className="flex items-center gap-2 text-slate-500 dark:text-slate-400 bg-white dark:bg-slate-900 px-3 py-1.5 rounded-lg border border-slate-100 dark:border-slate-800">
          <Globe size={12} className="text-slate-300 dark:text-slate-600" />
          <span data-testid={`device-ip-${device.id}`} className="text-[11px] font-bold font-mono tracking-tighter">{device.ip}</span>
        </div>
        <div data-testid={`device-port-${device.id}`} className="flex items-center gap-1 text-slate-400 dark:text-slate-500 font-mono font-bold text-[11px]">
          <Hash size={12} /> {device.port}
        </div>
      </div>

      {/* Status badge */}
      <span
        data-testid={`device-status-${device.id}`}
        className={cn(
          "px-3 py-1 rounded-full text-[9px] font-bold uppercase tracking-widest border shadow-sm shrink-0",
          device.is_enabled
            ? "bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 border-emerald-100 dark:border-emerald-800"
            : "bg-slate-100 dark:bg-slate-700 text-slate-500 dark:text-slate-400 border-slate-200 dark:border-slate-700"
        )}
      >
        {device.is_enabled ? t("devices.table.active") : t("devices.table.disabled")}
      </span>

      {/* Toggle button */}
      <button
        data-testid={`device-toggle-${device.id}`}
        onClick={() => setShowToggleModal(true)}
        disabled={isToggling}
        className={cn(
          "px-3 py-1.5 rounded-xl text-[9px] font-bold uppercase tracking-widest transition-all border shadow-sm",
          isToggling && "opacity-50 cursor-not-allowed",
          device.is_enabled
            ? "text-rose-600 dark:text-rose-300 hover:bg-rose-50 dark:bg-rose-950 border-rose-100 dark:border-rose-800 bg-white dark:bg-slate-900"
            : "bg-emerald-600 text-white hover:bg-emerald-700 border-transparent"
        )}
      >
        {isToggling ? (
          <RefreshCw size={12} className="animate-spin" />
        ) : device.is_enabled ? (
          t("common.disable")
        ) : (
          t("common.enable")
        )}
      </button>

      {/* Actions dropdown */}
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <button data-testid={`device-menu-${device.id}`} className="p-2 text-slate-300 dark:text-slate-600 hover:text-primary-600 dark:text-primary-400 rounded-lg transition-colors opacity-0 group-hover:opacity-100">
            <MoreVertical size={16} />
          </button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end" className="w-48 p-2 rounded-xl">
          {onEdit && (
            <DropdownMenuItem
              data-testid={`device-edit-${device.id}`}
              className="gap-3 p-3 rounded-lg font-bold text-xs uppercase"
              onSelect={() => onEdit(device)}
            >
              <Edit2 size={14} className="text-slate-500 dark:text-slate-400" />
              {t("devices.table.edit")}
            </DropdownMenuItem>
          )}
          <DropdownMenuItem
            data-testid={`device-delete-${device.id}`}
            onSelect={() => setShowDeleteModal(true)}
            className="gap-3 p-3 rounded-lg font-bold text-xs uppercase text-rose-600 dark:text-rose-300 focus:text-rose-600 dark:text-rose-300"
          >
            <Trash2 size={14} />
            {t("devices.table.delete")}
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>

      {/* Enable/Disable confirmation modal */}
      <Modal
        isOpen={showToggleModal}
        onClose={() => setShowToggleModal(false)}
        title={t(device.is_enabled ? "devices.modal.disable_title" : "devices.modal.enable_title")}
        icon={<Power size={20} />}
        maxWidth="sm"
        testId="device-toggle-modal"
      >
        <p className="text-sm text-slate-600 dark:text-slate-300 mb-6">
          {t(device.is_enabled ? "devices.confirm_disable" : "devices.confirm_enable", { name: device.name })}
        </p>
        <div className="flex justify-end gap-3">
          <button
            data-testid="device-toggle-modal-cancel"
            onClick={() => setShowToggleModal(false)}
            className="px-4 py-2 text-xs font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-slate-700 rounded-xl transition-colors"
          >
            {t("common.cancel")}
          </button>
          <button
            data-testid="device-toggle-modal-confirm"
            onClick={handleToggleConfirm}
            className={cn(
              "px-4 py-2 text-xs font-bold uppercase tracking-widest rounded-xl transition-colors",
              device.is_enabled
                ? "bg-rose-600 text-white hover:bg-rose-700"
                : "bg-emerald-600 text-white hover:bg-emerald-700"
            )}
          >
            {device.is_enabled ? t("common.disable") : t("common.enable")}
          </button>
        </div>
      </Modal>

      {/* Delete confirmation modal */}
      <Modal
        isOpen={showDeleteModal}
        onClose={() => setShowDeleteModal(false)}
        title={t("devices.modal.delete_title")}
        icon={<AlertTriangle size={20} />}
        maxWidth="sm"
        testId="device-delete-modal"
      >
        <p className="text-sm text-slate-600 dark:text-slate-300 mb-6">
          {t("devices.toast.delete_confirm", { name: device.name })}
        </p>
        <div className="flex justify-end gap-3">
          <button
            data-testid="device-delete-modal-cancel"
            onClick={() => setShowDeleteModal(false)}
            className="px-4 py-2 text-xs font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-slate-700 rounded-xl transition-colors"
          >
            {t("common.cancel")}
          </button>
          <button
            data-testid="device-delete-modal-confirm"
            onClick={handleDeleteConfirm}
            className="px-4 py-2 text-xs font-bold uppercase tracking-widest bg-rose-600 text-white hover:bg-rose-700 rounded-xl transition-colors"
          >
            {t("devices.table.delete")}
          </button>
        </div>
      </Modal>
    </div>
  );
}
