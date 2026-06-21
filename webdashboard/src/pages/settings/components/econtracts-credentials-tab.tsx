import { KeyRound } from "lucide-react";
import React, { useState } from "react";

import Button from "@/components/ui/button";
import { Combobox } from "@/components/ui/combobox";
import FormField from "@/components/ui/form-field";
import { Typography } from "@/components/ui/typography";
import { EContractConfig } from "@/lib/api/econtracts";
import { cn } from "@/lib/utils";

interface EContractsCredentialsTabProps {
  configs: EContractConfig[];
  isSaving: boolean;
  onSave: (config: EContractConfig) => void;
  selectedEmployerId: string;
}

export default function EContractsCredentialsTab({
  configs,
  isSaving,
  onSave,
  selectedEmployerId,
}: EContractsCredentialsTabProps) {
  const [configForm, setConfigForm] = useState<Partial<EContractConfig>>(() => {
    const activeConf = configs.find((c) => c.employer_id === selectedEmployerId);
    return activeConf ?? {
      provider_type: "smartca",
      api_url: "",
      client_id: "",
      client_secret_encrypted: "",
      webhook_secret_encrypted: "",
      hsm_key_alias: "",
      default_auth_type: "smartca",
      is_active: true,
    };
  });

  const handleSubmit = (e: React.SyntheticEvent) => {
    e.preventDefault();
    onSave({
      ...configForm,
      employer_id: selectedEmployerId,
    } as EContractConfig);
  };

  return (
    <CredentialsForm
      configForm={configForm}
      isSaving={isSaving}
      onChange={setConfigForm}
      onSubmit={handleSubmit}
    />
  );
}

interface CredentialsFormProps {
  configForm: Partial<EContractConfig>;
  isSaving: boolean;
  onChange: React.Dispatch<React.SetStateAction<Partial<EContractConfig>>>;
  onSubmit: (e: React.SyntheticEvent) => void;
}

function CredentialsForm({
  configForm,
  isSaving,
  onChange,
  onSubmit,
}: CredentialsFormProps) {
  return (
    <form
      className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 p-8 rounded-[2.5rem] shadow-sm space-y-6"
      noValidate
      onSubmit={onSubmit}
    >
      <div className="flex justify-between items-center border-b border-slate-50 dark:border-slate-800 pb-4">
        <div>
          <Typography
            className="text-sm font-bold uppercase tracking-widest text-slate-800 dark:text-slate-100 flex items-center gap-2"
            variant="h3"
          >
            <KeyRound className="text-primary-500" size={16} /> API Integration Credentials
          </Typography>
          <Typography
            className="text-[10px] text-slate-400 font-semibold uppercase tracking-wider mt-0.5"
            variant="p"
          >
            Configure keys, endpoints, and credentials for CA providers
          </Typography>
        </div>
        <div className="flex items-center gap-2.5">
          <Typography
            className="text-xs font-bold text-slate-400 uppercase tracking-widest"
            variant="small"
          >
            Active Configuration
          </Typography>
          <div
            aria-checked={configForm.is_active ?? true}
            className={cn(
              "relative inline-flex h-6 w-11 shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-primary-500/20 focus:ring-offset-2",
              configForm.is_active ? "bg-primary-600" : "bg-slate-200 dark:bg-slate-800"
            )}
            onClick={() => {
              onChange((prev) => ({ ...prev, is_active: !prev.is_active }));
            }}
            onKeyDown={(e) => {
              if (e.key === " " || e.key === "Enter") {
                e.preventDefault();
                onChange((prev) => ({ ...prev, is_active: !prev.is_active }));
              }
            }}
            role="switch"
            tabIndex={0}
          >
            <span
              className={cn(
                "pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out",
                configForm.is_active ? "translate-x-5" : "translate-x-0"
              )}
            />
          </div>
        </div>
      </div>

      <ProviderSection configForm={configForm} onChange={onChange} />
      <EndpointFields configForm={configForm} onChange={onChange} />
      <SecretFields configForm={configForm} onChange={onChange} />

      <FormField
        id="hsm-alias-input"
        label="HSM Key Alias (Optional, required if using HSM signatures)"
        onChange={(e) => {
          onChange((prev) => ({ ...prev, hsm_key_alias: e.target.value }));
        }}
        placeholder="VD: alias_hsm_primary_01"
        value={configForm.hsm_key_alias ?? ""}
      />

      <div className="flex justify-end pt-4 border-t border-slate-50 dark:border-slate-800">
        <Button loading={isSaving} type="submit" variant="secondary">
          SAVE CONFIGURATIONS
        </Button>
      </div>
    </form>
  );
}

interface FormSectionProps {
  configForm: Partial<EContractConfig>;
  onChange: React.Dispatch<React.SetStateAction<Partial<EContractConfig>>>;
}

function ProviderSection({ configForm, onChange }: FormSectionProps) {
  return (
    <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
      <div className="space-y-2">
        <label
          className="text-[11px] font-bold text-slate-500 uppercase tracking-widest block cursor-pointer"
          htmlFor="provider-type-select"
        >
          Provider Type
        </label>
        <Combobox
          id="provider-type-select"
          onChange={(val) => {
            onChange((prev) => ({ ...prev, provider_type: val }));
          }}
          options={[
            { value: "smartca", label: "VN SmartCA / Remote Sign (VNPT)" },
            { value: "hsm", label: "Corporate HSM Signature" },
            { value: "fpt", label: "FPT CeCA Compliance Engine" },
            { value: "docusign", label: "DocuSign International (eIDAS)" },
            { value: "mock", label: "Sandbox Staging Mock Provider" },
          ]}
          placeholder="Select Provider Type"
          value={configForm.provider_type ?? "smartca"}
        />
      </div>

      <div className="space-y-2">
        <label
          className="text-[11px] font-bold text-slate-500 uppercase tracking-widest block cursor-pointer"
          htmlFor="default-auth-select"
        >
          Default Authorization Verification
        </label>
        <Combobox
          id="default-auth-select"
          onChange={(val) => {
            onChange((prev) => ({
              ...prev,
              default_auth_type: val as EContractConfig["default_auth_type"],
            }));
          }}
          options={[
            { value: "smartca", label: "SmartCA Verification" },
            { value: "sms", label: "SMS OTP Code" },
            { value: "email", label: "Email OTP Code" },
            { value: "hsm", label: "HSM Enterprise Sign" },
          ]}
          placeholder="Select Authorization Type"
          value={configForm.default_auth_type ?? "smartca"}
        />
      </div>
    </div>
  );
}

function EndpointFields({ configForm, onChange }: FormSectionProps) {
  return (
    <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
      <FormField
        id="api-url-input"
        label="API Gateway Endpoint URL"
        onChange={(e) => {
          onChange((prev) => ({ ...prev, api_url: e.target.value }));
        }}
        placeholder="VD: https://api.vnpt-smartca.vn/v1"
        required
        value={configForm.api_url ?? ""}
      />
      <FormField
        id="client-id-input"
        label="Integration Client ID / Access ID"
        onChange={(e) => {
          onChange((prev) => ({ ...prev, client_id: e.target.value }));
        }}
        placeholder="VD: client_prod_88921"
        required
        value={configForm.client_id ?? ""}
      />
    </div>
  );
}

function SecretFields({ configForm, onChange }: FormSectionProps) {
  return (
    <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
      <FormField
        id="client-secret-input"
        label="Client Credentials Secret (Encrypted at Rest)"
        onChange={(e) => {
          onChange((prev) => ({ ...prev, client_secret_encrypted: e.target.value }));
        }}
        placeholder="••••••••••••••••••••••••••••"
        required
        type="password"
        value={configForm.client_secret_encrypted ?? ""}
      />
      <FormField
        id="webhook-secret-input"
        label="Webhook Secret (Verify incoming callback events)"
        onChange={(e) => {
          onChange((prev) => ({ ...prev, webhook_secret_encrypted: e.target.value }));
        }}
        placeholder="VD: whsec_abc123"
        value={configForm.webhook_secret_encrypted ?? ""}
      />
    </div>
  );
}
