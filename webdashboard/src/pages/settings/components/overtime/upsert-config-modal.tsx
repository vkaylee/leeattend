import React, { useEffect } from "react";
import { useForm } from "react-hook-form";
import { useTranslation } from "react-i18next";

import { zodResolver } from "@hookform/resolvers/zod";
import { z } from "zod";

import Button from "@/components/ui/button";
import Modal from "@/components/ui/modal";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import { useUpsertWorkspaceConfig } from "@/hooks/use-workspace-config";
import { type WorkspaceConfigEntry, type CreateConfigEntryRequest } from "@/lib/api/types";

const configSchema = z.object({
  config_key: z.string().min(1, "Key is required"),
  scope_type: z.enum(["workspace", "employer", "department", "employee"]),
  scope_id: z.string().optional(),
  value_type: z.enum(["int", "decimal", "text", "bool", "json"]),
  value: z.string().min(1, "Value is required"),
  effective_from: z.string().min(1, "Effective date is required"),
});

type ConfigFormValues = z.infer<typeof configSchema>;

function parseConfigValue(config: WorkspaceConfigEntry): {
  vType: "int" | "decimal" | "text" | "bool" | "json";
  vVal: string;
} {
  if (config.value_int !== null && config.value_int !== undefined) {
    return { vType: "int", vVal: config.value_int.toString() };
  }
  if (config.value_decimal !== null && config.value_decimal !== undefined) {
    return { vType: "decimal", vVal: config.value_decimal };
  }
  if (typeof config.value_bool === "boolean") {
    return { vType: "bool", vVal: config.value_bool.toString() };
  }
  if (config.value_json !== null && config.value_json !== undefined) {
    return { vType: "json", vVal: JSON.stringify(config.value_json) };
  }
  if (config.value_text !== null && config.value_text !== undefined) {
    return { vType: "text", vVal: config.value_text };
  }
  return { vType: "text", vVal: "" };
}

function useUpsertConfigForm({
  config,
  onClose,
}: {
  config: WorkspaceConfigEntry | null;
  onClose: () => void;
}) {
  const toast = useToast();
  const { mutateAsync: upsertConfig, isPending } = useUpsertWorkspaceConfig();

  const form = useForm<ConfigFormValues>({
    resolver: zodResolver(configSchema),
    defaultValues: {
      config_key: "",
      scope_type: "workspace",
      scope_id: "",
      value_type: "text",
      value: "",
      effective_from: new Date().toISOString().split("T")[0],
    },
  });

  useEffect(() => {
    if (config) {
      const { vType, vVal } = parseConfigValue(config);
      form.reset({
        config_key: config.config_key,
        scope_type: config.scope_type as "workspace" | "employer" | "department" | "employee",
        scope_id: config.scope_id ?? "",
        value_type: vType,
        value: vVal,
        effective_from: config.effective_from.split("T")[0],
      });
    }
  }, [config, form]);

  const onSubmit = async (data: ConfigFormValues) => {
    try {
      const payload: CreateConfigEntryRequest = {
        config_key: data.config_key,
        scope_type: data.scope_type as "workspace" | "department" | "employee",
        effective_from: data.effective_from,
      };

      if (data.scope_type !== "workspace" && data.scope_id) {
        payload.scope_id = data.scope_id;
      }

      switch (data.value_type) {
        case "int":
          payload.value_int = parseInt(data.value, 10);
          break;
        case "decimal":
          payload.value_decimal = parseFloat(data.value);
          break;
        case "bool":
          payload.value_bool = data.value === "true";
          break;
        case "json":
          payload.value_json = JSON.parse(data.value) as Record<string, unknown>;
          break;
        case "text":
          payload.value_text = data.value;
          break;
      }

      await upsertConfig(payload);
      toast.success(config ? "Configuration updated" : "Configuration created");
      onClose();
    } catch (error) {
      console.error(error);
      toast.error("Failed to save configuration");
    }
  };

  const valType = form.watch("value_type");

  return {
    form,
    onSubmit,
    isPending,
    valType,
  };
}

export function UpsertConfigModal({
  config,
  isOpen,
  onClose,
  wsId: _wsId,
}: {
  config: WorkspaceConfigEntry | null;
  isOpen: boolean;
  onClose: () => void;
  wsId: string;
}) {
  const { t } = useTranslation();
  const { form, onSubmit, isPending, valType } = useUpsertConfigForm({ config, onClose });

  return (
    <Modal isOpen={isOpen} onClose={onClose} title={config ? "Edit Configuration" : "New Configuration"}>
      <form className="space-y-6" onSubmit={form.handleSubmit(onSubmit)}>
        <div className="space-y-4">
          <div>
            <label className="block text-xs font-bold text-slate-500 uppercase tracking-widest mb-2" htmlFor="config_key">Key</label>
            <input
              {...form.register("config_key")}
              className="w-full bg-slate-50 dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-xl px-4 py-3 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/50"
              id="config_key"
              placeholder="e.g. overtime.max_hours_per_month"
            />
            {!!form.formState.errors.config_key && <Typography className="text-rose-500 text-xs mt-1" variant="p">{form.formState.errors.config_key.message}</Typography>}
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="block text-xs font-bold text-slate-500 uppercase tracking-widest mb-2" htmlFor="scope_type">Scope</label>
              <select
                {...form.register("scope_type")}
                className="w-full bg-slate-50 dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-xl px-4 py-3 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/50"
                id="scope_type"
              >
                <option value="workspace">Workspace</option>
                <option value="employer">Employer</option>
                <option value="department">Department</option>
                <option value="employee">Employee</option>
              </select>
            </div>
            {form.watch("scope_type") !== "workspace" && (
              <div>
                <label className="block text-xs font-bold text-slate-500 uppercase tracking-widest mb-2" htmlFor="scope_id">Scope ID</label>
                <input
                  {...form.register("scope_id")}
                  className="w-full bg-slate-50 dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-xl px-4 py-3 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/50"
                  id="scope_id"
                  placeholder="UUID"
                />
              </div>
            )}
          </div>

          <div className="grid grid-cols-3 gap-4">
            <div>
              <label className="block text-xs font-bold text-slate-500 uppercase tracking-widest mb-2" htmlFor="value_type">Value Type</label>
              <select
                {...form.register("value_type")}
                className="w-full bg-slate-50 dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-xl px-4 py-3 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/50"
                id="value_type"
              >
                <option value="text">Text</option>
                <option value="int">Integer</option>
                <option value="decimal">Decimal</option>
                <option value="bool">Boolean</option>
                <option value="json">JSON</option>
              </select>
            </div>
            <div className="col-span-2">
              <label className="block text-xs font-bold text-slate-500 uppercase tracking-widest mb-2" htmlFor="value">Value</label>
              {valType === "bool" ? (
                <select
                  {...form.register("value")}
                  className="w-full bg-slate-50 dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-xl px-4 py-3 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/50"
                  id="value"
                >
                  <option value="true">True</option>
                  <option value="false">False</option>
                </select>
              ) : (
                <input
                  {...form.register("value")}
                  className="w-full bg-slate-50 dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-xl px-4 py-3 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/50"
                  id="value"
                  placeholder="Enter value"
                />
              )}
            </div>
          </div>

          <div>
            <label className="block text-xs font-bold text-slate-500 uppercase tracking-widest mb-2" htmlFor="effective_from">Effective From</label>
            <input
              {...form.register("effective_from")}
              className="w-full bg-slate-50 dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-xl px-4 py-3 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/50"
              id="effective_from"
              type="date"
            />
          </div>
        </div>

        <div className="flex justify-end gap-3 pt-4 border-t border-slate-100 dark:border-slate-800">
          <Button onClick={onClose} type="button" variant="ghost">
            {t("common.cancel")}
          </Button>
          <Button loading={isPending} type="submit" variant="primary">
            {t("common.save")}
          </Button>
        </div>
      </form>
    </Modal>
  );
}
