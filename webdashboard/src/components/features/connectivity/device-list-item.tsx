import { useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { useToast } from "@/components/ui/toast";
import { Device } from "@/lib/api";
import { logError } from "@/lib/logger";
import { cn } from "@/lib/utils";

import { DeviceModals } from "./device-modals";
import { DeviceTopBar } from "./device-top-bar";

interface DeviceListItemProps {
  device: Device;
  onDelete: (id: string) => Promise<void>;
  onEdit?: (device: Device) => void;
  onUpdate: (id: string, data: Partial<Device>) => Promise<void>;
}

interface DeviceBottomBarProps {
  device: Device;
  isToggling: boolean;
  onToggle: () => void;
}

function DeviceBottomBar({ device, isToggling, onToggle }: DeviceBottomBarProps) {
  const { t } = useTranslation();

  return (
    <div className="mt-4 pt-4 border-t border-slate-50 dark:border-slate-800/50 flex items-center justify-end">
      <Button
        className={cn(
          "h-auto px-4 py-1.5 rounded-lg text-[9px] font-black uppercase tracking-widest border",
          device.is_enabled
            ? "text-rose-500 border-rose-100 dark:border-rose-900/50 hover:bg-rose-500 hover:text-white"
            : "bg-emerald-600 text-white border-transparent hover:bg-emerald-700 shadow-none"
        )}
        data-testid={`device-toggle-${device.id}`}
        disabled={isToggling}
        onClick={onToggle}
        variant={device.is_enabled ? "outline" : "primary"}
      >
        {device.is_enabled ? t("devices.info.disable_btn") : t("devices.info.enable_btn")}
      </Button>
    </div>
  );
}

export default function DeviceListItem({ device, onUpdate, onDelete, onEdit }: DeviceListItemProps) {
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
    } catch (e: unknown) {
      logError(e, { component: "DeviceListItem" });
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
    } catch (e: unknown) {
      logError(e, { component: "DeviceListItem" });
    }
  }

  return (
    <div className={cn("relative flex flex-col p-5 rounded-[1.5rem] transition-all group border-2", device.is_enabled ? "bg-white dark:bg-slate-900 border-transparent hover:border-primary-100 dark:hover:border-primary-900 shadow-sm" : "bg-slate-50/50 dark:bg-slate-900/20 border-slate-100 dark:border-slate-800 opacity-60")} data-testid={`device-item-${device.id}`}>
      <DeviceTopBar device={device} onEdit={onEdit} setShowDeleteModal={setShowDeleteModal} />
      <DeviceBottomBar device={device} isToggling={isToggling} onToggle={() => { setShowToggleModal(true); }} />
      <DeviceModals
        device={device}
        handleDeleteConfirm={handleDeleteConfirm}
        handleToggleConfirm={handleToggleConfirm}
        onCloseDelete={() => { setShowDeleteModal(false); }}
        onCloseToggle={() => { setShowToggleModal(false); }}
        showDeleteModal={showDeleteModal}
        showToggleModal={showToggleModal}
      />
    </div>
  );
}
