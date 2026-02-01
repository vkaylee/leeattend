import { useState } from "react";
import { useTranslation } from "react-i18next";
import {
  Cpu,
  Globe,
  Hash,
  MoreVertical,
  Edit2,
  Trash2,
  RefreshCw
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

  async function handleToggle() {
    const confirmKey = device.is_enabled ? "devices.confirm_disable" : "devices.confirm_enable";
    if (!confirm(t(confirmKey, { name: device.name }))) return;

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

  async function handleDelete() {
    if (!confirm(t("devices.toast.delete_confirm"))) return;
    try {
      await onDelete(device.id);
      toast.success(t("devices.toast.delete_success"));
    } catch (e) {
      console.error(e);
    }
  }

  const isDimmed = !parentAgentActive || !device.is_enabled;

  return (
    <div className={cn(
      "flex items-center gap-6 p-4 rounded-2xl transition-all group",
      "bg-slate-50/50 hover:bg-slate-100/50 border border-transparent hover:border-slate-200",
      isDimmed && "opacity-50"
    )}>
      {/* Device icon */}
      <div className={cn(
        "w-10 h-10 rounded-xl flex items-center justify-center",
        "bg-white border border-slate-100 shadow-inner",
        "group-hover:bg-primary-50 group-hover:text-primary-600 transition-colors"
      )}>
        <Cpu size={18} className="text-slate-400 group-hover:text-primary-600" />
      </div>

      {/* Device info */}
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2">
          <span className="font-bold text-slate-700 text-sm uppercase tracking-tight truncate">
            {device.name}
          </span>
          <span className="text-[8px] bg-slate-200 px-1.5 py-0.5 rounded font-bold text-slate-500 uppercase tracking-widest shrink-0">
            {device.brand}
          </span>
        </div>
      </div>

      {/* Connection info */}
      <div className="flex items-center gap-3 shrink-0">
        <div className="flex items-center gap-2 text-slate-500 bg-white px-3 py-1.5 rounded-lg border border-slate-100">
          <Globe size={12} className="text-slate-300" />
          <span className="text-[11px] font-bold font-mono tracking-tighter">{device.ip}</span>
        </div>
        <div className="flex items-center gap-1 text-slate-400 font-mono font-bold text-[11px]">
          <Hash size={12} /> {device.port}
        </div>
      </div>

      {/* Status badge */}
      <span className={cn(
        "px-3 py-1 rounded-full text-[9px] font-bold uppercase tracking-widest border shadow-sm shrink-0",
        device.is_enabled
          ? "bg-emerald-50 text-emerald-600 border-emerald-100"
          : "bg-slate-100 text-slate-500 border-slate-200"
      )}>
        {device.is_enabled ? t("devices.table.active") : t("devices.table.disabled")}
      </span>

      {/* Toggle button */}
      <button
        onClick={handleToggle}
        disabled={isToggling}
        className={cn(
          "px-3 py-1.5 rounded-xl text-[9px] font-bold uppercase tracking-widest transition-all border shadow-sm",
          isToggling && "opacity-50 cursor-not-allowed",
          device.is_enabled
            ? "text-rose-600 hover:bg-rose-50 border-rose-100 bg-white"
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
          <button className="p-2 text-slate-300 hover:text-primary-600 rounded-lg transition-colors opacity-0 group-hover:opacity-100">
            <MoreVertical size={16} />
          </button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end" className="w-48 p-2 rounded-xl">
          {onEdit && (
            <DropdownMenuItem
              className="gap-3 p-3 rounded-lg font-bold text-xs uppercase"
              onSelect={() => onEdit(device)}
            >
              <Edit2 size={14} className="text-slate-500" />
              {t("devices.table.edit")}
            </DropdownMenuItem>
          )}
          <DropdownMenuItem
            onSelect={handleDelete}
            className="gap-3 p-3 rounded-lg font-bold text-xs uppercase text-rose-600 focus:text-rose-600"
          >
            <Trash2 size={14} />
            {t("devices.table.delete")}
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    </div>
  );
}
