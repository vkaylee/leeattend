import type { components } from "@/types/api-generated";

import { Nullable } from "@/types/api-overrides";

import { client } from "./client";

export type ConditionEarlyArrivalStrategy = "Allow" | "Block" | "Clamp";

/** Mirrors the validated JSONB contract accepted by the condition type API. */
export interface ConditionRuleEffects {
  allow_early_ot?: boolean;
  authorized_early_leave_minutes?: boolean;
  early_arrival_strategy?: ConditionEarlyArrivalStrategy;
  grace_period_minutes?: boolean;
  is_break_auto_deduct?: boolean;
  is_break_overtime?: boolean;
  is_early_leave_paid?: boolean;
}

// Re-export generated types with JSONB field overrides (OpenAPI doesn't type JSONB strongly)
export type ConditionTypeDefinition = Omit<
  components["schemas"]["ConditionTypeDefinition"],
  "name_i18n" | "affects_rules"
> & {
  name_i18n: Record<string, string>;
  affects_rules: ConditionRuleEffects;
};

export type CreateConditionTypeRequest = Omit<
  components["schemas"]["CreateConditionTypeRequest"],
  "name_i18n" | "affects_rules"
> & {
  name_i18n: Record<string, string>;
  affects_rules: ConditionRuleEffects;
};

export type UpdateConditionTypeRequest = Omit<
  components["schemas"]["UpdateConditionTypeRequest"],
  "name_i18n" | "affects_rules" | "code"
> & {
  name_i18n?: Record<string, string>;
  affects_rules?: ConditionRuleEffects;
};

export async function fetchConditionTypes(
  activeOnly?: boolean,
): Promise<ConditionTypeDefinition[]> {
  const { data } = await client.GET<ConditionTypeDefinition[]>("/v1/condition-types", {
    params: {
      query: activeOnly !== undefined ? { active_only: activeOnly } : undefined,
    },
  });
  return (data as Nullable<ConditionTypeDefinition[]>) ?? [];
}

export async function fetchConditionType(
  id: string,
): Promise<ConditionTypeDefinition> {
  const { data } = await client.GET<ConditionTypeDefinition>("/v1/condition-types/{id}", {
    params: {
      path: { id },
    },
  });
  return data as ConditionTypeDefinition;
}

export async function createConditionType(
  req: CreateConditionTypeRequest,
): Promise<ConditionTypeDefinition> {
  const { data } = await client.POST<ConditionTypeDefinition>("/v1/condition-types", {
    body: req,
  });
  return data as ConditionTypeDefinition;
}

export async function updateConditionType(
  id: string,
  req: UpdateConditionTypeRequest,
): Promise<ConditionTypeDefinition> {
  const { data } = await client.PATCH<ConditionTypeDefinition>("/v1/condition-types/{id}", {
    params: {
      path: { id },
    },
    body: req,
  });
  return data as ConditionTypeDefinition;
}

export async function deleteConditionType(id: string): Promise<void> {
  await client.DELETE("/v1/condition-types/{id}", {
    params: {
      path: { id },
    },
  });
}
