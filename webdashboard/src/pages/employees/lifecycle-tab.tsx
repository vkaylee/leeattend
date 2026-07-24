import { ClipboardList, Plus, CheckCircle2, Circle, AlertCircle, FileText } from "lucide-react";
import { useState } from "react";

import type { LifecycleChecklistItem, ChecklistItemStatus } from "@/lib/api/types";

import Button from "@/components/ui/button";
import EmptyState from "@/components/ui/empty-state";
import FormField from "@/components/ui/form-field";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import {
  useLifecycleChecklists,
  useCreateLifecycleChecklist,
  useUpdateLifecycleChecklistItem,
  useLifecycleTemplates,
} from "@/hooks/use-lifecycle";
import { cn } from "@/lib/utils";

function formatChecklistType(raw: string): string {
  return raw.replace(/_/g, " ").toUpperCase();
}

interface EmployeeLifecycleTabProps {
  employmentId?: string;
  wsId: string;
}

/** Checklist progress bar + status badge. */
function ChecklistHeader({ checklist }: { checklist: { checklist_type: string; progress_percent: number; started_at: string; status: string } }) {
  return (
    <div className="flex flex-col md:flex-row md:items-center justify-between gap-4 border-b border-slate-50 dark:border-slate-800/60 pb-6">
      <div className="space-y-1">
        <Typography className="text-xl font-bold text-slate-800 dark:text-slate-200" variant="h3">
          {formatChecklistType(checklist.checklist_type)} CHECKLIST
        </Typography>
        <Typography className="text-xs text-slate-400" variant="p">
          Started on {new Date(checklist.started_at).toLocaleDateString()}
        </Typography>
      </div>

      <div className="flex items-center gap-3">
        <div className="w-32 bg-slate-100 dark:bg-slate-800 h-2.5 rounded-full overflow-hidden">
          <div
            className="bg-emerald-500 h-full rounded-full transition-all duration-500"
            style={{ width: `${checklist.progress_percent}%` }}
          />
        </div>
        <span className="font-extrabold text-sm text-slate-700 dark:text-slate-300">
          {checklist.progress_percent}%
        </span>
        <span className={cn(
          "text-[10px] font-bold uppercase tracking-wider px-2 py-0.5 rounded-full",
          checklist.status === "completed"
            ? "bg-emerald-50 text-emerald-600 border border-emerald-200"
            : "bg-blue-50 text-blue-600 border border-blue-200"
        )}>
          {checklist.status.replace(/_/g, " ")}
        </span>
      </div>
    </div>
  );
}

/** A single checklist item row with toggle, notes, edit. */
function ChecklistItemRow({
  item,
  isEditingNotes,
  itemNotes,
  editingNotesItemId: _editingNotesItemId,
  onToggle,
  onEditNotes,
  onSaveNotes,
  onCancelEdit,
  onNotesChange,
}: {
  item: LifecycleChecklistItem;
  isEditingNotes: boolean;
  itemNotes: string;
  editingNotesItemId: string | null;
  onToggle: (item: LifecycleChecklistItem) => void;
  onEditNotes: (item: LifecycleChecklistItem) => void;
  onSaveNotes: (item: LifecycleChecklistItem) => void;
  onCancelEdit: () => void;
  onNotesChange: (value: string) => void;
}) {
  const isCompleted = item.status === "completed";

  return (
    <div
      className={cn(
        "p-4 rounded-2xl border transition-all flex flex-col gap-3",
        isCompleted
          ? "bg-slate-50/20 dark:bg-slate-900/40 border-slate-100 dark:border-slate-800/50 opacity-80"
          : "bg-white dark:bg-slate-950 border-slate-100 dark:border-slate-800 hover:border-slate-200 hover:shadow-sm"
      )}
    >
      <div className="flex items-start gap-4 justify-between">
        <Button
          className="text-slate-400 hover:text-primary-600 transition-colors pt-0.5 h-auto p-0"
          onClick={() => { onToggle(item); }}
          variant="ghost"
        >
          {isCompleted ? (
            <CheckCircle2 className="w-5 h-5 text-emerald-500" />
          ) : (
            <Circle className="w-5 h-5" />
          )}
        </Button>

        <div className="flex-1 space-y-1">
          <div className="flex items-center gap-2 flex-wrap">
            <span className={cn(
              "font-semibold text-sm",
              isCompleted
                ? "text-slate-400 line-through"
                : "text-slate-800 dark:text-slate-200"
            )}>
              {item.title}
            </span>
            <span className="bg-slate-100 dark:bg-slate-800 text-slate-500 dark:text-slate-400 text-[9px] px-2 py-0.5 rounded-full font-bold uppercase tracking-wider">
              {formatChecklistType(item.category)}
            </span>
            {item.is_required ? (
              <span className="bg-rose-50 dark:bg-rose-950/20 text-rose-600 dark:text-rose-400 text-[9px] px-2 py-0.5 rounded-full font-bold uppercase tracking-wider">
                Required
              </span>
            ) : null}
          </div>

          {!!item.description && (
            <Typography className="text-xs text-slate-400 dark:text-slate-500" variant="p">
              {item.description}
            </Typography>
          )}

          {!!item.due_date && (
            <div className="text-[10px] text-slate-400 font-semibold flex items-center gap-1">
              <AlertCircle className="w-3.5 h-3.5 text-amber-500" />
              <span>Due: {new Date(item.due_date).toLocaleDateString()}</span>
            </div>
          )}

          {/* Item Notes Display */}
          {!!item.notes && !isEditingNotes && (
            <div className="bg-slate-50 dark:bg-slate-800/30 p-2.5 rounded-xl border border-slate-100 dark:border-slate-800 flex items-start gap-2 mt-2">
              <FileText className="w-3.5 h-3.5 text-slate-400 mt-0.5" />
              <div className="text-xs text-slate-500 dark:text-slate-400 italic">
                &ldquo;{item.notes}&rdquo;
              </div>
            </div>
          )}

          {isEditingNotes ? (
            <div className="space-y-2 mt-2">
              <FormField
                label="Notes"
                onChange={(e) => { onNotesChange(e.target.value); }}
                placeholder="Add notes for this checklist item..."
                type="textarea"
                value={itemNotes}
              />
              <div className="flex gap-2 justify-end">
                <Button
                  className="text-[10px] py-1 px-3 h-auto"
                  onClick={onCancelEdit}
                  variant="ghost"
                >
                  CANCEL
                </Button>
                <Button
                  className="text-[10px] py-1 px-3 h-auto"
                  onClick={() => { onSaveNotes(item); }}
                >
                  SAVE NOTES
                </Button>
              </div>
            </div>
          ) : null}
        </div>

        {!isEditingNotes && (
          <Button
            className="text-[10px] py-1 px-2.5 h-auto text-slate-400 hover:text-slate-600 hover:bg-slate-50 dark:hover:bg-slate-800"
            onClick={() => { onEditNotes(item); }}
            variant="ghost"
          >
            NOTES
          </Button>
        )}
      </div>
    </div>
  );
}

export default function EmployeeLifecycleTab({ employmentId }: EmployeeLifecycleTabProps) {
  const toast = useToast();

  const [selectedTemplateId, setSelectedTemplateId] = useState("");
  const [userSelectedChecklistId, setUserSelectedChecklistId] = useState<string | null>(null);
  const [editingNotesItemId, setEditingNotesItemId] = useState<string | null>(null);
  const [itemNotes, setItemNotes] = useState("");

  // Queries
  const { data: checklists = [], isLoading } = useLifecycleChecklists(employmentId ?? "");
  const { data: templates = [] } = useLifecycleTemplates();

  // Mutations
  const createChecklistMutation = useCreateLifecycleChecklist(employmentId ?? "");
  // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition -- checklist/employment may be null during loading
  const activeChecklistId = userSelectedChecklistId ?? checklists[0]?.id ?? null;
  // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition -- checklist/employment may be null during loading
  const updateItemStatusMutation = useUpdateLifecycleChecklistItem(activeChecklistId ?? "", employmentId ?? "");

  // Active checklist details
  const activeChecklist = checklists.find((c) => c.id === activeChecklistId) ?? checklists[0];

  if (!employmentId) {
    return (
      <div className="p-6 text-center text-slate-400">
        No active employment profile to configure checklists.
      </div>
    );
  }

  const handleCreateChecklist = () => {
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
  };

  const handleToggleItemStatus = (item: LifecycleChecklistItem) => {
    const newStatus: ChecklistItemStatus = item.status === "completed" ? "pending" : "completed";

    updateItemStatusMutation.mutate(
      {
        itemId: item.id,
        payload: {
          status: newStatus,
          notes: item.notes ?? undefined,
        },
      },
      {
        onSuccess: () => {
          toast.success(`Task marked as ${newStatus}`);
        },
        onError: (err) => {
          toast.error(err instanceof Error ? err.message : "Failed to update item status");
        },
      }
    );
  };

  const handleEditNotes = (item: LifecycleChecklistItem) => {
    setEditingNotesItemId(item.id);
    setItemNotes(item.notes ?? "");
  };

  const handleSaveNotes = (item: LifecycleChecklistItem) => {
    updateItemStatusMutation.mutate(
      {
        itemId: item.id,
        payload: {
          status: item.status,
          notes: itemNotes.trim() ? itemNotes : null,
        },
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
  };

  // Filter templates that are active
  const availableTemplates = templates.filter(t => t.is_active);

  if (isLoading) {
    return (
      <div className="bg-slate-50 dark:bg-slate-900 border border-slate-100 dark:border-slate-800 p-6 sm:p-8 rounded-2xl sm:rounded-[2.5rem] shadow-sm animate-pulse h-96" />
    );
  }

  // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition -- activeChecklist may be null before data loads despite isLoading guard
  const sortedItems = activeChecklist?.items
    ? [...activeChecklist.items].sort((a: LifecycleChecklistItem, b: LifecycleChecklistItem) => a.display_order - b.display_order)
    : [];

  return (
    <div className="space-y-8 animate-in fade-in duration-500">
      {/* Instantiate Section */}
      <div className="bg-slate-50 dark:bg-slate-800/40 p-6 rounded-3xl border border-slate-100 dark:border-slate-800 flex flex-col md:flex-row md:items-end gap-4 justify-between">
        <div className="flex-1 space-y-2">
          <Typography className="text-sm font-bold text-slate-700 dark:text-slate-300" variant="h4">
            Assign Checklist Template
          </Typography>
          <select
            className="w-full max-w-md bg-white dark:bg-slate-950 border border-slate-200 dark:border-slate-800 rounded-xl px-4 py-2.5 text-sm"
            onChange={(e) => { setSelectedTemplateId(e.target.value); }}
            value={selectedTemplateId}
          >
            <option value="">Select a template to assign...</option>
            {availableTemplates.map((temp) => (
              <option key={temp.id} value={temp.id}>
                {temp.name} ({formatChecklistType(temp.checklist_type)})
              </option>
            ))}
          </select>
        </div>
        <Button
          icon={<Plus size={16} />}
          onClick={handleCreateChecklist}
        >
          ASSIGN TEMPLATE
        </Button>
      </div>

      {checklists.length === 0 ? (
        <EmptyState
          description="This employee does not have any active lifecycle checklists assigned."
          icon={<ClipboardList size={40} />}
          title="No Checklists Assigned"
        />
      ) : (
        <div className="space-y-6">
          {/* Checklist Tab selector if multiple checklists exist */}
          {checklists.length > 1 && (
            <div className="flex bg-slate-100 dark:bg-slate-800/50 p-1 rounded-2xl w-fit border border-slate-200 dark:border-slate-700/50 shadow-inner">
              {checklists.map((c) => (
                <Button
                  className={cn(
                    "px-4 py-2 rounded-xl text-xs font-bold transition-all",
                    activeChecklistId === c.id
                      ? "bg-white dark:bg-slate-950 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800"
                      : "text-slate-500 hover:text-slate-700 dark:hover:text-slate-300"
                  )}
                  key={c.id}
                  onClick={() => { setUserSelectedChecklistId(c.id); }}
                  variant="ghost"
                >
                  {formatChecklistType(c.checklist_type)} ({c.progress_percent}%)
                </Button>
              ))}
            </div>
          )}

          {/* eslint-disable-next-line @typescript-eslint/no-unnecessary-condition -- activeChecklist may be null before data loads */}
          {!!activeChecklist && (
            <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-2xl sm:rounded-[2.5rem] shadow-sm p-5 sm:p-6 space-y-6">
              <ChecklistHeader checklist={activeChecklist} />

              {/* Items List */}
              <div className="space-y-4">
                {sortedItems.map((item: LifecycleChecklistItem) => (
                  <ChecklistItemRow
                    editingNotesItemId={editingNotesItemId}
                    isEditingNotes={editingNotesItemId === item.id}
                    item={item}
                    itemNotes={itemNotes}
                    key={item.id}
                    onCancelEdit={() => { setEditingNotesItemId(null); }}
                    onEditNotes={handleEditNotes}
                    onNotesChange={setItemNotes}
                    onSaveNotes={handleSaveNotes}
                    onToggle={handleToggleItemStatus}
                  />
                ))}
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
