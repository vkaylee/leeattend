import type { components } from "@/types/api-generated";

import type { CreateConfigEntryRequest, OrganizationConfigEntry } from "./types";

import { client } from "./client";

export type ScopeType = "organization" | "employer" | "department" | "employee" | "employment";

interface ConfigValue {
  Bool?: boolean;
  Decimal?: number;
  Int?: number;
  Json?: unknown;
  Text?: string;
}

export const organizationConfigApi = {
  listConfigs: async (prefix?: string): Promise<OrganizationConfigEntry[]> => {
    const { data } = await client.GET<OrganizationConfigEntry[]>("/api/v1/configs", {
      params: { query: prefix ? { prefix } : undefined },
    });
    return data;
  },

  getConfig: async (key: string, date?: string): Promise<components["schemas"]["ResolvedConfig"] | null> => {
    const { data } = await client.GET<components["schemas"]["ResolvedConfig"]>("/api/v1/configs/{key}", {
      params: { path: { key }, query: date ? { date } : undefined },
    });
    return data ?? null;
  },

  upsertConfig: async (payload: CreateConfigEntryRequest): Promise<OrganizationConfigEntry> => {
    let value: ConfigValue;
    if (payload.value_bool !== undefined && payload.value_bool !== null) value = { Bool: payload.value_bool };
    else if (payload.value_decimal !== undefined && payload.value_decimal !== null) value = { Decimal: payload.value_decimal };
    else if (payload.value_int !== undefined && payload.value_int !== null) value = { Int: payload.value_int };
    else if (payload.value_json !== undefined && payload.value_json !== null) value = { Json: payload.value_json };
    else value = { Text: payload.value_text ?? "" };

    const { data } = await client.PUT<OrganizationConfigEntry>("/api/v1/configs", {
      body: {
        config_key: payload.config_key,
        effective_from: payload.effective_from,
        effective_to: payload.effective_to,
        scope_id: payload.scope_id,
        scope_type: payload.scope_type,
        value: value as components["schemas"]["ConfigValue"],
      },
    });
    return data;
  },

  deleteConfig: async (id: string): Promise<void> => {
    await client.DELETE("/api/v1/configs/{id}", { params: { path: { id } } });
  },
};
