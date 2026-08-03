import { useQueryClient } from "@tanstack/react-query";
import { Activity } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { useToast } from "@/components/ui/toast";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { useDeviceInfo, devicesKeys } from "@/hooks/use-devices";
import { Device, DeviceInfo, DeviceStatus } from "@/lib/api";
import { logError } from "@/lib/logger";
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

interface DeviceBottomBarProps {
  device: Device;
  displayInfo: DeviceInfo | null;
  isToggling: boolean;
  onToggle: () => void;
}

function DeviceBottomBar({ device, displayInfo, isToggling, onToggle }: DeviceBottomBarProps) {
  const { t } = useTranslation();
  const { formatDateTimeLocale } = useDateTimeFormatter();

  return (
    <div className="mt-4 pt-4 border-t border-slate-50 dark:border-slate-800/50 flex items-center justify-between">
      <div className="text-[10px] font-bold text-slate-400">
        {displayInfo ? (
          <span className="text-emerald-500 flex items-center gap-1.5">
            <Activity size={10} /> {t("devices.info.last_synced", { time: displayInfo.last_sync ? formatDateTimeLocale(displayInfo.last_sync) : "—" })}
          </span>
        ) : (
          <span className="italic">{t("devices.info.loading")}</span>
        )}
      </div>
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

  const { data: deviceInfo } = useDeviceInfo(device.id, parentAgentActive && isVisible);
  const displayInfo: DeviceInfo | null = (deviceInfo as DeviceInfo | null) ?? (status?.info as DeviceInfo | null) ?? null;

  async function handleRefresh() {
    if (!onRefresh) return;
    setIsRefreshing(true);
    try {
      onRefresh(device.id);
      await Promise.resolve();
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

      <DeviceBottomBar
        device={device}
        displayInfo={displayInfo}
        isToggling={isToggling}
        onToggle={() => { setShowToggleModal(true); }}
      />

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
