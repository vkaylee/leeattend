import { useMemo } from "react";

import type {
  LifecycleChecklistTemplateItem,
  ChecklistCategory,
  ChecklistAssigneeType,
} from "@/lib/api/types";

import Button from "@/components/ui/button";
import { Combobox } from "@/components/ui/combobox";
import FormField from "@/components/ui/form-field";
import Sheet from "@/components/ui/sheet";
import { useToast } from "@/components/ui/toast";
import { useDerivedStateFrom } from "@/hooks/use-derived-state-from";
import {
  useCreateLifecycleTemplateItem,
  useUpdateLifecycleTemplateItem,
} from "@/hooks/use-lifecycle";
import { useWorkspaceMembers } from "@/hooks/use-workspace";

interface TemplateItemDrawerProps {
  editingItem: LifecycleChecklistTemplateItem | null;
  isOpen: boolean;
  onClose: () => void;
  templateId: string;
}

  // eslint-disable-next-line max-lines-per-function
export function TemplateItemDrawer({ isOpen, onClose, editingItem, templateId }: TemplateItemDrawerProps) {
  const toast = useToast();
  const createMutation = useCreateLifecycleTemplateItem(templateId);
  const updateMutation = useUpdateLifecycleTemplateItem(templateId);
  const { members } = useWorkspaceMembers();

  interface ItemFormState {
    assigneeType: ChecklistAssigneeType;
    assigneeUserId: string;
    category: ChecklistCategory;
    description: string;
    displayOrder: string;
    dueDaysOffset: string;
    isRequired: boolean;
    title: string;
  }

  const EMPTY_ITEM_FORM: ItemFormState = {
    title: "",
    description: "",
    category: "hr_admin",
    assigneeType: "hr_manager",
    assigneeUserId: "",
    isRequired: true,
    dueDaysOffset: "0",
    displayOrder: "0",
  };

  // Compute external form values — stabilized via useMemo.
  const externalForm = useMemo((): ItemFormState => {
    if (editingItem) {
      return {
        title: editingItem.title,
        description: editingItem.description ?? "",
        category: editingItem.category,
        assigneeType: editingItem.assignee_type,
        assigneeUserId: editingItem.assignee_user_id ?? "",
        isRequired: editingItem.is_required,
        dueDaysOffset: String(editingItem.due_days_offset),
        displayOrder: String(editingItem.display_order),
      };
    }
    return EMPTY_ITEM_FORM;
  }, [editingItem]);

  const [form, setForm] = useDerivedStateFrom(externalForm);

  const handleSave = () => {
    if (!form.title.trim()) {
      toast.error("Checklist item title is required");
      return;
    }

    const payload = {
      title: form.title,
      description: form.description.trim() ? form.description : undefined,
      category: form.category,
      assignee_type: form.assigneeType,
      assignee_user_id: form.assigneeType === "specific_user" && form.assigneeUserId ? form.assigneeUserId : undefined,
      is_required: form.isRequired,
      due_days_offset: parseInt(form.dueDaysOffset) || 0,
      display_order: parseInt(form.displayOrder) || 0,
    };

    if (editingItem) {
      updateMutation.mutate(
        {
          id: editingItem.id,
          payload,
        },
        {
          onSuccess: () => {
            toast.success("Checklist item updated");
            onClose();
          },
          onError: (err) => {
            toast.error(err instanceof Error ? err.message : "Failed to update item");
          },
        }
      );
    } else {
      createMutation.mutate(payload, {
        onSuccess: () => {
          toast.success("Checklist item added");
          onClose();
        },
        onError: (err) => {
          toast.error(err instanceof Error ? err.message : "Failed to add item");
        },
      });
    }
  };

  const memberOptions = members.map((m) => ({
    value: m.user_id,
    label: m.full_name ?? m.email,
  }));

  return (
    <Sheet
      isOpen={isOpen}
      onClose={onClose}
      title={editingItem ? "Edit Checklist Item" : "Add Checklist Item"}
    >
      <div className="space-y-6 pt-4 max-h-[80vh] overflow-y-auto pr-2 custom-scrollbar">
        <FormField
          label="Item Title"
          onChange={(e) => { setForm(prev => ({ ...prev, title: e.target.value })); }}
          placeholder="e.g., Assign laptop and corporate email"
          value={form.title}
        />
        <FormField
          label="Description / Instructions"
          onChange={(e) => { setForm(prev => ({ ...prev, description: e.target.value })); }}
          placeholder="Add detailed instructions for completion"
          type="textarea"
          value={form.description}
        />

        <div className="grid grid-cols-2 gap-4">
          <div className="space-y-2">
            <label className="text-xs font-bold text-slate-500 uppercase tracking-wider block" htmlFor="item-category">
              Category
            </label>
            <select
              className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-xl px-4 py-2.5 text-sm"
              id="item-category"
              onChange={(e) => { setForm(prev => ({ ...prev, category: e.target.value as ChecklistCategory })); }}
              value={form.category}
            >
              <option value="hr_admin">HR Admin</option>
              <option value="it_setup">IT Setup</option>
              <option value="documentation">Documentation</option>
              <option value="training">Training</option>
              <option value="equipment">Equipment</option>
            </select>
          </div>

          <div className="space-y-2">
            <label className="text-xs font-bold text-slate-500 uppercase tracking-wider block" htmlFor="item-assignee-type">
              Assignee Type
            </label>
            <select
              className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-xl px-4 py-2.5 text-sm"
              id="item-assignee-type"
              onChange={(e) => { setForm(prev => ({ ...prev, assigneeType: e.target.value as ChecklistAssigneeType })); }}
              value={form.assigneeType}
            >
              <option value="hr_manager">HR Manager</option>
              <option value="direct_manager">Direct Manager</option>
              <option value="it_admin">IT Admin</option>
              <option value="employee">Self (Employee)</option>
              <option value="specific_user">Specific Workspace User</option>
            </select>
          </div>
        </div>

        {form.assigneeType === "specific_user" && (
          <div className="space-y-2">
            <label className="text-xs font-bold text-slate-500 uppercase tracking-wider block" htmlFor="item-assignee">
              Select Specific Assignee
            </label>
            <Combobox
              onChange={(v) => { setForm(prev => ({ ...prev, assigneeUserId: v })); }}
              options={memberOptions}
              placeholder="Search user..."
              value={form.assigneeUserId}
            />
          </div>
        )}

        <div className="grid grid-cols-2 gap-4">
          <FormField
            label="Due Days Offset"
            onChange={(e) => { setForm(prev => ({ ...prev, dueDaysOffset: e.target.value })); }}
            placeholder="e.g., -5 (5 days before) or 2 (2 days after)"
            type="number"
            value={form.dueDaysOffset}
          />
          <FormField
            label="Display Order"
            onChange={(e) => { setForm(prev => ({ ...prev, displayOrder: e.target.value })); }}
            placeholder="Order priority"
            type="number"
            value={form.displayOrder}
          />
        </div>

        <div className="flex items-center gap-3 bg-slate-50 dark:bg-slate-800/30 p-4 rounded-2xl border border-slate-100 dark:border-slate-800">
          <input
            checked={form.isRequired}
            className="w-4 h-4 text-primary-600 focus:ring-primary-500 border-slate-300 rounded"
            id="isRequired"
            onChange={(e) => { setForm(prev => ({ ...prev, isRequired: e.target.checked })); }}
            type="checkbox"
          />
          <label className="text-sm font-semibold text-slate-700 dark:text-slate-300 cursor-pointer" htmlFor="isRequired">
            Mark this task item as required to complete the checklist
          </label>
        </div>

        <div className="flex justify-end gap-3 pt-6 border-t border-slate-100 dark:border-slate-800">
          <Button onClick={onClose} variant="ghost">
            CANCEL
          </Button>
          <Button onClick={handleSave}>
            SAVE ITEM
          </Button>
        </div>
      </div>
    </Sheet>
  );
}
