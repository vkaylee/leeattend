import { Cpu, Save, Plus } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import Modal from "@/components/ui/modal";
import { Device, type CreateDeviceRequest } from "@/lib/api";

import { ConnectionSection } from "./connection-section";
import { InfrastructureSection } from "./infrastructure-section";

interface AddEditDeviceModalProps {
  editingDevice: Device | null;
  isOpen: boolean;
  newDevice: CreateDeviceRequest;
  onClose: () => void;
  onNewDeviceChange: (field: string, value: string | number) => void;
  onSave: (e: React.SyntheticEvent) => void;
  onSetEditingDevice: (device: Device | null) => void;
}

export function AddEditDeviceModal({
  editingDevice,
  isOpen,
  newDevice,
  onClose,
  onNewDeviceChange,
  onSave,
  onSetEditingDevice,
}: AddEditDeviceModalProps) {
  const { t } = useTranslation();

  const getValue = (field: string): string | number => {
    if (editingDevice) {
      const val = editingDevice[field as keyof Device];
      return (typeof val === "string" || typeof val === "number") ? val : "";
    }
    return newDevice[field as keyof CreateDeviceRequest] ?? "";
  };

  const handleChange = (field: string, value: string | number) => {
    if (editingDevice) {
      onSetEditingDevice({ ...editingDevice, [field]: value });
    } else {
      onNewDeviceChange(field, value);
    }
  };

  return (
    <Modal
      icon={<Cpu size={24} />}
      isOpen={isOpen}
      maxWidth="lg"
      onClose={onClose}
      subtitle={t("devices.form.infrastructure")}
      title={editingDevice ? t("devices.form.edit_device") : t("devices.form.new_device")}
    >
      <form className="space-y-8 pt-4 animate-in fade-in duration-500" data-testid={editingDevice ? "edit-device-form" : "add-device-form"} onSubmit={onSave}>
        <InfrastructureSection
          editingDevice={editingDevice}
          getValue={getValue}
          handleChange={handleChange}
          newDevice={newDevice}
          t={t}
        />
        <ConnectionSection getValue={getValue} handleChange={handleChange} t={t} />

        <div className="flex gap-4 pt-6">
          <Button
            className="flex-1 h-auto py-4 px-8 rounded-2xl font-bold text-xs uppercase tracking-widest"
            data-testid="device-form-cancel"
            onClick={onClose}
            type="button"
            variant="outline"
          >
            {t("devices.form.cancel")}
          </Button>
          <Button
            className="flex-1 h-auto py-4 px-8 rounded-2xl font-bold uppercase tracking-widest text-xs"
            data-testid="device-form-submit"
            icon={editingDevice ? <Save size={18} /> : <Plus size={18} />}
            type="submit"
            variant="primary"
          >
            {editingDevice ? t("devices.form.update_device") : t("devices.form.save_device")}
          </Button>
        </div>
      </form>
    </Modal>
  );
}
