-- Create employer_econtract_configs table
CREATE TABLE employer_econtract_configs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    employer_id UUID NOT NULL REFERENCES employers(id) ON DELETE CASCADE,
    workspace_id UUID NOT NULL,
    provider_type VARCHAR(50) NOT NULL,
    api_url VARCHAR(255) NOT NULL,
    client_id VARCHAR(255) NOT NULL,
    client_secret_encrypted TEXT NOT NULL,
    webhook_secret_encrypted TEXT,
    hsm_key_alias VARCHAR(100),
    default_auth_type VARCHAR(50) DEFAULT 'sms' NOT NULL,
    is_active BOOLEAN DEFAULT TRUE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT uq_employer_provider UNIQUE (employer_id, provider_type)
);

CREATE INDEX idx_employer_econtract_configs_lookup ON employer_econtract_configs(employer_id, workspace_id);

-- Create employer_econtract_routing_rules table
CREATE TABLE employer_econtract_routing_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    employer_id UUID NOT NULL REFERENCES employers(id) ON DELETE CASCADE,
    workspace_id UUID NOT NULL,
    signer_country VARCHAR(10) NOT NULL,
    signing_method VARCHAR(50) NOT NULL,
    target_provider_type VARCHAR(50) NOT NULL,
    priority INT DEFAULT 1 NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE INDEX idx_econtract_routing_rules ON employer_econtract_routing_rules(employer_id, signer_country);

-- Create econtract_envelopes table
CREATE TABLE econtract_envelopes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    contract_id UUID NOT NULL REFERENCES employee_contracts(id) ON DELETE CASCADE,
    employer_id UUID NOT NULL REFERENCES employers(id) ON DELETE RESTRICT,
    provider_type VARCHAR(50) NOT NULL,
    external_envelope_id VARCHAR(255) NOT NULL,
    status VARCHAR(50) DEFAULT 'draft' NOT NULL,
    signer_statuses JSONB DEFAULT '[]'::jsonb NOT NULL,
    audit_trail_metadata JSONB,
    signed_document_id UUID,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT uq_envelope_provider UNIQUE (provider_type, external_envelope_id)
);

CREATE INDEX idx_econtract_envelopes_external_id ON econtract_envelopes(external_envelope_id);
CREATE INDEX idx_econtract_envelopes_contract ON econtract_envelopes(contract_id);
