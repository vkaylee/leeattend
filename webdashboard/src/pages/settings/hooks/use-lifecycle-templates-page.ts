import { useState, useMemo } from "react";

import type {
  LifecycleChecklistTemplate,
  LifecycleChecklistTemplateItem,
} from "@/lib/api/types";

import { useToast } from "@/components/ui/toast";
import { useConfirm } from "@/hooks/use-confirm";
import { useDerivedStateFrom } from "@/hooks/use-derived-state-from";
import {
  useLifecycleTemplates,
  useUpdateLifecycleTemplate,
  useDeleteLifecycleTemplate,
  useLifecycleTemplateItems,
  useDeleteLifecycleTemplateItem,
} from "@/hooks/use-lifecycle";

type ChecklistType = "onboarding" | "offboarding" | "transfer" | "probation_end";

export function useLifecycleTemplatesPage() {
  const { confirm } = useConfirm();
  const toast = useToast();

  const [activeTab, setActiveTab] = useState<ChecklistType>("onboarding");

  // Drawers State
  const [isTemplateDrawerOpen, setIsTemplateDrawerOpen] = useState(false);
  const [editingTemplate, setEditingTemplate] = useState<LifecycleChecklistTemplate | null>(null);
  const [isItemDrawerOpen, setIsItemDrawerOpen] = useState(false);
  const [editingItem, setEditingItem] = useState<LifecycleChecklistTemplateItem | null>(null);

  // Queries
  const { data: templates = [], isLoading: isTemplatesLoading } = useLifecycleTemplates();

  // Mutations
  const updateTemplateMutation = useUpdateLifecycleTemplate();
  const deleteTemplateMutation = useDeleteLifecycleTemplate();

  const filteredTemplates = templates.filter((temp) => temp.checklist_type === activeTab);

  // Default selected template
  const defaultTemplateId = useMemo(() => {
    if (filteredTemplates.length > 0) {
      const defaultTemp = filteredTemplates.find((t) => t.is_default) ?? filteredTemplates[0];
      return defaultTemp.id;
    }
    return null;
  }, [filteredTemplates]);

  const [selectedTemplateId, setSelectedTemplateId] = useDerivedStateFrom(defaultTemplateId);

  // Item queries
  const { data: items = [], isLoading: isItemsLoading } = useLifecycleTemplateItems(selectedTemplateId ?? "");
  const deleteItemMutation = useDeleteLifecycleTemplateItem(selectedTemplateId ?? "");

  const activeTemplate = templates.find((t) => t.id === selectedTemplateId);

  // ── Template handlers ──────────────────────────────────────────────

  const handleCreateTemplate = () => {
    setEditingTemplate(null);
    setIsTemplateDrawerOpen(true);
  };

  const handleEditTemplate = (template: LifecycleChecklistTemplate, e: React.MouseEvent) => {
    e.stopPropagation();
    setEditingTemplate(template);
    setIsTemplateDrawerOpen(true);
  };

  const handleDeleteTemplate = async (template: LifecycleChecklistTemplate, e: React.MouseEvent) => {
    e.stopPropagation();
    const ok = await confirm({
      title: "Delete Template",
      message: `Are you sure you want to delete template "${template.name}"? This action cannot be undone.`,
      variant: "danger",
    });
    if (ok) {
      deleteTemplateMutation.mutate(template.id, {
        onSuccess: () => {
          toast.success("Template deleted successfully");
          if (selectedTemplateId === template.id) setSelectedTemplateId(null);
        },
        onError: (err) => {
          toast.error(err instanceof Error ? err.message : "Failed to delete template");
        },
      });
    }
  };

  const handleSetDefault = (template: LifecycleChecklistTemplate, e: React.MouseEvent) => {
    e.stopPropagation();
    updateTemplateMutation.mutate(
      { id: template.id, payload: { is_default: true } },
      {
        onSuccess: () => { toast.success(`Set "${template.name}" as default template`); },
        onError: (err) => { toast.error(err instanceof Error ? err.message : "Failed to set default template"); },
      },
    );
  };

  const handleToggleActive = (template: LifecycleChecklistTemplate, e: React.MouseEvent) => {
    e.stopPropagation();
    updateTemplateMutation.mutate(
      { id: template.id, payload: { is_active: !template.is_active } },
      {
        onSuccess: () => { toast.success(`Template ${template.is_active ? "disabled" : "enabled"} successfully`); },
        onError: (err) => { toast.error(err instanceof Error ? err.message : "Failed to update template status"); },
      },
    );
  };

  // ── Item handlers ──────────────────────────────────────────────────

  const handleCreateItem = () => {
    setEditingItem(null);
    setIsItemDrawerOpen(true);
  };

  const handleEditItem = (item: LifecycleChecklistTemplateItem) => {
    setEditingItem(item);
    setIsItemDrawerOpen(true);
  };

  const handleDeleteItem = async (item: LifecycleChecklistTemplateItem) => {
    const ok = await confirm({
      title: "Delete Checklist Item",
      message: `Are you sure you want to delete checklist item "${item.title}"?`,
      variant: "danger",
    });
    if (ok) {
      deleteItemMutation.mutate(item.id, {
        onSuccess: () => { toast.success("Checklist item deleted successfully"); },
        onError: (err) => { toast.error(err instanceof Error ? err.message : "Failed to delete checklist item"); },
      });
    }
  };

  return {
    // State
    activeTab,
    setActiveTab,
    // Templates
    templates,
    filteredTemplates,
    isTemplatesLoading,
    selectedTemplateId,
    setSelectedTemplateId,
    activeTemplate,
    // Items
    items,
    isItemsLoading,
    // Drawers
    isTemplateDrawerOpen,
    setIsTemplateDrawerOpen,
    editingTemplate,
    isItemDrawerOpen,
    setIsItemDrawerOpen,
    editingItem,
    // Handlers
    handleCreateTemplate,
    handleEditTemplate,
    handleDeleteTemplate,
    handleSetDefault,
    handleToggleActive,
    handleCreateItem,
    handleEditItem,
    handleDeleteItem,
  };
}
