import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import { useToast } from "@/components/ui/toast";
import { useConfirm } from "@/hooks/use-confirm";
import { 
  createAssignmentRule, 
  deleteAssignmentRule,
  updateAssignmentRule,
  AssignmentRule,
  AssignmentRuleConditions
} from "@/lib/api";

import { useAssignmentRulesData } from "./use-assignment-rules-data";

export function useAssignmentRules() {
  const { t } = useTranslation();
  const { wsId, rules, rulesLoading, patterns, departments } = useAssignmentRulesData();
  const queryClient = useQueryClient();
  const toast = useToast();
  const { confirm, ConfirmationDialog } = useConfirm();

  const [isModalOpen, setIsModalOpen] = useState(false);
  const [editingRule, setEditingRule] = useState<AssignmentRule | null>(null);

  const [name, setName] = useState("");
  const [patternId, setPatternId] = useState("");
  const [priority, setPriority] = useState(0);
  const [conditions, setConditions] = useState<AssignmentRuleConditions>({});

  const closeModal = () => { 
    setIsModalOpen(false); 
    setEditingRule(null); 
  };

  const createMutation = useMutation({
    mutationFn: (data: Parameters<typeof createAssignmentRule>[1]) => createAssignmentRule(wsId, data),
    onSuccess: () => { 
      toast.success(t("rules.messages.create_success")); 
      closeModal(); 
      void queryClient.invalidateQueries({ queryKey: ["assignment-rules"] }); 
    },
  });

  const updateMutation = useMutation({
    mutationFn: ({ id, data }: { id: string, data: Parameters<typeof updateAssignmentRule>[2] }) => updateAssignmentRule(wsId, id, data),
    onSuccess: () => { 
      toast.success(t("rules.messages.update_success")); 
      closeModal(); 
      void queryClient.invalidateQueries({ queryKey: ["assignment-rules"] }); 
    },
  });

  const deleteMutation = useMutation({
    mutationFn: (id: string) => deleteAssignmentRule(wsId, id),
    onSuccess: () => { 
      toast.success(t("rules.messages.delete_success")); 
      void queryClient.invalidateQueries({ queryKey: ["assignment-rules"] }); 
    }
  });

  const openCreateModal = () => { 
    setName(""); 
    setPatternId(""); 
    setPriority(0); 
    setConditions({}); 
    setIsModalOpen(true); 
  };

  const openEditModal = (rule: AssignmentRule) => { 
    setEditingRule(rule); 
    setName(rule.name); 
    setPatternId(rule.pattern_id); 
    setPriority(rule.priority); 
    setConditions(rule.conditions); 
    setIsModalOpen(true); 
  };

  const handleSubmit = () => {
    if (!name || !patternId) return;
    const payload = { name, pattern_id: patternId, priority, conditions };
    if (editingRule) {
      updateMutation.mutate({ id: editingRule.id, data: payload });
    } else {
      createMutation.mutate(payload);
    }
  };

  const handleDelete = async (id: string) => {
    const ok = await confirm({ 
      title: t("common.confirm_delete"), 
      message: t("rules.confirm_delete_msg", "Are you sure you want to delete this rule?"), 
      variant: "danger" 
    }); 
    if (ok) {
      deleteMutation.mutate(id);
    }
  };

  return {
    t,
    rules,
    rulesLoading,
    patterns,
    departments,
    isModalOpen,
    editingRule,
    name,
    setName,
    patternId,
    setPatternId,
    priority,
    setPriority,
    conditions,
    setConditions,
    openCreateModal,
    openEditModal,
    closeModal,
    handleSubmit,
    handleDelete,
    isPending: createMutation.isPending || updateMutation.isPending,
    ConfirmationDialog
  };
}
