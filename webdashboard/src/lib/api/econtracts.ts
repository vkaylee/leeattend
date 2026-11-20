// e-Contracts e-signature integration — stub barrel.
// Types are re-exported from econtracts-types.ts.
// Backend HTTP endpoints are not yet wired (apiserver/src/services/econtract/ has
// provider trait + dispatcher but no handlers). All API functions return empty
// results until the backend is connected.
//
// When wiring: replace stubs with real client.GET/POST calls targeting the
// registered routes.
// @see TD-1

import { toast } from "@/components/ui/toast-imperative";

import type { Employer } from "./types";

import {
  EContractConfig,
  EContractRoutingRule,
  EContractEnvelope,
  SignerStatus,
} from "./econtracts-types";

export type { EContractConfig, EContractRoutingRule, EContractEnvelope, SignerStatus };

// ── Stub API functions ──────────────────────────────────────────────────

export function fetchEContractConfigs(
  _employers: Employer[],
): Promise<EContractConfig[]> {
  return Promise.resolve([]);
}

export function saveEContractConfig(
  _config: EContractConfig,
): Promise<EContractConfig> {
  return Promise.resolve(_config);
}

export function fetchEContractRules(
  _employers: Employer[],
): Promise<EContractRoutingRule[]> {
  return Promise.resolve([]);
}

export function saveEContractRule(
  _rule: Omit<EContractRoutingRule, "id" | "created_at"> & { id?: string },
): Promise<EContractRoutingRule> {
  toast.error("eContracts backend not wired — cannot save routing rule.");
  // Return a graceful stub so callers don't crash.
  return Promise.resolve({
    ..._rule,
    id: _rule.id ?? `stub_rule_${Date.now()}`,
    created_at: new Date().toISOString(),
  });
}

export function deleteEContractRule(_ruleId: string): Promise<void> {
  return Promise.resolve();
}

export function fetchEContractEnvelopes(
  _employers: Employer[],
): Promise<EContractEnvelope[]> {
  return Promise.resolve([]);
}

export function saveEContractEnvelope(
  envelope: EContractEnvelope,
): Promise<EContractEnvelope> {
  return Promise.resolve(envelope);
}
