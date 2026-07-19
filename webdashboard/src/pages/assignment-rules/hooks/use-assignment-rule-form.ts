import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import { useToast } from "@/components/ui/toast";
import {
  createAssignmentRule,
  updateAssignmentRule,
  getCurrentWorkspaceId,
  AssignmentRuleConditions,
} from "@/lib/api";

import { useAssignmentRulesData } from "./use-assignment-rules-data";

export function useAssignmentRuleForm(ruleId?: string) {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const toast = useToast();
  const queryClient = useQueryClient();
  const wsId = getCurrentWorkspaceId() ?? "";
  const isEditMode = !!ruleId;

  // Form State
  const [name, setName] = useState("");
  const [patternId, setPatternId] = useState("");
  const [priority, setPriority] = useState(0);
  const [conditions, setConditions] = useState<AssignmentRuleConditions>({});
  const [startDate, setStartDate] = useState(() => new Date().toISOString().split("T")[0]);
  const [endDate, setEndDate] = useState("");
  const [isIndefinite, setIsIndefinite] = useState(true);
  const [isInitialized, setIsInitialized] = useState(false);
  const [isDirty, setIsDirty] = useState(false);

  // Load context data (patterns, departments, employers, locations)
  const { patterns, departments, employers, locations, rules, rulesLoading } = useAssignmentRulesData();

  const currentRule = rules?.find((r) => r.id === ruleId);

  // Initialize form when data becomes available
  if (currentRule && isEditMode && !isInitialized && !rulesLoading) {
    setIsInitialized(true);
    setName(currentRule.name);
    setPatternId(currentRule.pattern_id);
    setPriority(currentRule.priority);
    setConditions(currentRule.conditions);
    setStartDate(currentRule.start_date ?? "");
    setEndDate(currentRule.end_date ?? "");
    setIsIndefinite(!currentRule.end_date);
  }

  // Mutations
  const createMutation = useMutation({
    mutationFn: (data: Parameters<typeof createAssignmentRule>[1]) => createAssignmentRule(wsId, data),
    onSuccess: () => {
      toast.success(t("rules.messages.create_success"));
      setIsDirty(false);
      void queryClient.invalidateQueries({ queryKey: ["assignment-rules"] });
      void navigate({ to: "/shifts/rules", search: { tab: "rules" } });
    },
    onError: () => {
      toast.error(t("rules.messages.create_error", "Failed to create rule"));
    },
  });

  const updateMutation = useMutation({
    mutationFn: ({ id, data }: { id: string; data: Parameters<typeof updateAssignmentRule>[2] }) =>
      updateAssignmentRule(wsId, id, data),
    onSuccess: () => {
      toast.success(t("rules.messages.update_success"));
      setIsDirty(false);
      void queryClient.invalidateQueries({ queryKey: ["assignment-rules"] });
      void navigate({ to: "/shifts/rules", search: { tab: "rules" } });
    },
    onError: () => {
      toast.error(t("rules.messages.update_error", "Failed to update rule"));
    },
  });

  const handleFieldChange = <T>(setter: React.Dispatch<React.SetStateAction<T>>, value: T) => {
    setter(value);
    setIsDirty(true);
  };

  const handleConditionsChange = (newConditions: AssignmentRuleConditions) => {
    setConditions(newConditions);
    setIsDirty(true);
  };

  // eslint-disable-next-line @typescript-eslint/no-deprecated
  const handleSubmit = (e?: React.FormEvent) => {
    if (e) e.preventDefault();
    if (!name || !patternId) return;

    const payload = {
      name,
      pattern_id: patternId,
      priority,
      conditions,
      start_date: startDate || null,
      end_date: isIndefinite ? null : (endDate || null),
    };

    if (isEditMode && ruleId) {
      updateMutation.mutate({ id: ruleId, data: payload });
    } else {
      createMutation.mutate(payload);
    }
  };

  return {
    t,
    wsId,
    isEditMode,
    rulesLoading,
    patterns,
    departments,
    employers,
    locations,
    name,
    setName: (val: string) => { handleFieldChange(setName, val); },
    patternId,
    setPatternId: (val: string) => { handleFieldChange(setPatternId, val); },
    priority,
    setPriority: (val: number) => { handleFieldChange(setPriority, val); },
    conditions,
    setConditions: handleConditionsChange,
    startDate,
    setStartDate: (val: string) => { handleFieldChange(setStartDate, val); },
    endDate,
    setEndDate: (val: string) => { handleFieldChange(setEndDate, val); },
    isIndefinite,
    setIsIndefinite: (val: boolean) => { handleFieldChange(setIsIndefinite, val); },
    handleSubmit,
    isPending: createMutation.isPending || updateMutation.isPending,
    isDirty,
    setIsDirty,
  };
}
