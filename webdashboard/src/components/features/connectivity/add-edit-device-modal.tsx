import { Cpu, Save, Plus } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Modal from "@/components/ui/modal";
import { AuthorizedAgent, Device } from "@/lib/api";

import { ConnectionSection } from "./connection-section";
import { InfrastructureSection } from "./infrastructure-section";

interface AddEditDeviceModalProps {
  agents: AuthorizedAgent[];
  editingDevice: Device | null;
  isOpen: boolean;
  newDevice: Partial<Device>;
  onClose: () => void;
  onNewDeviceChange: (field: string, value: string | number) => void;
  onSave: (e: React.SyntheticEvent) => void;
  onSetEditingDevice: (device: Device | null) => void;
}

export function AddEditDeviceModal({
  agents,
  editingDevice,
  isOpen,
  newDevice,
  onClose,
  onNewDeviceChange,
  onSave,
  onSetEditingDevice,
}: AddEditDeviceModalProps) {
  const { t } = useTranslation();

  const getValue = (field: keyof Device): string | number => {
    if (editingDevice) {
      const val = editingDevice[field];
      return (typeof val === "string" || typeof val === "number") ? val : "";
    }
    return (newDevice[field] as string | number) || "";
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
          agents={agents}
          editingDevice={editingDevice}
          getValue={getValue}
          handleChange={handleChange}
          newDevice={newDevice}
          t={t}
        />
        <ConnectionSection getValue={getValue} handleChange={handleChange} t={t} />

        <div className="flex gap-4 pt-6">
          <button
            className="flex-1 px-8 py-4 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-2xl font-bold text-xs uppercase tracking-widest text-slate-500 dark:text-slate-400 hover:bg-slate-50 dark:hover:bg-slate-800 transition-all active:scale-95"
            data-testid="device-form-cancel"
            onClick={onClose}
            type="button"
          >
            {t("devices.form.cancel")}
          </button>
          <button
            className="flex-1 bg-primary-600 text-white px-8 py-4 rounded-2xl font-bold uppercase tracking-widest text-xs hover:bg-primary-700 transition-all shadow-xl shadow-primary-100 dark:shadow-black/30 active:scale-95 flex items-center justify-center gap-3"
            data-testid="device-form-submit"
            type="submit"
          >
            {editingDevice ? <Save size={18} /> : <Plus size={18} />}
            {editingDevice ? t("devices.form.update_device") : t("devices.form.save_device")}
          </button>
        </div>
      </form>
    </Modal>
  );
}
