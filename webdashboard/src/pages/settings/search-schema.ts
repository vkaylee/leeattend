import { z } from "zod";

/**
 * URL search-param contracts for settings sub-pages under `/settings/*`.
 * Each schema is owned by its specific route file — see ADR-006.
 */

export const payrollSettingsSearchSchema = z
  .object({
    tab: z
      .enum(["policy", "components", "bases", "incentives", "overtime", "statutory_rules"])
      .catch("policy"),
  })
  .catch({ tab: "policy" });

export type PayrollSettingsSearch = z.infer<typeof payrollSettingsSearchSchema>;

export const leaveSettingsSearchSchema = z
  .object({
    tab: z.enum(["types", "policies", "assignments"]).catch("types"),
  })
  .catch({ tab: "types" });

export type LeaveSettingsSearch = z.infer<typeof leaveSettingsSearchSchema>;

export const lifecycleTemplatesSearchSchema = z
  .object({
    tab: z.enum(["onboarding", "offboarding", "transfer", "probation_end"]).catch("onboarding"),
  })
  .catch({ tab: "onboarding" });

export type LifecycleTemplatesSearch = z.infer<typeof lifecycleTemplatesSearchSchema>;
