// e-Contracts e-signature integration — stub barrel.
// Types are re-exported from econtracts-types.ts.
// Backend HTTP endpoints are not yet wired (apiserver/src/services/econtract/ has
// provider trait + dispatcher but no handlers). All API functions return empty
// results until the backend is connected.
//
// When wiring: replace stubs with real client.GET/POST calls targeting the
// registered routes, and remove this comment block.
// @see TD-1
/* eslint-disable @typescript-eslint/require-await -- stubs; will await real API calls when backend is wired */

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

const UNWIRED = () => {
  if (import.meta.env.DEV) {
    // eslint-disable-next-line no-console
    console.info("[econtracts] Backend not wired — returning empty result.");
  }
};

export async function fetchEContractConfigs(
  _wsId: string,
  _employers: Employer[],
): Promise<EContractConfig[]> {
  UNWIRED();
  return [];
}

export async function saveEContractConfig(
  _wsId: string,
  _config: EContractConfig,
): Promise<EContractConfig> {
  UNWIRED();
  return _config;
}

export async function fetchEContractRules(
  _wsId: string,
  _employers: Employer[],
): Promise<EContractRoutingRule[]> {
  UNWIRED();
  return [];
}

export async function saveEContractRule(
  _wsId: string,
  _rule: Omit<EContractRoutingRule, "id" | "created_at"> & { id?: string },
): Promise<EContractRoutingRule> {
  UNWIRED();
  toast.error("eContracts backend not wired — cannot save routing rule.");
  // Return a graceful stub so callers don't crash.
  return {
    ..._rule,
    id: _rule.id ?? `stub_rule_${Date.now()}`,
    created_at: new Date().toISOString(),
  };
}

export async function deleteEContractRule(_wsId: string, _ruleId: string): Promise<void> {
  UNWIRED();
}

export async function fetchEContractEnvelopes(
  _wsId: string,
  _employers: Employer[],
): Promise<EContractEnvelope[]> {
  UNWIRED();
  return [];
}

export async function saveEContractEnvelope(
  _wsId: string,
  envelope: EContractEnvelope,
): Promise<EContractEnvelope> {
  UNWIRED();
  return envelope;
}
