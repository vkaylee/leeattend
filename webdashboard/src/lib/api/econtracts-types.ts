// e-Contracts e-signature integration types.
// e-signature backend is not yet wired to HTTP routes
// (apiserver/src/services/econtract/ provider trait + mock impl exist but no handlers).
// These types are consumed by the settings UI and employee detail drawer
// which will activate once the backend endpoints are registered.
// See: https://linear/leeattend/econtract-backend-wiring
// @see TD-1

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
