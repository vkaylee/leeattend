import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import { AddEditDeviceModal } from "@/components/features/connectivity/add-edit-device-modal";
import DeviceListItem from "@/components/features/connectivity/device-list-item";
import Button from "@/components/ui/button";
import { useDevices } from "@/hooks/use-devices";
import { useWorkspace } from "@/hooks/use-workspace";
import type { CreateDeviceRequest, Device } from "@/lib/api";

const emptyDevice = (timezone?: string): CreateDeviceRequest => ({
  name: "", agent_id: "", ip: "", port: 4370, brand: "zkteco", password: "", timezone: timezone ?? "UTC",
});

export default function DevicesSettings() {
  const { t } = useTranslation();
  const { workspace } = useWorkspace();
  const { devices, isLoading, createDevice, updateDevice, deleteDevice } = useDevices();
  const [editingDevice, setEditingDevice] = useState<Device | null>(null);
  const [newDevice, setNewDevice] = useState(() => emptyDevice(workspace?.timezone));
  const [isOpen, setIsOpen] = useState(false);

  const openCreate = () => { setEditingDevice(null); setNewDevice(emptyDevice(workspace?.timezone)); setIsOpen(true); };
  const openEdit = (device: Device) => { setEditingDevice(device); setIsOpen(true); };
  const save = async (event: React.SyntheticEvent) => {
    event.preventDefault();
    if (editingDevice) {
      await updateDevice(editingDevice.id, {
        brand: editingDevice.brand, ip: editingDevice.ip, is_enabled: editingDevice.is_enabled,
        location_id: editingDevice.location_id ?? null, name: editingDevice.name,
        password: editingDevice.password ?? null, port: editingDevice.port, timezone: editingDevice.timezone ?? null,
      });
    } else await createDevice(newDevice);
    setIsOpen(false); setEditingDevice(null);
  };

  return (
    <div className="p-4 sm:p-8 space-y-8 max-w-[1600px] mx-auto" data-testid="devices-settings">
      <div className="flex items-center justify-between gap-4">
        <div><h1 className="text-3xl font-bold tracking-tight text-slate-700 dark:text-slate-200">{t("common.devices")}</h1><p className="text-slate-500 dark:text-slate-400 mt-1">{t("devices.form.infrastructure")}</p></div>
        <Button data-testid="devices-add-button" onClick={openCreate} variant="primary">{t("devices.add_btn")}</Button>
      </div>
      {isLoading ? <div className="animate-pulse text-slate-400 font-bold uppercase tracking-widest">{t("common.loading")}</div> : (
        <div className="grid gap-4">{devices.map(device => <DeviceListItem device={device} key={device.id} onDelete={deleteDevice} onEdit={openEdit} onUpdate={updateDevice} />)}</div>
      )}
      <AddEditDeviceModal editingDevice={editingDevice} isOpen={isOpen} newDevice={newDevice} onClose={() => { setIsOpen(false); setEditingDevice(null); }} onNewDeviceChange={(field, value) => { setNewDevice(prev => ({ ...prev, [field]: value })); }} onSave={save} onSetEditingDevice={setEditingDevice} />
    </div>
  );
}
