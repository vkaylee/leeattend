import { Plus } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { Device, DeviceStatus } from "@/lib/api";

import DeviceListItem from "./device-list-item";

interface AgentDeviceListProps {
  agentId: string;
  devices: Device[];
  deviceStatuses?: Record<string, DeviceStatus>;
  isExpanded: boolean;
  isOnline: boolean;
  onAddDevice?: (agentId: string) => void;
  onDeleteDevice: (id: string) => Promise<void>;
  onEditDevice?: (device: Device) => void;
  onRefreshDevice?: (deviceId: string) => void;
  onUpdateDevice: (id: string, data: Partial<Device>) => Promise<void>;
}

export function AgentDeviceList({
  agentId,
  deviceStatuses = {},
  devices,
  isExpanded,
  isOnline,
  onAddDevice,
  onDeleteDevice,
  onEditDevice,
  onRefreshDevice,
  onUpdateDevice,
}: AgentDeviceListProps) {
  const { t } = useTranslation();

  return (
    <div className="flex flex-col gap-4">
      {devices.length === 0 ? (
        <div className="text-center py-8 text-slate-400 dark:text-slate-500 text-xs font-bold uppercase tracking-widest">
          {t("devices.no_devices_for_agent")}
        </div>
      ) : (
        devices.map(device => (
          <DeviceListItem
            device={device}
            isVisible={isExpanded}
            key={device.id}
            onDelete={onDeleteDevice}
            onEdit={onEditDevice}
            onRefresh={onRefreshDevice}
            onUpdate={onUpdateDevice}
            parentAgentActive={isOnline}
            status={deviceStatuses[device.id.toLowerCase()]}
          />
        ))
      )}

      {!!onAddDevice && (
        <Button
          className="group relative flex items-center justify-center p-6 border-2 border-dashed border-slate-200 dark:border-slate-700 rounded-[1.5rem] text-slate-400 dark:text-slate-500 hover:text-primary-600 dark:hover:text-primary-400 hover:border-primary-300 hover:bg-primary-50 dark:bg-primary-950/30 transition-all gap-4 h-auto"
          data-testid={`add-device-btn-${agentId}`}
          onClick={() => { onAddDevice(agentId); }}
          variant="ghost"
        >
          <div className="w-10 h-10 rounded-xl bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 flex items-center justify-center shadow-sm group-hover:scale-110 transition-transform">
            <Plus size={20} />
          </div>
          <span className="text-xs font-black uppercase tracking-widest">{t("devices.add_btn")}</span>
        </Button>
      )}
    </div>
  );
}
