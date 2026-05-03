import { useQueryClient } from "@tanstack/react-query";
import { Activity } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import { useToast } from "@/components/ui/toast";
import { useDeviceInfo, devicesKeys } from "@/hooks/use-devices";
import { Device, DeviceInfo, DeviceStatus } from "@/lib/api";
import { cn } from "@/lib/utils";

import { DeviceModals } from "./device-modals";
import { DeviceStats } from "./device-stats";
import { DeviceTopBar } from "./device-top-bar";

interface DeviceListItemProps {
  device: Device;
  isVisible?: boolean;
  onDelete: (id: string) => Promise<void>;
  onEdit?: (device: Device) => void;
  onRefresh?: (deviceId: string) => void;
  onUpdate: (id: string, data: Partial<Device>) => Promise<void>;
  parentAgentActive?: boolean;
  status?: DeviceStatus;
}

export default function DeviceListItem({
  device,
  status,
  parentAgentActive = true,
  onUpdate,
  onDelete,
  onEdit,
  onRefresh,
  isVisible = true
}: DeviceListItemProps) {
  const { t } = useTranslation();
  const toast = useToast();
  const queryClient = useQueryClient();
  const [isToggling, setIsToggling] = useState(false);
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [showToggleModal, setShowToggleModal] = useState(false);
  const [showDeleteModal, setShowDeleteModal] = useState(false);

  // Fetch detailed info automatically on mount (if agent is active)
  const { data: deviceInfo } = useDeviceInfo(device.id, parentAgentActive && isVisible);

  // Merge status.info (heartbeat data) with deviceInfo (on-demand data)
  const displayInfo: DeviceInfo | null = deviceInfo ?? status?.info ?? null;

  async function handleRefresh() {
    if (!onRefresh) return;
    setIsRefreshing(true);
    try {
      await Promise.resolve(onRefresh(device.id));
      // Invalidate the query to trigger a refetch after the agent has time to respond
      setTimeout(() => {
        void queryClient.invalidateQueries({ queryKey: devicesKeys.info(device.id) });
      }, 2000);
    } finally {
      setTimeout(() => { setIsRefreshing(false); }, 2000);
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
      className={cn(
        "relative flex flex-col p-5 rounded-[1.5rem] transition-all group border-2",
        isDimmed
          ? "bg-slate-50/50 dark:bg-slate-900/20 border-slate-100 dark:border-slate-800 opacity-60"
          : "bg-white dark:bg-slate-900 border-transparent hover:border-primary-100 dark:hover:border-primary-900 shadow-sm",
      )}
      data-testid={`device-item-${device.id}`}
    >
      <DeviceTopBar
        device={device}
        handleRefresh={handleRefresh}
        isRefreshing={isRefreshing}
        onEdit={onEdit}
        setShowDeleteModal={setShowDeleteModal}
        status={status}
      />

      <DeviceStats displayInfo={displayInfo} />

      <div className="mt-4 pt-4 border-t border-slate-50 dark:border-slate-800/50 flex items-center justify-between">
        <div className="text-[10px] font-bold text-slate-400">
          {displayInfo ? (
            <span className="text-emerald-500 flex items-center gap-1.5">
              <Activity size={10} /> {t("devices.info.last_synced", { time: displayInfo.last_sync ? new Date(displayInfo.last_sync).toLocaleString() : "—" })}
            </span>
          ) : (
            <span className="italic">{t("devices.info.loading")}</span>
          )}
        </div>
        <button
          className={cn(
            "px-4 py-1.5 rounded-lg text-[9px] font-black uppercase tracking-widest transition-all border",
            device.is_enabled
              ? "text-rose-500 border-rose-100 dark:border-rose-900/50 hover:bg-rose-500 hover:text-white"
              : "bg-emerald-600 text-white border-transparent hover:bg-emerald-700"
          )}
          data-testid={`device-toggle-${device.id}`}
          disabled={isToggling}
          onClick={() => { setShowToggleModal(true); }}
        >
          {device.is_enabled ? t("devices.info.disable_btn") : t("devices.info.enable_btn")}
        </button>
      </div>

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

