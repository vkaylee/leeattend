import { FileText, Layers, Scale, Settings } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import type { Employer } from "@/lib/api/types";

import { Combobox } from "@/components/ui/combobox";
import PageHeader from "@/components/ui/page-header";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";

import EContractsCredentialsTab from "./components/econtracts-credentials-tab";
import EContractsEnvelopesTab from "./components/econtracts-envelopes-tab";
import EContractsRulesTab from "./components/econtracts-rules-tab";
import { useEContractsSettings } from "./hooks/use-econtracts-settings";

export default function EContractsSettingsPage() {
  const { t } = useTranslation();
  const toast = useToast();
  const {
    activeEmployerId,
    activeEmployerName,
    activeTab,
    configs,
    configMutation,
    employers,
    envelopes,
    handleEmployerChange,
    isLoadingEmployers,
    isLoadingEnvelopes,
    isLoadingRules,
    ruleDeleteMutation,
    ruleMutation,
    rules,
    setActiveTab,
    wsId,
  } = useEContractsSettings();

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

      <LegalEntityPicker
        activeEmployerId={activeEmployerId}
        activeEmployerName={activeEmployerName}
        employers={employers}
        isLoadingEmployers={isLoadingEmployers}
        onEmployerChange={handleEmployerChange}
      />

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
          <EContractsCredentialsTab
            configs={configs}
            isSaving={configMutation.isPending}
            key={`${activeEmployerId}-${configs.length}`}
            onSave={(config) => {
              if (!activeEmployerId) {
                toast.error("Please select a Legal Entity.");
                return;
              }
              configMutation.mutate(config);
            }}
            selectedEmployerId={activeEmployerId}
          />
        </TabsContent>

        {/* Tab 2 Content: Routing Priority Rules */}
        <TabsContent value="rules">
          <EContractsRulesTab
            isCreatingRule={ruleMutation.isPending}
            isDeletingRule={ruleDeleteMutation.isPending}
            isLoading={isLoadingRules}
            onCreateRule={(rule) => {
              ruleMutation.mutate(rule);
            }}
            onDeleteRule={(ruleId) => {
              ruleDeleteMutation.mutate(ruleId);
            }}
            rules={rules}
            selectedEmployerId={activeEmployerId}
          />
        </TabsContent>

        {/* Tab 3 Content: Envelope Audit Dashboard */}
        <TabsContent value="envelopes">
          <EContractsEnvelopesTab envelopes={envelopes} isLoading={isLoadingEnvelopes} />
        </TabsContent>
      </Tabs>
    </div>
  );
}

interface LegalEntityPickerProps {
  activeEmployerId: string;
  activeEmployerName: string;
  employers: Employer[];
  isLoadingEmployers: boolean;
  onEmployerChange: (id: string) => void;
}

function LegalEntityPicker({
  activeEmployerId,
  activeEmployerName,
  employers,
  isLoadingEmployers,
  onEmployerChange,
}: LegalEntityPickerProps) {
  return (
    <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 p-6 rounded-[2rem] shadow-sm flex flex-col md:flex-row items-start md:items-center justify-between gap-4">
      <div>
        <Typography as="h2" className="text-xs font-bold text-slate-400 uppercase tracking-widest mb-1" variant="h2">
          Active Legal Entity Context
        </Typography>
        <Typography className="text-sm font-bold text-slate-700 dark:text-slate-200" variant="p">
          {activeEmployerName.toUpperCase()}
        </Typography>
      </div>
      <div className="w-full md:w-80">
        <label className="sr-only" htmlFor="employer-select">Select Legal Entity</label>
        <Combobox
          emptyText="No employers found"
          id="employer-select"
          onChange={onEmployerChange}
          options={employers.map((emp: Employer) => ({
            value: emp.id,
            label: emp.name.toUpperCase(),
          }))}
          placeholder={isLoadingEmployers ? "Loading Legal Entities..." : "Select Legal Entity"}
          value={activeEmployerId}
        />
      </div>
    </div>
  );
}
