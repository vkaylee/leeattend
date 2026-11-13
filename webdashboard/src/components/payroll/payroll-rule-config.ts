/** Narrow value to Record<string, unknown> shape. */
export function isObject(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

export interface PercentageConfig { base?: string; cap_amount?: number; rate: number }
export interface FixedConfig { amount: number }

/** Narrow the unknown rule_config to PercentageConfig, defaulting missing fields. */
export function asPercentageConfig(raw: unknown): PercentageConfig {
  const c = isObject(raw) ? raw : {};
  return {
    rate: typeof c.rate === "number" ? c.rate : 0,
    cap_amount: typeof c.cap_amount === "number" ? c.cap_amount : undefined,
  };
}

/** Narrow the unknown rule_config to FixedConfig, defaulting missing fields. */
export function asFixedConfig(raw: unknown): FixedConfig {
  const c = isObject(raw) ? raw : {};
  return { amount: typeof c.amount === "number" ? c.amount : 0 };
}
