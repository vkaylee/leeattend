import { Scale, Settings, Layers, FileText, Plus, Trash2, Search, Eye, KeyRound, Globe, AlertCircle } from "lucide-react";
import React, { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";

import PageHeader from "@/components/ui/page-header";
import Button from "@/components/ui/button";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";
import FormField from "@/components/ui/form-field";
import Modal from "@/components/ui/modal";
import EmptyState from "@/components/ui/empty-state";
import { fetchEmployers } from "@/lib/api/employers";
import { getCurrentWorkspaceId } from "@/lib/api/helpers";
import { useToast } from "@/components/ui/toast";
import { cn } from "@/lib/utils";

import {
  fetchEContractConfigs,
  saveEContractConfig,
  fetchEContractRules,
  saveEContractRule,
  deleteEContractRule,
  fetchEContractEnvelopes,
  EContractConfig,
  EContractRoutingRule,
  EContractEnvelope,
} from "@/lib/api/econtracts";

export default function EContractsSettingsPage() {
  const { t } = useTranslation();
  const toast = useToast();
  const queryClient = useQueryClient();
  const wsId = getCurrentWorkspaceId() || "";

  const [activeTab, setActiveTab] = useState<string>("configs");
  const [selectedEmployerId, setSelectedEmployerId] = useState<string>("");

  // Queries
  const { data: employers = [], isLoading: isLoadingEmployers } = useQuery({
    queryKey: ["employers", wsId],
    queryFn: () => fetchEmployers(wsId),
    enabled: !!wsId,
  });

  const { data: configs = [], isLoading: isLoadingConfigs } = useQuery({
    queryKey: ["econtract-configs", wsId],
    queryFn: () => fetchEContractConfigs(wsId, employers),
    enabled: !!wsId && employers.length > 0,
  });

  const { data: rules = [], isLoading: isLoadingRules } = useQuery({
    queryKey: ["econtract-rules", wsId],
    queryFn: () => fetchEContractRules(wsId, employers),
    enabled: !!wsId && employers.length > 0,
  });

  const { data: envelopes = [], isLoading: isLoadingEnvelopes } = useQuery({
    queryKey: ["econtract-envelopes", wsId],
    queryFn: () => fetchEContractEnvelopes(wsId, employers),
    enabled: !!wsId && employers.length > 0,
  });

  // Auto-select first employer
  useEffect(() => {
    if (employers.length > 0 && !selectedEmployerId) {
      setSelectedEmployerId(employers[0].id);
    }
  }, [employers, selectedEmployerId]);

  // Mutations
  const configMutation = useMutation({
    mutationFn: (config: EContractConfig) => saveEContractConfig(wsId, config),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["econtract-configs", wsId] });
      toast.success(t("common.saved_success", { defaultValue: "Saved configuration successfully!" }));
    },
    onError: (err: any) => {
      toast.error(err.message || "Failed to save configuration");
    },
  });

  const ruleMutation = useMutation({
    mutationFn: (rule: Omit<EContractRoutingRule, "id" | "created_at">) => saveEContractRule(wsId, rule),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["econtract-rules", wsId] });
      toast.success("Routing rule added successfully!");
    },
  });

  const ruleDeleteMutation = useMutation({
    mutationFn: (ruleId: string) => deleteEContractRule(wsId, ruleId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["econtract-rules", wsId] });
      toast.success("Routing rule deleted successfully!");
    },
  });

  // Credential configuration state fields
  const [configForm, setConfigForm] = useState<Partial<EContractConfig>>({
    provider_type: "smartca",
    api_url: "",
    client_id: "",
    client_secret_encrypted: "",
    webhook_secret_encrypted: "",
    hsm_key_alias: "",
    default_auth_type: "smartca",
    is_active: true,
  });

  // Update form fields when active config changes for the selected employer
  useEffect(() => {
    if (selectedEmployerId && configs.length > 0) {
      const activeConf = configs.find((c: EContractConfig) => c.employer_id === selectedEmployerId);
      if (activeConf) {
        setConfigForm(activeConf);
      } else {
        setConfigForm({
          provider_type: "smartca",
          api_url: "",
          client_id: "",
          client_secret_encrypted: "",
          webhook_secret_encrypted: "",
          hsm_key_alias: "",
          default_auth_type: "smartca",
          is_active: true,
        });
      }
    }
  }, [selectedEmployerId, configs]);

  // Form handle for Config save
  const handleSaveConfig = (e: React.FormEvent) => {
    e.preventDefault();
    if (!selectedEmployerId) {
      toast.error("Please select a Legal Entity.");
      return;
    }
    configMutation.mutate({
      ...configForm,
      employer_id: selectedEmployerId,
    } as EContractConfig);
  };

  // Rule Creation Modal state
  const [isRuleModalOpen, setIsRuleModalOpen] = useState(false);
  const [newRuleForm, setNewRuleForm] = useState<Partial<EContractRoutingRule>>({
    signer_country: "VN",
    signing_method: "smartca",
    target_provider_type: "smartca",
    priority: 1,
  });

  const handleCreateRule = (e: React.FormEvent) => {
    e.preventDefault();
    if (!selectedEmployerId) return;
    ruleMutation.mutate({
      ...newRuleForm,
      employer_id: selectedEmployerId,
    } as Omit<EContractRoutingRule, "id" | "created_at">);
    setIsRuleModalOpen(false);
  };

  // Webhook log Modal state
  const [selectedEnvelopeForLog, setSelectedEnvelopeForLog] = useState<EContractEnvelope | null>(null);

  // Filters for Envelope dashboard
  const [envFilterSearch, setEnvFilterSearch] = useState("");
  const [envFilterProvider, setEnvFilterProvider] = useState("all");
  const [envFilterStatus, setEnvFilterStatus] = useState("all");

  const filteredEnvelopes = envelopes.filter((env: EContractEnvelope) => {
    const matchesSearch = env.external_envelope_id.toLowerCase().includes(envFilterSearch.toLowerCase()) || 
      env.id.toLowerCase().includes(envFilterSearch.toLowerCase());
    const matchesProvider = envFilterProvider === "all" || env.provider_type === envFilterProvider;
    const matchesStatus = envFilterStatus === "all" || env.status === envFilterStatus;
    return matchesSearch && matchesProvider && matchesStatus;
  });

  const employerRules = rules.filter((r: EContractRoutingRule) => r.employer_id === selectedEmployerId);

  const activeEmployerName = employers.find((e: any) => e.id === selectedEmployerId)?.name || "Select Legal Entity";

  if (!wsId) {
    return (
      <div className="p-12 text-center text-slate-500 dark:text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50">
        {t("common.select_workspace")}
      </div>
    );
  }

  return (
    <div className="space-y-6 animate-in fade-in duration-700 pb-20">
      <PageHeader
        icon={<Scale size={24} />}
        subtitle="Manage legal entity e-signing credentials, routing precedence rules, and tracking transactions"
        title="e-Contracts Integration"
      />

      {/* Global Legal Entity Picker */}
      <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 p-6 rounded-[2rem] shadow-sm flex flex-col md:flex-row items-start md:items-center justify-between gap-4">
        <div>
          <h2 className="text-xs font-bold text-slate-400 uppercase tracking-widest mb-1">
            Active Legal Entity Context
          </h2>
          <p className="text-sm font-bold text-slate-700 dark:text-slate-200">
            {activeEmployerName.toUpperCase()}
          </p>
        </div>
        <div className="w-full md:w-80">
          <label className="sr-only" htmlFor="employer-select">Select Legal Entity</label>
          <select
            className="w-full h-11 px-3 bg-slate-50 border border-slate-200 rounded-xl focus:ring-2 focus:ring-primary-500/20 focus:border-primary-500 dark:bg-slate-850 dark:border-slate-800 text-xs font-bold uppercase tracking-wider text-slate-600 dark:text-slate-300"
            id="employer-select"
            onChange={(e) => { setSelectedEmployerId(e.target.value); }}
            value={selectedEmployerId}
          >
            {isLoadingEmployers ? (
              <option>Loading Employers...</option>
            ) : employers.length === 0 ? (
              <option>No employers found</option>
            ) : (
              employers.map((emp) => (
                <option key={emp.id} value={emp.id}>
                  {emp.name.toUpperCase()}
                </option>
              ))
            )}
          </select>
        </div>
      </div>

      {/* Navigation tabs */}
      <Tabs onValueChange={setActiveTab} value={activeTab}>
        <TabsList>
          <TabsTrigger className="flex items-center gap-2 uppercase tracking-widest text-[11px]" value="configs">
            <Settings size={14} /> Credentials Manager
          </TabsTrigger>
          <TabsTrigger className="flex items-center gap-2 uppercase tracking-widest text-[11px]" value="rules">
            <Layers size={14} /> Routing Priority Rules
          </TabsTrigger>
          <TabsTrigger className="flex items-center gap-2 uppercase tracking-widest text-[11px]" value="envelopes">
            <FileText size={14} /> Envelope Audit Dashboard
          </TabsTrigger>
        </TabsList>

        {/* Tab 1 Content: Credentials Manager */}
        <TabsContent value="configs">
          <form className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 p-8 rounded-[2.5rem] shadow-sm space-y-6" noValidate onSubmit={handleSaveConfig}>
            <div className="flex justify-between items-center border-b border-slate-50 dark:border-slate-800 pb-4">
              <div>
                <h3 className="text-sm font-bold uppercase tracking-widest text-slate-800 dark:text-slate-100 flex items-center gap-2">
                  <KeyRound className="text-primary-500" size={16} /> API Integration Credentials
                </h3>
                <p className="text-[10px] text-slate-400 font-semibold uppercase tracking-wider mt-0.5">
                  Configure keys, endpoints, and credentials for CA providers
                </p>
              </div>
              <div className="flex items-center gap-2.5">
                <span className="text-xs font-bold text-slate-400 uppercase tracking-widest">Active Configuration</span>
                <div
                  aria-checked={configForm.is_active ?? true}
                  className={cn(
                    "relative inline-flex h-6 w-11 shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-primary-500/20 focus:ring-offset-2",
                    configForm.is_active ? "bg-primary-600" : "bg-slate-200 dark:bg-slate-800"
                  )}
                  onClick={() => { setConfigForm((prev) => ({ ...prev, is_active: !prev.is_active })); }}
                  onKeyDown={(e) => {
                    if (e.key === " " || e.key === "Enter") {
                      e.preventDefault();
                      setConfigForm((prev) => ({ ...prev, is_active: !prev.is_active }));
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

            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              <div className="space-y-2">
                <label className="text-[11px] font-bold text-slate-500 uppercase tracking-widest" htmlFor="provider-type-select">
                  Provider Type
                  <select
                    className="w-full h-12 px-3 mt-2 bg-slate-50 border border-slate-200 rounded-xl focus:ring-primary-500 focus:border-primary-500 dark:bg-slate-900/50 dark:border-slate-800 text-sm font-semibold"
                    id="provider-type-select"
                    onChange={(e) => { setConfigForm((prev) => ({ ...prev, provider_type: e.target.value })); }}
                    value={configForm.provider_type || "smartca"}
                  >
                    <option value="smartca">VN SmartCA / Remote Sign (VNPT)</option>
                    <option value="hsm">Corporate HSM Signature</option>
                    <option value="fpt">FPT CeCA Compliance Engine</option>
                    <option value="docusign">DocuSign International (eIDAS)</option>
                    <option value="mock">Sandbox Staging Mock Provider</option>
                  </select>
                </label>
              </div>

              <div className="space-y-2">
                <label className="text-[11px] font-bold text-slate-500 uppercase tracking-widest" htmlFor="default-auth-select">
                  Default Authorization Verification
                  <select
                    className="w-full h-12 px-3 mt-2 bg-slate-50 border border-slate-200 rounded-xl focus:ring-primary-500 focus:border-primary-500 dark:bg-slate-900/50 dark:border-slate-800 text-sm font-semibold"
                    id="default-auth-select"
                    onChange={(e) => { setConfigForm((prev) => ({ ...prev, default_auth_type: e.target.value as any })); }}
                    value={configForm.default_auth_type || "smartca"}
                  >
                    <option value="smartca">SmartCA Verification</option>
                    <option value="sms">SMS OTP Code</option>
                    <option value="email">Email OTP Code</option>
                    <option value="hsm">HSM Enterprise Sign</option>
                  </select>
                </label>
              </div>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              <FormField
                id="api-url-input"
                label="API Gateway Endpoint URL"
                onChange={(e) => { setConfigForm((prev) => ({ ...prev, api_url: e.target.value })); }}
                placeholder="VD: https://api.vnpt-smartca.vn/v1"
                required
                value={configForm.api_url || ""}
              />
              <FormField
                id="client-id-input"
                label="Integration Client ID / Access ID"
                onChange={(e) => { setConfigForm((prev) => ({ ...prev, client_id: e.target.value })); }}
                placeholder="VD: client_prod_88921"
                required
                value={configForm.client_id || ""}
              />
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              <FormField
                id="client-secret-input"
                label="Client Credentials Secret (Encrypted at Rest)"
                onChange={(e) => { setConfigForm((prev) => ({ ...prev, client_secret_encrypted: e.target.value })); }}
                placeholder="••••••••••••••••••••••••••••"
                required
                type="password"
                value={configForm.client_secret_encrypted || ""}
              />
              <FormField
                id="webhook-secret-input"
                label="Webhook Secret (Verify incoming callback events)"
                onChange={(e) => { setConfigForm((prev) => ({ ...prev, webhook_secret_encrypted: e.target.value })); }}
                placeholder="VD: whsec_abc123"
                value={configForm.webhook_secret_encrypted || ""}
              />
            </div>

            <FormField
              id="hsm-alias-input"
              label="HSM Key Alias (Optional, required if using HSM signatures)"
              onChange={(e) => { setConfigForm((prev) => ({ ...prev, hsm_key_alias: e.target.value })); }}
              placeholder="VD: alias_hsm_primary_01"
              value={configForm.hsm_key_alias || ""}
            />

            <div className="flex justify-end pt-4 border-t border-slate-50 dark:border-slate-800">
              <Button loading={configMutation.isPending} type="submit" variant="secondary">
                SAVE CONFIGURATIONS
              </Button>
            </div>
          </form>
        </TabsContent>

        {/* Tab 2 Content: Routing Priority Rules */}
        <TabsContent value="rules">
          <div className="space-y-6">
            <div className="flex justify-between items-center px-4">
              <div>
                <h3 className="text-sm font-bold uppercase tracking-widest text-slate-800 dark:text-slate-100 flex items-center gap-2">
                  <Globe className="text-primary-500" size={16} /> Signing Provider Routing Precedence Rules
                </h3>
                <p className="text-[10px] text-slate-400 font-semibold uppercase tracking-wider mt-0.5">
                  Route envelopes to specific signature providers based on country & verification criteria
                </p>
              </div>
              <Button icon={<Plus size={16} />} onClick={() => { setIsRuleModalOpen(true); }} variant="outline">
                ADD ROUTING RULE
              </Button>
            </div>

            <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-[2.5rem] overflow-hidden shadow-sm">
              {isLoadingRules ? (
                <div className="p-8 text-center text-xs text-slate-500 animate-pulse">Loading routing rules...</div>
              ) : employerRules.length === 0 ? (
                <EmptyState
                  action={<Button onClick={() => { setIsRuleModalOpen(true); }} variant="secondary">Create First Rule</Button>}
                  description="No custom routing rules established for this Legal Entity. All envelopes default to Mock Provider."
                  icon={<Layers size={40} />}
                  title="No Precedence Rules Found"
                />
              ) : (
                <table className="w-full text-left border-collapse">
                  <thead>
                    <tr className="bg-slate-50 dark:bg-slate-800/10">
                      <th className="px-8 py-5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest border-b border-slate-50 dark:border-slate-800">Signer Country</th>
                      <th className="px-8 py-5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest border-b border-slate-50 dark:border-slate-800">Signing Auth</th>
                      <th className="px-8 py-5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest border-b border-slate-50 dark:border-slate-800">Routed Provider</th>
                      <th className="px-8 py-5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest border-b border-slate-50 dark:border-slate-800 text-center">Priority</th>
                      <th className="px-8 py-5 border-b border-slate-50 dark:border-slate-800"></th>
                    </tr>
                  </thead>
                  <tbody>
                    {employerRules
                      .sort((a, b) => b.priority - a.priority)
                      .map((rule) => (
                        <tr className="border-b border-slate-50 dark:border-slate-800/50 hover:bg-slate-50/50 dark:hover:bg-slate-800/10 transition-colors" key={rule.id}>
                          <td className="px-8 py-6 text-xs font-mono font-bold text-slate-700 dark:text-slate-200">
                            {rule.signer_country}
                          </td>
                          <td className="px-8 py-6 text-xs font-bold text-slate-800 dark:text-slate-200 uppercase tracking-wider">
                            {rule.signing_method}
                          </td>
                          <td className="px-8 py-6 text-xs font-bold text-slate-800 dark:text-slate-200 uppercase tracking-wider">
                            {rule.target_provider_type}
                          </td>
                          <td className="px-8 py-6 text-center text-xs font-bold text-slate-800 dark:text-slate-200">
                            <span className="px-2.5 py-1 bg-slate-100 dark:bg-slate-800 rounded-lg shadow-sm">
                              {rule.priority}
                            </span>
                          </td>
                          <td className="px-8 py-6 text-right">
                            <Button
                              aria-label="Delete rule"
                              className="text-slate-400 hover:text-rose-600 hover:bg-rose-50 dark:hover:bg-rose-950/40 p-2 rounded-xl transition-colors h-auto w-auto shadow-none border-none"
                              icon={<Trash2 size={14} />}
                              onClick={() => { ruleDeleteMutation.mutate(rule.id); }}
                              size="sm"
                              variant="ghost"
                            />
                          </td>
                        </tr>
                      ))}
                  </tbody>
                </table>
              )}
            </div>
          </div>
        </TabsContent>

        {/* Tab 3 Content: Envelope Audit Dashboard */}
        <TabsContent value="envelopes">
          <div className="space-y-6">
            {/* Filter toolbar */}
            <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 p-6 rounded-[2rem] shadow-sm flex flex-col md:flex-row items-stretch md:items-center justify-between gap-4">
              <div className="relative flex-1 max-w-md">
                <Search className="absolute left-4 top-1/2 -translate-y-1/2 text-slate-400 dark:text-slate-500" size={16} />
                <input
                  className="w-full h-11 pl-11 pr-4 bg-slate-50 border border-slate-200 rounded-2xl text-xs font-semibold tracking-tight text-slate-700 dark:text-slate-200 placeholder:text-slate-400 focus:outline-none focus:ring-2 focus:ring-primary-500/20 focus:border-primary-500 transition-all dark:bg-slate-850 dark:border-slate-800"
                  onChange={(e) => { setEnvFilterSearch(e.target.value); }}
                  placeholder="Search envelopes by ID..."
                  type="text"
                  value={envFilterSearch}
                />
              </div>

              <div className="flex flex-wrap items-center gap-3">
                <div className="w-40">
                  <label className="sr-only" htmlFor="provider-filter">Filter by Provider</label>
                  <select
                    className="w-full h-11 px-3 bg-slate-50 border border-slate-200 rounded-xl text-xs font-bold uppercase tracking-wider text-slate-600 dark:text-slate-300 dark:bg-slate-850 dark:border-slate-800"
                    id="provider-filter"
                    onChange={(e) => { setEnvFilterProvider(e.target.value); }}
                    value={envFilterProvider}
                  >
                    <option value="all">All Providers</option>
                    <option value="smartca">VN SmartCA</option>
                    <option value="hsm">Corporate HSM</option>
                    <option value="fpt">FPT CeCA</option>
                    <option value="docusign">DocuSign</option>
                    <option value="mock">Mock Sandbox</option>
                  </select>
                </div>

                <div className="w-40">
                  <label className="sr-only" htmlFor="status-filter">Filter by Status</label>
                  <select
                    className="w-full h-11 px-3 bg-slate-50 border border-slate-200 rounded-xl text-xs font-bold uppercase tracking-wider text-slate-600 dark:text-slate-300 dark:bg-slate-850 dark:border-slate-800"
                    id="status-filter"
                    onChange={(e) => { setEnvFilterStatus(e.target.value); }}
                    value={envFilterStatus}
                  >
                    <option value="all">All Statuses</option>
                    <option value="draft">Draft</option>
                    <option value="sent">Sent</option>
                    <option value="completed">Completed</option>
                    <option value="voided">Voided</option>
                  </select>
                </div>
              </div>
            </div>

            <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-[2.5rem] overflow-hidden shadow-sm">
              {isLoadingEnvelopes ? (
                <div className="p-8 text-center text-xs text-slate-500 animate-pulse">Loading envelopes...</div>
              ) : filteredEnvelopes.length === 0 ? (
                <EmptyState
                  description="No signing envelope transactions match the filter criteria."
                  icon={<FileText size={40} />}
                  title="No Envelopes Found"
                />
              ) : (
                <table className="w-full text-left border-collapse">
                  <thead>
                    <tr className="bg-slate-50 dark:bg-slate-800/10">
                      <th className="px-8 py-5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest border-b border-slate-50 dark:border-slate-800">External ID</th>
                      <th className="px-8 py-5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest border-b border-slate-50 dark:border-slate-800">Provider</th>
                      <th className="px-8 py-5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest border-b border-slate-50 dark:border-slate-800">Status</th>
                      <th className="px-8 py-5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest border-b border-slate-50 dark:border-slate-800">Date Generated</th>
                      <th className="px-8 py-5 border-b border-slate-50 dark:border-slate-800"></th>
                    </tr>
                  </thead>
                  <tbody>
                    {filteredEnvelopes.map((env) => (
                      <tr className="border-b border-slate-50 dark:border-slate-800/50 hover:bg-slate-50/50 dark:hover:bg-slate-800/10 transition-colors" key={env.id}>
                        <td className="px-8 py-6 text-xs font-mono font-bold text-slate-800 dark:text-slate-200">
                          {env.external_envelope_id}
                        </td>
                        <td className="px-8 py-6 text-xs font-bold uppercase tracking-wider text-slate-700 dark:text-slate-350">
                          {env.provider_type}
                        </td>
                        <td className="px-8 py-6">
                          <span
                            className={cn(
                              "text-[9px] font-bold uppercase tracking-widest px-2.5 py-0.5 rounded-full border shadow-sm",
                              env.status === "completed"
                                ? "bg-emerald-50 text-emerald-600 border-emerald-100 dark:bg-emerald-950/40 dark:text-emerald-300 dark:border-emerald-800"
                                : env.status === "sent"
                                ? "bg-amber-50 text-amber-600 border-amber-100 dark:bg-amber-950/40 dark:text-amber-300 dark:border-amber-800"
                                : "bg-slate-50 text-slate-500 border-slate-100 dark:bg-slate-800/50 dark:text-slate-400 dark:border-slate-750"
                            )}
                          >
                            {env.status}
                          </span>
                        </td>
                        <td className="px-8 py-6 text-xs font-mono text-slate-500 dark:text-slate-400">
                          {new Date(env.created_at).toLocaleDateString()}
                        </td>
                        <td className="px-8 py-6 text-right">
                          <Button
                            className="flex items-center gap-1 text-[10px] font-bold text-slate-500 hover:text-primary-600 px-3 py-1.5 rounded-xl border border-slate-200 dark:border-slate-850 hover:bg-slate-50 transition-colors"
                            icon={<Eye size={12} />}
                            onClick={() => { setSelectedEnvelopeForLog(env); }}
                            size="sm"
                            variant="ghost"
                          >
                            WEBHOOK LOGS
                          </Button>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              )}
            </div>
          </div>
        </TabsContent>
      </Tabs>

      {/* Creation Modal for Routing Rules */}
      <Modal
        icon={<Plus size={24} />}
        isOpen={isRuleModalOpen}
        onClose={() => { setIsRuleModalOpen(false); }}
        subtitle="Add specific routing configuration priority rules"
        title="Add Routing Rule"
      >
        <form className="space-y-6 pt-4" noValidate onSubmit={handleCreateRule}>
          <div className="grid grid-cols-2 gap-4">
            <FormField
              id="rule-country-input"
              label="Signer Country Code (e.g. VN, US)"
              onChange={(e) => { setNewRuleForm((prev) => ({ ...prev, signer_country: e.target.value.toUpperCase() })); }}
              required
              value={newRuleForm.signer_country || ""}
            />
            <div className="space-y-2">
              <label className="text-[11px] font-bold text-slate-500 uppercase tracking-widest" htmlFor="rule-auth-select">
                Preferred Signing Auth
                <select
                  className="w-full h-10 px-3 mt-2 bg-slate-50 border border-slate-200 rounded-xl focus:ring-primary-500 focus:border-primary-500 dark:bg-slate-900/50 dark:border-slate-800 text-sm font-semibold"
                  id="rule-auth-select"
                  onChange={(e) => { setNewRuleForm((prev) => ({ ...prev, signing_method: e.target.value as any })); }}
                  value={newRuleForm.signing_method || "smartca"}
                >
                  <option value="smartca">SmartCA Remote OTP</option>
                  <option value="sms">SMS OTP verification</option>
                  <option value="email">Email OTP verification</option>
                  <option value="hsm">HSM Enterprise Sign</option>
                </select>
              </label>
            </div>
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-2">
              <label className="text-[11px] font-bold text-slate-500 uppercase tracking-widest" htmlFor="rule-provider-select">
                Target Provider Route
                <select
                  className="w-full h-10 px-3 mt-2 bg-slate-50 border border-slate-200 rounded-xl focus:ring-primary-500 focus:border-primary-500 dark:bg-slate-900/50 dark:border-slate-800 text-sm font-semibold"
                  id="rule-provider-select"
                  onChange={(e) => { setNewRuleForm((prev) => ({ ...prev, target_provider_type: e.target.value })); }}
                  value={newRuleForm.target_provider_type || "smartca"}
                >
                  <option value="smartca">VN SmartCA</option>
                  <option value="hsm">Corporate HSM</option>
                  <option value="fpt">FPT CeCA</option>
                  <option value="docusign">DocuSign</option>
                  <option value="mock">Mock Sandbox</option>
                </select>
              </label>
            </div>
            <FormField
              id="rule-priority-input"
              label="Priority Weight (Integer)"
              onChange={(e) => { setNewRuleForm((prev) => ({ ...prev, priority: parseInt(e.target.value) || 1 })); }}
              required
              type="number"
              value={String(newRuleForm.priority || 1)}
            />
          </div>

          <div className="flex items-center justify-end gap-3 pt-6 border-t border-slate-100 dark:border-slate-800">
            <Button onClick={() => { setIsRuleModalOpen(false); }} type="button" variant="outline">
              CANCEL
            </Button>
            <Button type="submit" variant="secondary">
              ADD RULE
            </Button>
          </div>
        </form>
      </Modal>

      {/* Webhook log details Modal */}
      <Modal
        icon={<AlertCircle size={24} />}
        isOpen={Boolean(selectedEnvelopeForLog)}
        onClose={() => { setSelectedEnvelopeForLog(null); }}
        subtitle={`Audit events & incoming webhook responses for ${selectedEnvelopeForLog?.external_envelope_id}`}
        title="Webhooks & Audit Trail"
      >
        {selectedEnvelopeForLog && (
          <div className="space-y-6 pt-4">
            <div className="bg-slate-950 p-6 rounded-3xl font-mono text-[11px] text-emerald-400 overflow-x-auto select-all max-h-96 shadow-inner">
              <pre>{JSON.stringify(selectedEnvelopeForLog.audit_trail_metadata || { message: "No callback metadata captured" }, null, 2)}</pre>
            </div>
            <div className="flex items-center justify-end pt-4 border-t border-slate-100 dark:border-slate-800">
              <Button onClick={() => { setSelectedEnvelopeForLog(null); }} type="button" variant="outline">
                CLOSE
              </Button>
            </div>
          </div>
        )}
      </Modal>
    </div>
  );
}
