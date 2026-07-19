// local Mock API layer for e-Contracts Hub Option A & Option B
import { Employer } from "./types";

export interface EContractConfig {
  api_url: string;
  client_id: string;
  client_secret_encrypted: string;
  created_at: string;
  default_auth_type: 'sms' | 'email' | 'smartca' | 'hsm';
  employer_id: string;
  hsm_key_alias?: string;
  id: string;
  is_active: boolean;
  provider_type: string;
  updated_at: string;
  webhook_secret_encrypted?: string;
}

export interface EContractRoutingRule {
  created_at: string;
  employer_id: string;
  id: string;
  priority: number;
  signer_country: string;
  signing_method: 'sms' | 'email' | 'smartca' | 'hsm';
  target_provider_type: string;
}

export interface SignerStatus {
  email: string;
  name: string;
  phone?: string;
  role: string;
  routing_order: number;
  signed_at?: string;
  signer_id: string;
  status: 'pending' | 'signed' | 'voided';
}

export interface EContractEnvelope {
  audit_trail_metadata?: unknown;
  contract_id: string;
  created_at: string;
  employer_id: string;
  external_envelope_id: string;
  id: string;
  provider_type: string;
  signer_statuses: SignerStatus[];
  status: 'draft' | 'sent' | 'completed' | 'voided';
}

// Helpers
const getStorageKey = (key: string, wsId: string) => `leeattend:econtract:${key}:${wsId}`;

const delay = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

// Mock Data Initializers
const initMockData = (wsId: string, employers: Employer[]) => {
  const configsKey = getStorageKey("configs", wsId);
  const rulesKey = getStorageKey("rules", wsId);
  const envelopesKey = getStorageKey("envelopes", wsId);

  if (!localStorage.getItem(configsKey) && employers.length > 0) {
    const defaultConfigs: EContractConfig[] = [
      {
        id: "c-1",
        employer_id: employers[0].id,
        provider_type: "smartca",
        api_url: "https://api.vnpt-smartca.vn/v1",
        client_id: "vnpt_client_prod_01",
        client_secret_encrypted: "••••••••••••••••••••••••••••",
        webhook_secret_encrypted: "webhook_sec_vnpt_9981",
        hsm_key_alias: "hsm_token_alias_01",
        default_auth_type: "smartca",
        is_active: true,
        created_at: new Date(Date.now() - 30 * 24 * 60 * 60 * 1000).toISOString(),
        updated_at: new Date(Date.now() - 30 * 24 * 60 * 60 * 1000).toISOString(),
      },
    ];
    localStorage.setItem(configsKey, JSON.stringify(defaultConfigs));
  }

  if (!localStorage.getItem(rulesKey) && employers.length > 0) {
    const defaultRules: EContractRoutingRule[] = [
      {
        id: "r-1",
        employer_id: employers[0].id,
        signer_country: "VN",
        signing_method: "smartca",
        target_provider_type: "smartca",
        priority: 10,
        created_at: new Date(Date.now() - 29 * 24 * 60 * 60 * 1000).toISOString(),
      },
      {
        id: "r-2",
        employer_id: employers[0].id,
        signer_country: "VN",
        signing_method: "sms",
        target_provider_type: "mock",
        priority: 5,
        created_at: new Date(Date.now() - 28 * 24 * 60 * 60 * 1000).toISOString(),
      },
    ];
    localStorage.setItem(rulesKey, JSON.stringify(defaultRules));
  }

  if (!localStorage.getItem(envelopesKey) && employers.length > 0) {
    const defaultEnvelopes: EContractEnvelope[] = [
      {
        id: "env-mock-1",
        contract_id: "contract-1",
        employer_id: employers[0].id,
        provider_type: "smartca",
        external_envelope_id: "VNPT-ENV-887192",
        status: "completed",
        signer_statuses: [
          {
            signer_id: "hr-1",
            name: "Lê Tuyết Mai (HR)",
            email: "mai.lt@domain.com",
            role: "representative",
            routing_order: 1,
            status: "signed",
            signed_at: new Date(Date.now() - 2 * 24 * 60 * 60 * 1000).toISOString(),
          },
          {
            signer_id: "emp-1",
            name: "Nguyễn Văn A (Employee)",
            email: "a.nv@domain.com",
            role: "employee",
            routing_order: 2,
            status: "signed",
            signed_at: new Date(Date.now() - 2 * 24 * 60 * 60 * 1000).toISOString(),
          },
        ],
        audit_trail_metadata: {
          webhook_events: [
            { event: "envelope.created", timestamp: new Date(Date.now() - 2 * 24 * 60 * 60 * 1000).toISOString() },
            { event: "signer.representative.signed", timestamp: new Date(Date.now() - 2 * 24 * 60 * 60 * 1000).toISOString() },
            { event: "signer.employee.signed", timestamp: new Date(Date.now() - 2 * 24 * 60 * 60 * 1000).toISOString() },
            { event: "envelope.completed", timestamp: new Date(Date.now() - 2 * 24 * 60 * 60 * 1000).toISOString() },
          ],
        },
        created_at: new Date(Date.now() - 2 * 24 * 60 * 60 * 1000).toISOString(),
      },
    ];
    localStorage.setItem(envelopesKey, JSON.stringify(defaultEnvelopes));
  }
};

export async function fetchEContractConfigs(wsId: string, employers: Employer[]): Promise<EContractConfig[]> {
  await delay(300);
  initMockData(wsId, employers);
  const data = localStorage.getItem(getStorageKey("configs", wsId));
  return data ? (JSON.parse(data) as EContractConfig[]) : [];
}

export async function saveEContractConfig(wsId: string, config: EContractConfig): Promise<EContractConfig> {
  await delay(400);
  const key = getStorageKey("configs", wsId);
  // eslint-disable-next-line @typescript-eslint/prefer-nullish-coalescing
  const existing: EContractConfig[] = JSON.parse(localStorage.getItem(key) || "[]") as EContractConfig[];
  const idx = existing.findIndex((c) => c.employer_id === config.employer_id && c.provider_type === config.provider_type);
  
  const updatedConfig = {
    ...config,
    updated_at: new Date().toISOString(),
  };

  if (idx > -1) {
    existing[idx] = updatedConfig;
  } else {
    updatedConfig.id = updatedConfig.id || `c-${Math.random().toString(36).substring(2, 11)}`;
    updatedConfig.created_at = new Date().toISOString();
    existing.push(updatedConfig);
  }

  localStorage.setItem(key, JSON.stringify(existing));
  return updatedConfig;
}

export async function fetchEContractRules(wsId: string, employers: Employer[]): Promise<EContractRoutingRule[]> {
  await delay(300);
  initMockData(wsId, employers);
  const data = localStorage.getItem(getStorageKey("rules", wsId));
  return data ? (JSON.parse(data) as EContractRoutingRule[]) : [];
}

export async function saveEContractRule(wsId: string, rule: Omit<EContractRoutingRule, "id" | "created_at"> & { id?: string }): Promise<EContractRoutingRule> {
  await delay(300);
  const key = getStorageKey("rules", wsId);
  // eslint-disable-next-line @typescript-eslint/prefer-nullish-coalescing
  const existing: EContractRoutingRule[] = JSON.parse(localStorage.getItem(key) || "[]") as EContractRoutingRule[];
  
  if (rule.id) {
    const idx = existing.findIndex((r) => r.id === rule.id);
    if (idx > -1) {
      existing[idx] = {
        ...existing[idx],
        ...rule,
      };
      localStorage.setItem(key, JSON.stringify(existing));
      return existing[idx];
    }
  }

  const newRule: EContractRoutingRule = {
    ...rule,
  // eslint-disable-next-line @typescript-eslint/no-deprecated
    id: `r-${Math.random().toString(36).substr(2, 9)}`,
    created_at: new Date().toISOString(),
  };

  existing.push(newRule);
  localStorage.setItem(key, JSON.stringify(existing));
  return newRule;
}

export async function deleteEContractRule(wsId: string, ruleId: string): Promise<void> {
  await delay(200);
  const key = getStorageKey("rules", wsId);
  // eslint-disable-next-line @typescript-eslint/prefer-nullish-coalescing
  const existing: EContractRoutingRule[] = JSON.parse(localStorage.getItem(key) || "[]") as EContractRoutingRule[];
  const filtered = existing.filter((r) => r.id !== ruleId);
  localStorage.setItem(key, JSON.stringify(filtered));
}

export async function fetchEContractEnvelopes(wsId: string, employers: Employer[]): Promise<EContractEnvelope[]> {
  await delay(400);
  // eslint-disable-next-line max-lines
  initMockData(wsId, employers);
  const data = localStorage.getItem(getStorageKey("envelopes", wsId));
  return data ? (JSON.parse(data) as EContractEnvelope[]) : [];
}

export async function saveEContractEnvelope(wsId: string, envelope: EContractEnvelope): Promise<EContractEnvelope> {
  await delay(300);
  const key = getStorageKey("envelopes", wsId);
  // eslint-disable-next-line @typescript-eslint/prefer-nullish-coalescing
  const existing: EContractEnvelope[] = JSON.parse(localStorage.getItem(key) || "[]") as EContractEnvelope[];
  const idx = existing.findIndex((e) => e.id === envelope.id);
  if (idx > -1) {
    existing[idx] = envelope;
  } else {
    existing.push(envelope);
  }
  localStorage.setItem(key, JSON.stringify(existing));
  return envelope;
}
