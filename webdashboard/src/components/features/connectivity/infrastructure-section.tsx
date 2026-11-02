import { Cpu, Globe } from "lucide-react";
import React from "react";

import { Combobox } from "@/components/ui/combobox";
import type { Device, CreateDeviceRequest } from "@/lib/api";

import { FormField } from "./form-field";

interface SectionProps {
  getValue: (field: string) => string | number;
  handleChange: (field: string, value: string | number) => void;
  t: (key: string) => string;
}

interface InfrastructureSectionProps extends SectionProps {
  editingDevice: Device | null;
  newDevice: CreateDeviceRequest;
}

export function InfrastructureSection({ t, getValue, handleChange }: InfrastructureSectionProps) {
  const brandOptions = [
    { value: "zkteco", label: "ZKTeco", icon: <Cpu size={14} /> },
    { value: "mock", label: "Mock (Dev)", icon: <Cpu size={14} /> },
  ];
  const timezoneOptions = [
    { value: "Asia/Ho_Chi_Minh", label: "Asia/Ho_Chi_Minh", icon: <Globe size={14} /> },
    { value: "Asia/Bangkok", label: "Asia/Bangkok", icon: <Globe size={14} /> },
    { value: "UTC", label: "UTC", icon: <Globe size={14} /> },
  ];
  return <>
    <FormField label={t("devices.form.clock_name")}><input className="w-full px-5 py-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl font-bold text-sm" data-testid="device-name-input" onChange={(e) => { handleChange("name", e.target.value); }} placeholder={t("devices.form.clock_name_placeholder")} required value={getValue("name")} /></FormField>
    <FormField label={t("devices.form.brand")}><Combobox className="bg-slate-50 dark:bg-slate-800 border-slate-100 h-14" data-testid="device-brand-select" onChange={(value) => { handleChange("brand", value); }} options={brandOptions} placeholder={t("devices.form.brand_placeholder")} value={getValue("brand") as string} /></FormField>
    <FormField label={t("devices.form.timezone")}><Combobox className="bg-slate-50 dark:bg-slate-800 border-slate-100 h-14" data-testid="device-timezone-select" onChange={(value) => { handleChange("timezone", value); }} options={timezoneOptions} placeholder={t("devices.form.timezone_placeholder")} value={getValue("timezone") as string} /></FormField>
  </>;
}
