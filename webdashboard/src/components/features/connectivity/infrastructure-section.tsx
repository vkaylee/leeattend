import { Cpu, Globe, Server } from "lucide-react";
import React from "react";

import { Combobox } from "@/components/ui/combobox";
import { AuthorizedAgent, Device, type CreateDeviceRequest } from "@/lib/api";

import { FormField } from "./form-field";

interface SectionProps {
  getValue: (field: string) => string | number;
  handleChange: (field: string, value: string | number) => void;
  t: (key: string) => string;
}

interface InfrastructureSectionProps extends SectionProps {
  agents: AuthorizedAgent[];
  editingDevice: Device | null;
  newDevice: CreateDeviceRequest;
}

export function InfrastructureSection({ t, getValue, handleChange, agents, editingDevice, newDevice }: InfrastructureSectionProps) {
  const brandOptions = [
    { value: "zkteco", label: "ZKTeco", icon: <Cpu size={14} /> },
    { value: "mock", label: "Mock (Dev)", icon: <Cpu size={14} /> }
  ];

  // The agent resolves the clock's naive wall-clock reading against this zone,
  // so it must match where the hardware physically sits. Getting it wrong shifts
  // every punch from this device by the offset difference.
  const timezoneOptions = [
    { value: "Asia/Ho_Chi_Minh", label: "Asia/Ho_Chi_Minh", icon: <Globe size={14} /> },
    { value: "Asia/Bangkok", label: "Asia/Bangkok", icon: <Globe size={14} /> },
    { value: "UTC", label: "UTC", icon: <Globe size={14} /> }
  ];

  return (
    <>
      <FormField label={t("devices.form.clock_name")}>
        <input
          className="w-full px-5 py-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-bold text-sm tracking-tight shadow-sm"
          data-testid="device-name-input"
          onChange={(e) => { handleChange("name", e.target.value); }}
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
            onChange={(value) => { handleChange("brand", value); }}
            options={brandOptions}
            placeholder={t("devices.form.brand_placeholder")}
            value={getValue("brand") as string}
          />
        </FormField>
      </div>

      <FormField label={t("devices.form.timezone")}>
        <Combobox
          className="bg-slate-50 dark:bg-slate-800 border-slate-100 dark:border-slate-800 h-14"
          data-testid="device-timezone-select"
          onChange={(value) => { handleChange("timezone", value); }}
          options={timezoneOptions}
          placeholder={t("devices.form.timezone_placeholder")}
          value={getValue("timezone") as string}
        />
      </FormField>
    </>
  );
}
