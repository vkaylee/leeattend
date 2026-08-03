import { useState, useCallback } from "react";

import type { ChecklistItemStatus, LifecycleChecklistItem } from "@/lib/api/types";

import { useToast } from "@/components/ui/toast";
import {
  useLifecycleChecklists,
  useCreateLifecycleChecklist,
  useUpdateLifecycleChecklistItem,
  useLifecycleTemplates,
} from "@/hooks/use-lifecycle";
import { maybeNull } from "@/types/api-overrides";

export function useLifecycleChecklistState(employmentId: string) {
  const toast = useToast();

  const [selectedTemplateId, setSelectedTemplateId] = useState("");
  const [userSelectedChecklistId, setUserSelectedChecklistId] = useState<string | null>(null);
  const [editingNotesItemId, setEditingNotesItemId] = useState<string | null>(null);
  const [itemNotes, setItemNotes] = useState("");

  // Queries
  const { data: checklists = [], isLoading } = useLifecycleChecklists(employmentId);
  const { data: templates = [] } = useLifecycleTemplates();

  // Mutations
  const createChecklistMutation = useCreateLifecycleChecklist(employmentId);
  const firstChecklist = checklists.length > 0 ? checklists[0] : undefined;
  const activeChecklistId: string | null = userSelectedChecklistId ?? firstChecklist?.id ?? null;
  const updateItemStatusMutation = useUpdateLifecycleChecklistItem(activeChecklistId ?? "", employmentId);

  // Active checklist details
  const activeChecklist = checklists.find((c) => c.id === activeChecklistId) ?? checklists[0];

  // Filter templates that are active
  const availableTemplates = templates.filter(t => t.is_active);

  const handleCreateChecklist = useCallback(() => {
    if (!selectedTemplateId) {
      toast.error("Please select a checklist template");
      return;
    }
    createChecklistMutation.mutate(
      { template_id: selectedTemplateId },
      {
        onSuccess: (res) => {
          toast.success("Checklist instantiated successfully");
          setUserSelectedChecklistId(res.id);
          setSelectedTemplateId("");
        },
        onError: (err) => {
          toast.error(err instanceof Error ? err.message : "Failed to instantiate checklist");
        },
      }
    );
  }, [selectedTemplateId, createChecklistMutation, toast]);

  const handleToggleItemStatus = useCallback((item: LifecycleChecklistItem) => {
    const newStatus: ChecklistItemStatus = item.status === "completed" ? "pending" : "completed";
    updateItemStatusMutation.mutate(
      {
        itemId: item.id,
        payload: { status: newStatus, notes: item.notes ?? undefined },
      },
      {
        onSuccess: () => { toast.success(`Task marked as ${newStatus}`); },
        onError: (err) => {
          toast.error(err instanceof Error ? err.message : "Failed to update item status");
        },
      }
    );
  }, [updateItemStatusMutation, toast]);

  const handleEditNotes = useCallback((item: LifecycleChecklistItem) => {
    setEditingNotesItemId(item.id);
    setItemNotes(item.notes ?? "");
  }, []);

  const handleSaveNotes = useCallback((item: LifecycleChecklistItem) => {
    updateItemStatusMutation.mutate(
      {
        itemId: item.id,
        payload: { status: item.status, notes: itemNotes.trim() ? itemNotes : null },
      },
      {
        onSuccess: () => {
          toast.success("Task notes updated");
          setEditingNotesItemId(null);
        },
        onError: (err) => {
          toast.error(err instanceof Error ? err.message : "Failed to update notes");
        },
      }
    );
  }, [updateItemStatusMutation, toast, itemNotes]);

  const sortedItems = maybeNull(activeChecklist)?.items
    ? [...activeChecklist.items].sort((a: LifecycleChecklistItem, b: LifecycleChecklistItem) => a.display_order - b.display_order)
    : [];

  return {
    // State
    selectedTemplateId,
    setSelectedTemplateId,
    userSelectedChecklistId,
    setUserSelectedChecklistId,
    editingNotesItemId,
    setEditingNotesItemId,
    itemNotes,
    setItemNotes,
    // Data
    checklists,
    isLoading,
    templates,
    availableTemplates,
    activeChecklist,
    activeChecklistId,
    sortedItems,
    // Handlers
    handleCreateChecklist,
    handleToggleItemStatus,
    handleEditNotes,
    handleSaveNotes,
  };
}
