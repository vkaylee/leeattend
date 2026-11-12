import React, { useEffect } from "react";
import { useForm } from "react-hook-form";
import { useTranslation } from "react-i18next";

import { zodResolver } from "@hookform/resolvers/zod";
import { z } from "zod";

import Button from "@/components/ui/button";
import Modal from "@/components/ui/modal";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import { useUpsertOrganizationConfig } from "@/hooks/use-organization-config";
import { type OrganizationConfigEntry, type CreateConfigEntryRequest } from "@/lib/api/types";
import { logError } from "@/lib/logger";

const configSchema = z.object({
  config_key: z.string().min(1, "settings.overtime.config.errors.key_required"),
  scope_type: z.enum(["organization", "employer", "department", "employee", "employment"]),
  scope_id: z.string().optional(),
  value_type: z.enum(["int", "decimal", "text", "bool", "json"]),
  value: z.string().min(1, "settings.overtime.config.errors.value_required"),
  effective_from: z.string().min(1, "settings.overtime.config.errors.effective_from_required"),
});

type ConfigFormValues = z.infer<typeof configSchema>;

function parseConfigValue(config: OrganizationConfigEntry): {
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
  config: OrganizationConfigEntry | null;
  onClose: () => void;
}) {
  const { t } = useTranslation(["settings", "common"]);
  const toast = useToast();
  const { mutateAsync: upsertConfig, isPending } = useUpsertOrganizationConfig();

  const form = useForm<ConfigFormValues>({
    resolver: zodResolver(configSchema),
    defaultValues: config
      ? {
          config_key: config.config_key,
          scope_type: config.scope_type as ConfigFormValues["scope_type"],
          scope_id: config.scope_id ?? "",
          value_type: parseConfigValue(config).vType,
          value: parseConfigValue(config).vVal,
          effective_from: config.effective_from.split("T")[0],
        }
      : {
          config_key: "",
          scope_type: "organization",
          scope_id: "",
          value_type: "text",
          value: "",
          effective_from: new Date().toISOString().split("T")[0],
        },
  });

  useEffect(() => {
    if (config) {
      form.reset({
        config_key: config.config_key,
        scope_type: config.scope_type as ConfigFormValues["scope_type"],
        scope_id: config.scope_id ?? "",
        value_type: parseConfigValue(config).vType,
        value: parseConfigValue(config).vVal,
        effective_from: config.effective_from.split("T")[0],
      });
    }
  }, [config, form]);

  const onSubmit = async (data: ConfigFormValues) => {
    try {
      let scopeId: string | null = null;
      if (data.scope_type !== "organization" && data.scope_id) {
        scopeId = data.scope_id;
      }

      const payload: CreateConfigEntryRequest = {
        config_key: data.config_key,
        scope_type: data.scope_type,
        scope_id: scopeId,
        effective_from: data.effective_from,
      };

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
          // value_json is typed as unknown on CreateConfigEntryRequest;
          // the JSON value is validated by the zod schema at form level.
          payload.value_json = JSON.parse(data.value) as unknown;
          break;
        case "text":
          payload.value_text = data.value;
          break;
      }

      await upsertConfig(payload);
      toast.success(config ? t("settings.overtime.config.success_updated") : t("settings.overtime.config.success_created"));
      onClose();
    } catch (error) {
      logError(error, { component: "UpsertConfigModal" });
      toast.error(t("settings.overtime.config.failed_save"));
    }
  };

  // eslint-disable-next-line react-hooks/incompatible-library
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
}: {
  config: OrganizationConfigEntry | null;
  isOpen: boolean;
  onClose: () => void;
}) {
  const { t } = useTranslation(["settings", "common"]);
  const { form, onSubmit, isPending, valType } = useUpsertConfigForm({ config, onClose });

  return (
    <Modal isOpen={isOpen} onClose={onClose} title={config ? t("settings.overtime.config.edit_title") : t("settings.overtime.config.new_title")}>
      <form className="space-y-6" onSubmit={form.handleSubmit(onSubmit)}>
        <div className="space-y-4">
          <div>
            <label className="block text-xs font-bold text-slate-500 uppercase tracking-widest mb-2" htmlFor="config_key">{t("settings.overtime.config.key")}</label>
            <input
              {...form.register("config_key")}
              className="w-full bg-slate-50 dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-xl px-4 py-3 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/50"
              id="config_key"
              placeholder={t("settings.overtime.config.key_placeholder")}
            />
            {!!form.formState.errors.config_key && <Typography className="text-rose-500 text-xs mt-1" variant="p">{t(form.formState.errors.config_key.message ?? "")}</Typography>}
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="block text-xs font-bold text-slate-500 uppercase tracking-widest mb-2" htmlFor="scope_type">{t("settings.overtime.config.scope")}</label>
              <select
                {...form.register("scope_type")}
                className="w-full bg-slate-50 dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-xl px-4 py-3 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/50"
                id="scope_type"
              >
                <option value="organization">{t("settings.overtime.config.scopes.organization", { defaultValue: "Organization" })}</option>
                <option value="employer">{t("settings.overtime.config.scopes.employer")}</option>
                <option value="department">{t("settings.overtime.config.scopes.department")}</option>
                <option value="employee">{t("settings.overtime.config.scopes.employee")}</option>
              </select>
            </div>
            {form.watch("scope_type") !== "organization" && (
              <div>
                <label className="block text-xs font-bold text-slate-500 uppercase tracking-widest mb-2" htmlFor="scope_id">{t("settings.overtime.config.scope_id")}</label>
                <input
                  {...form.register("scope_id")}
                  className="w-full bg-slate-50 dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-xl px-4 py-3 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/50"
                  id="scope_id"
                  placeholder={t("settings.overtime.config.scope_id_placeholder")}
                />
              </div>
            )}
          </div>

          <div className="grid grid-cols-3 gap-4">
            <div>
              <label className="block text-xs font-bold text-slate-500 uppercase tracking-widest mb-2" htmlFor="value_type">{t("settings.overtime.config.value_type")}</label>
              <select
                {...form.register("value_type")}
                className="w-full bg-slate-50 dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-xl px-4 py-3 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/50"
                id="value_type"
              >
                <option value="text">{t("settings.overtime.config.types.text")}</option>
                <option value="int">{t("settings.overtime.config.types.int")}</option>
                <option value="decimal">{t("settings.overtime.config.types.decimal")}</option>
                <option value="bool">{t("settings.overtime.config.types.bool")}</option>
                <option value="json">{t("settings.overtime.config.types.json")}</option>
              </select>
            </div>
            <div className="col-span-2">
              <label className="block text-xs font-bold text-slate-500 uppercase tracking-widest mb-2" htmlFor="value">{t("settings.overtime.config.value")}</label>
              {valType === "bool" ? (
                <select
                  {...form.register("value")}
                  className="w-full bg-slate-50 dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-xl px-4 py-3 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/50"
                  id="value"
                >
                  <option value="true">{t("settings.overtime.config.bool_true")}</option>
                  <option value="false">{t("settings.overtime.config.bool_false")}</option>
                </select>
              ) : (
                <input
                  {...form.register("value")}
                  className="w-full bg-slate-50 dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-xl px-4 py-3 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/50"
                  id="value"
                  placeholder={t("settings.overtime.config.value_placeholder")}
                />
              )}
            </div>
          </div>

          <div>
            <label className="block text-xs font-bold text-slate-500 uppercase tracking-widest mb-2" htmlFor="effective_from">{t("settings.overtime.config.effective_from")}</label>
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
