import { AlertCircle } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { Typography } from "@/components/ui/typography";
import { Device } from "@/lib/api";

import DeviceListItem from "./device-list-item";

interface UnassignedDevicesSectionProps {
  onDeleteDevice: (id: string) => Promise<void>;
  onEditDevice: (device: Device) => void;
  onUpdateDevice: (id: string, data: Partial<Device>) => Promise<void>;
  unassignedDevices: Device[];
}

export function UnassignedDevicesSection({
  onDeleteDevice,
  onEditDevice,
  onUpdateDevice,
  unassignedDevices,
}: UnassignedDevicesSectionProps) {
  const { t } = useTranslation();

  if (unassignedDevices.length === 0) return null;

  return (
    <div className="mt-8 animate-in fade-in duration-500" data-testid="unassigned-devices-section">
      <div className="flex items-center gap-3 mb-4 px-2">
        <AlertCircle className="text-amber-500" size={16} />
        <Typography className="text-xs font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400" variant="h3">
          {t("devices.unassigned_title")} ({unassignedDevices.length})
        </Typography>
      </div>
      <div className="bg-amber-50 dark:bg-amber-950/50 border border-amber-100 dark:border-amber-800 rounded-[2.5rem] p-6 space-y-3" data-testid="unassigned-devices-list">
        {unassignedDevices.map(device => (
          <DeviceListItem
            device={device}
            key={device.id}
            onDelete={onDeleteDevice}
            onEdit={onEditDevice}
            onUpdate={onUpdateDevice}
          />
        ))}
      </div>
    </div>
  );
}
