import { describe, it, expect, beforeEach, vi } from "vitest";

import {
  fetchEContractConfigs,
  saveEContractConfig,
  fetchEContractRules,
  saveEContractRule,
  deleteEContractRule,
  fetchEContractEnvelopes,
  saveEContractEnvelope,
  EContractConfig,
  EContractRoutingRule,
  EContractEnvelope
} from "../econtracts";
import { Employer } from "../types";

const mockEmployers = [
  {
    id: "employer-1",
    name: "Cong Ty Co Phan Giai Phap LeeAttend",
    tax_code: "0101234567",
    address: "Ho Chi Minh City",
  }
] as unknown as Employer[];

describe("econtracts API mockup layer", () => {
  beforeEach(async () => {
    localStorage.clear();
    vi.clearAllMocks();
    // Initialize default mock data in localStorage
    await fetchEContractConfigs("ws-1", mockEmployers);
    await fetchEContractRules("ws-1", mockEmployers);
    await fetchEContractEnvelopes("ws-1", mockEmployers);
  });

  describe("Configs CRUD", () => {
    it("should initialize default config when localStorage is empty", async () => {
      const configs = await fetchEContractConfigs("ws-1", mockEmployers);
      expect(configs).toHaveLength(1);
      expect(configs[0].employer_id).toBe(mockEmployers[0].id);
      expect(configs[0].provider_type).toBe("smartca");
    });

    it("should save a new e-contract config successfully", async () => {
      const newConfig: EContractConfig = {
        id: "",
        employer_id: "employer-2",
        provider_type: "fpt",
        api_url: "https://api.fpt.vn/v1",
        client_id: "fpt_client_01",
        client_secret_encrypted: "client-secret-key",
        default_auth_type: "sms",
        is_active: true,
        created_at: "",
        updated_at: "",
      };

      const saved = await saveEContractConfig("ws-1", newConfig);
      expect(saved.employer_id).toBe("employer-2");
      expect(saved.provider_type).toBe("fpt");
      expect(saved.id).toBeDefined();
      expect(saved.id).not.toBe("");

      const configs = await fetchEContractConfigs("ws-1", mockEmployers);
      // Default config was initialized, plus our saved config
      expect(configs.find(c => c.employer_id === "employer-2")).toBeDefined();
    });

    it("should update an existing e-contract config", async () => {
      // First initialize default config
      const initialConfigs = await fetchEContractConfigs("ws-1", mockEmployers);
      const firstConfig = initialConfigs[0];

      const updatedConfig = {
        ...firstConfig,
        api_url: "https://new-api.vnpt.vn",
      };

      const saved = await saveEContractConfig("ws-1", updatedConfig);
      expect(saved.api_url).toBe("https://new-api.vnpt.vn");

      const configs = await fetchEContractConfigs("ws-1", mockEmployers);
      const vnptConfig = configs.find(c => c.id === firstConfig.id);
      expect(vnptConfig?.api_url).toBe("https://new-api.vnpt.vn");
    });
  });

  describe("Routing Rules CRUD", () => {
    it("should initialize default rules when empty", async () => {
      const rules = await fetchEContractRules("ws-1", mockEmployers);
      expect(rules).toHaveLength(2);
      expect(rules[0].signing_method).toBe("smartca");
      expect(rules[1].signing_method).toBe("sms");
    });

    it("should create a new routing rule", async () => {
      const newRule: Omit<EContractRoutingRule, "id" | "created_at"> = {
        employer_id: "employer-1",
        signer_country: "US",
        signing_method: "email",
        target_provider_type: "docusign",
        priority: 15,
      };

      const saved = await saveEContractRule("ws-1", newRule);
      expect(saved.signer_country).toBe("US");
      expect(saved.id).toBeDefined();

      const rules = await fetchEContractRules("ws-1", mockEmployers);
      expect(rules).toHaveLength(3); // 2 defaults + 1 new
      expect(rules.find(r => r.signer_country === "US")).toBeDefined();
    });

    it("should update an existing routing rule", async () => {
      const rules = await fetchEContractRules("ws-1", mockEmployers);
      const firstRule = rules[0];

      const updatedRule = {
        ...firstRule,
        priority: 99,
      };

      const saved = await saveEContractRule("ws-1", updatedRule);
      expect(saved.priority).toBe(99);

      const updatedRules = await fetchEContractRules("ws-1", mockEmployers);
      const found = updatedRules.find(r => r.id === firstRule.id);
      expect(found?.priority).toBe(99);
    });

    it("should delete a routing rule", async () => {
      const rules = await fetchEContractRules("ws-1", mockEmployers);
      const ruleIdToDelete = rules[0].id;

      await deleteEContractRule("ws-1", ruleIdToDelete);

      const updatedRules = await fetchEContractRules("ws-1", mockEmployers);
      expect(updatedRules).toHaveLength(1);
      expect(updatedRules.find(r => r.id === ruleIdToDelete)).toBeUndefined();
    });
  });

  describe("Envelopes CRUD", () => {
    it("should initialize default envelopes when empty", async () => {
      const envelopes = await fetchEContractEnvelopes("ws-1", mockEmployers);
      expect(envelopes).toHaveLength(1);
      expect(envelopes[0].external_envelope_id).toBe("VNPT-ENV-887192");
    });

    it("should create/save a new envelope", async () => {
      const newEnvelope: EContractEnvelope = {
        id: "env-2",
        contract_id: "contract-2",
        employer_id: "employer-1",
        provider_type: "fpt",
        external_envelope_id: "FPT-ENV-99999",
        status: "sent",
        signer_statuses: [
          {
            signer_id: "signer-1",
            name: "John Doe",
            email: "john@example.com",
            role: "employee",
            routing_order: 1,
            status: "pending",
          }
        ],
        created_at: new Date().toISOString(),
      };

      const saved = await saveEContractEnvelope("ws-1", newEnvelope);
      expect(saved.external_envelope_id).toBe("FPT-ENV-99999");

      const envelopes = await fetchEContractEnvelopes("ws-1", mockEmployers);
      expect(envelopes).toHaveLength(2); // 1 default + 1 new
    });

    it("should update an existing envelope", async () => {
      const envelopes = await fetchEContractEnvelopes("ws-1", mockEmployers);
      const envelopeToUpdate = envelopes[0];
      
      const updated = {
        ...envelopeToUpdate,
        status: "completed" as const,
      };

      const saved = await saveEContractEnvelope("ws-1", updated);
      expect(saved.status).toBe("completed");

      const reFetched = await fetchEContractEnvelopes("ws-1", mockEmployers);
      expect(reFetched[0].status).toBe("completed");
    });
  });

  describe("LocalStorage fallback branches", () => {
    it("covers fallback branches when localStorage is empty", async () => {
      localStorage.clear();
      
      // 1. saveEContractConfig when configs is null
      const config: EContractConfig = {
        id: "c-fallback",
        employer_id: "employer-1",
        provider_type: "smartca",
        api_url: "https://api.smartca.vn",
        client_id: "client",
        client_secret_encrypted: "secret",
        default_auth_type: "smartca",
        is_active: true,
        created_at: "",
        updated_at: "",
      };
      await saveEContractConfig("ws-1", config);

      // 2. saveEContractRule when rules is null
      const rule: Omit<EContractRoutingRule, "id" | "created_at"> = {
        employer_id: "employer-1",
        signer_country: "VN",
        signing_method: "smartca",
        target_provider_type: "smartca",
        priority: 10,
      };
      await saveEContractRule("ws-1", rule);

      // 3. deleteEContractRule when rules is null
      localStorage.clear();
      await deleteEContractRule("ws-1", "rule-nonexistent");

      // 4. saveEContractEnvelope when envelopes is null
      localStorage.clear();
      const envelope: EContractEnvelope = {
        id: "env-fallback",
        contract_id: "contract-1",
        employer_id: "employer-1",
        provider_type: "smartca",
        external_envelope_id: "VNPT-ENV-887192",
        status: "sent",
        signer_statuses: [],
        created_at: "2025-01-01T07:00:00Z",
      };
      await saveEContractEnvelope("ws-1", envelope);
    });
  });
});
