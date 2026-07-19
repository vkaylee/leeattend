import { useState, useEffect } from "react";

import type { LifecycleChecklistTemplate } from "@/lib/api/types";

import Button from "@/components/ui/button";
import FormField from "@/components/ui/form-field";
import Sheet from "@/components/ui/sheet";
import { useToast } from "@/components/ui/toast";
import {
  useCreateLifecycleTemplate,
  useUpdateLifecycleTemplate,
} from "@/hooks/use-lifecycle";

type ChecklistType = "onboarding" | "offboarding" | "transfer" | "probation_end";

interface TemplateDrawerProps {
  activeTab: ChecklistType;
  editingTemplate: LifecycleChecklistTemplate | null;
  isOpen: boolean;
  onClose: () => void;
}

export function TemplateDrawer({ isOpen, onClose, editingTemplate, activeTab }: TemplateDrawerProps) {
  const toast = useToast();
  const createMutation = useCreateLifecycleTemplate();
  const updateMutation = useUpdateLifecycleTemplate();

  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [isDefault, setIsDefault] = useState(false);

  useEffect(() => {
    if (editingTemplate) {
  // eslint-disable-next-line react-hooks/set-state-in-effect
      setName(editingTemplate.name);
      setDescription(editingTemplate.description ?? "");
      setIsDefault(editingTemplate.is_default);
    } else {
      setName("");
      setDescription("");
      setIsDefault(false);
    }
  }, [editingTemplate, isOpen]);

  const handleSave = () => {
    if (!name.trim()) {
      toast.error("Template name is required");
      return;
    }

    const payload = {
      name,
      description: description.trim() ? description : undefined,
      checklist_type: activeTab,
      is_default: isDefault,
    };

    if (editingTemplate) {
      updateMutation.mutate(
        {
          id: editingTemplate.id,
          payload: {
            name: payload.name,
            description: payload.description,
            is_default: payload.is_default,
          },
        },
        {
          onSuccess: () => {
            toast.success("Template updated successfully");
            onClose();
          },
          onError: (err) => {
            toast.error(err instanceof Error ? err.message : "Failed to update template");
          },
        }
      );
    } else {
      createMutation.mutate(payload, {
        onSuccess: () => {
          toast.success("Template created successfully");
          onClose();
        },
        onError: (err) => {
          toast.error(err instanceof Error ? err.message : "Failed to create template");
        },
      });
    }
  };

  return (
    <Sheet
      isOpen={isOpen}
      onClose={onClose}
      title={editingTemplate ? "Edit Lifecycle Template" : "Create Lifecycle Template"}
    >
      <div className="space-y-6 pt-4">
        <FormField
          label="Template Name"
          onChange={(e) => { setName(e.target.value); }}
          placeholder="e.g., General Engineering Onboarding"
          value={name}
        />
        <FormField
          label="Description"
          onChange={(e) => { setDescription(e.target.value); }}
          placeholder="Describe when to apply this template or what it covers"
          type="textarea"
          value={description}
        />
        <div className="flex items-center gap-3 bg-slate-50 dark:bg-slate-800/30 p-4 rounded-2xl border border-slate-100 dark:border-slate-800">
          <input
            checked={isDefault}
            className="w-4 h-4 text-primary-600 focus:ring-primary-500 border-slate-300 rounded"
            id="isDefault"
            onChange={(e) => { setIsDefault(e.target.checked); }}
            type="checkbox"
          />
          <label className="text-sm font-semibold text-slate-700 dark:text-slate-300 cursor-pointer" htmlFor="isDefault">
            Set as Default Template for {activeTab.replace("_", " ")}
          </label>
        </div>

        <div className="flex justify-end gap-3 pt-6 border-t border-slate-100 dark:border-slate-800">
          <Button onClick={onClose} variant="ghost">
            CANCEL
          </Button>
          <Button onClick={handleSave}>
            SAVE TEMPLATE
          </Button>
        </div>
      </div>
    </Sheet>
  );
}
