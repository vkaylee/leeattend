import { Cpu, Lock, Save, Plus, Server } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { Combobox } from "@/components/ui/combobox";
import Modal from "@/components/ui/modal";
import { AuthorizedAgent, Device } from "@/lib/api";

interface AddEditDeviceModalProps {
  agents: AuthorizedAgent[];
  editingDevice: Device | null;
  isOpen: boolean;
  newDevice: { name: string; agent_id: string; ip: string; port: number; brand: string; password?: string; timezone: string };
  onClose: () => void;
  onNewDeviceChange: (field: string, value: string | number) => void;
  onSave: (e: React.SyntheticEvent) => void;
  onSetEditingDevice: (device: Device | null) => void;
}

interface FormFieldProps {
  label: string;
  children: React.ReactNode;
}

function FormField({ label, children }: FormFieldProps) {
  return (
    <div className="space-y-3 px-1">
      <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{label}</label>
      {children}
    </div>
  );
}

interface SectionProps {
  t: (key: string) => string;
  getValue: (field: any) => string | number;
  handleChange: (field: string, value: string | number) => void;
}

function InfrastructureSection({ t, getValue, handleChange, agents, editingDevice, newDevice }: SectionProps & { agents: AuthorizedAgent[], editingDevice: Device | null, newDevice: any }) {
  const brandOptions = [
    { value: "zkteco", label: "ZKTeco", icon: <Cpu size={14} /> },
    { value: "mock", label: "Mock (Dev)", icon: <Cpu size={14} /> }
  ];

  return (
    <>
      <FormField label={t("devices.form.clock_name")}>
        <input
          className="w-full px-5 py-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-bold text-sm tracking-tight shadow-sm"
          data-testid="device-name-input"
          onChange={(e) => handleChange("name", e.target.value)}
          placeholder={t("devices.form.clock_name_placeholder")}
          required
          value={getValue("name")}
        />
      </FormField>

      <div className="grid grid-cols-2 gap-8">
        <FormField label={t("devices.form.managed_by")}>
          <div className="w-full px-5 py-4 bg-slate-100 dark:bg-slate-700 border border-slate-200 dark:border-slate-700 rounded-2xl font-bold text-sm uppercase tracking-tight text-slate-600 dark:text-slate-300 flex items-center gap-2">
            <Server className="text-slate-400 dark:text-slate-500" size={14} />
            {agents.find(a => a.id === (editingDevice ? editingDevice.agent_id : newDevice.agent_id))?.name ?? "—"}
          </div>
        </FormField>
        <FormField label={t("devices.form.brand")}>
          <Combobox
            className="bg-slate-50 dark:bg-slate-800 border-slate-100 dark:border-slate-800 h-14"
            data-testid="device-brand-select"
            onChange={(value) => handleChange("brand", value)}
            options={brandOptions}
            placeholder={t("devices.form.brand_placeholder")}
            value={getValue("brand") as string}
          />
        </FormField>
      </div>
    </>
  );
}

function ConnectionSection({ t, getValue, handleChange }: SectionProps) {
  return (
    <>
      <div className="grid grid-cols-3 gap-8">
        <div className="col-span-2">
          <FormField label={t("devices.form.ip_local")}>
            <input
              className="w-full px-5 py-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-mono font-bold text-sm tracking-tight"
              data-testid="device-ip-input"
              onChange={(e) => handleChange("ip", e.target.value)}
              placeholder="192.168.1.100"
              required
              value={getValue("ip")}
            />
          </FormField>
        </div>
        <FormField label={t("devices.form.port")}>
          <input
            className="w-full px-5 py-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-mono font-bold text-sm text-center tracking-tighter"
            data-testid="device-port-input"
            onChange={(e) => handleChange("port", parseInt(e.target.value))}
            required
            type="number"
            value={getValue("port")}
          />
        </FormField>
      </div>

      <div className="grid grid-cols-2 gap-8">
        <FormField label={t("devices.form.password")}>
          <div className="relative">
            <Lock className="absolute left-4 top-1/2 -translate-y-1/2 text-slate-400" size={16} />
            <input
              className="w-full pl-11 pr-5 py-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-mono font-bold text-sm tracking-tight"
              data-testid="device-password-input"
              onChange={(e) => handleChange("password", e.target.value)}
              placeholder={t("devices.form.password_placeholder")}
              type="password"
              value={getValue("password")}
            />
          </div>
        </FormField>
      </div>
    </>
  );
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

  const getValue = (field: keyof typeof newDevice): string | number => {
    if (editingDevice) {
      const val = editingDevice[field as keyof Device];
      return (typeof val === "string" || typeof val === "number") ? val : "";
    }
    return newDevice[field] ?? "";
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
