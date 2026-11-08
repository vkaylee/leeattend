import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import type { Employer } from "@/lib/api/types";

import { useToast } from "@/components/ui/toast";
import {
  fetchEContractConfigs,
  saveEContractConfig,
  fetchEContractRules,
  saveEContractRule,
  deleteEContractRule,
  fetchEContractEnvelopes,
  EContractConfig,
  EContractRoutingRule,
} from "@/lib/api/econtracts";
import { fetchEmployers } from "@/lib/api/employers";

function getErrorMessage(err: unknown, fallback: string): string {
  if (err instanceof Error) {
    return err.message;
  }
  return fallback;
}

export function useEContractsSettings() {
  const { t } = useTranslation();
  const toast = useToast();
  const queryClient = useQueryClient();

  const [activeTab, setActiveTab] = useState<string>("configs");
  const [selectedEmployerId, setSelectedEmployerId] = useState<string>("default");

  // Queries
  const { data: employers = [], isLoading: isLoadingEmployers } = useQuery({
    queryKey: ["employers"],
    queryFn: () => fetchEmployers(),
  });

  const activeEmployerId =
    selectedEmployerId === "default"
      ? employers.length > 0
        ? employers[0].id
        : ""
      : selectedEmployerId;

  const hasEmployers = employers.length > 0;
  const isEnabled = hasEmployers;

  const { data: configs = [] } = useQuery({
    queryKey: ["econtract-configs"],
    queryFn: () => fetchEContractConfigs(employers),
    enabled: isEnabled,
  });

  const { data: rules = [], isLoading: isLoadingRules } = useQuery({
    queryKey: ["econtract-rules"],
    queryFn: () => fetchEContractRules(employers),
    enabled: isEnabled,
  });

  const { data: envelopes = [], isLoading: isLoadingEnvelopes } = useQuery({
    queryKey: ["econtract-envelopes"],
    queryFn: () => fetchEContractEnvelopes(employers),
    enabled: isEnabled,
  });

  // Mutations
  const configMutation = useMutation({
    mutationFn: (config: EContractConfig) => saveEContractConfig(config),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["econtract-configs"] });
      toast.success(t("common.saved_success", { defaultValue: "Saved configuration successfully!" }));
    },
    onError: (err: unknown) => {
      toast.error(getErrorMessage(err, "Failed to save configuration"));
    },
  });

  const ruleMutation = useMutation({
    mutationFn: (rule: Omit<EContractRoutingRule, "id" | "created_at">) => saveEContractRule(rule),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["econtract-rules"] });
      toast.success("Routing rule added successfully!");
    },
    onError: (err: unknown) => {
      toast.error(getErrorMessage(err, "Failed to add routing rule"));
    },
  });

  const ruleDeleteMutation = useMutation({
    mutationFn: (ruleId: string) => deleteEContractRule(ruleId),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["econtract-rules"] });
      toast.success("Routing rule deleted successfully!");
    },
    onError: (err: unknown) => {
      toast.error(getErrorMessage(err, "Failed to delete routing rule"));
    },
  });

  const activeEmployer = employers.find((e: Employer) => e.id === activeEmployerId);
  const activeEmployerName = activeEmployer ? activeEmployer.name : "Select Legal Entity";

  const handleEmployerChange = (val: string) => {
    setSelectedEmployerId(val);
  };

  return {
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
  };
}
