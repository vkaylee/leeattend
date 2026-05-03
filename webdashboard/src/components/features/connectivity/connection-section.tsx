import { Lock } from "lucide-react";
import React from "react";

import { Device } from "@/lib/api";

import { FormField } from "./form-field";

interface SectionProps {
  getValue: (field: keyof Device) => string | number;
  handleChange: (field: string, value: string | number) => void;
  t: (key: string) => string;
}

export function ConnectionSection({ t, getValue, handleChange }: SectionProps) {
  return (
    <>
      <div className="grid grid-cols-3 gap-8">
        <div className="col-span-2">
          <FormField label={t("devices.form.ip_local")}>
            <input
              className="w-full px-5 py-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-mono font-bold text-sm tracking-tight"
              data-testid="device-ip-input"
              onChange={(e) => { handleChange("ip", e.target.value); }}
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
            onChange={(e) => { handleChange("port", parseInt(e.target.value)); }}
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
              onChange={(e) => { handleChange("password", e.target.value); }}
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
