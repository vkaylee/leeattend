import { ClipboardList, Plus, CheckCircle2, Circle, AlertCircle, FileText, User } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import EmptyState from "@/components/ui/empty-state";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import FormField from "@/components/ui/form-field";
import {
  useLifecycleChecklists,
  useCreateLifecycleChecklist,
  useUpdateLifecycleChecklistItem,
  useLifecycleTemplates,
} from "@/hooks/use-lifecycle";
import type { LifecycleChecklistResponse, LifecycleChecklistItem, ChecklistItemStatus } from "@/lib/api/types";
import { cn } from "@/lib/utils";

interface EmployeeLifecycleTabProps {
  employmentId?: string;
  wsId: string;
}

export default function EmployeeLifecycleTab({ employmentId, wsId }: EmployeeLifecycleTabProps) {
  const { t } = useTranslation();
  const toast = useToast();

  const [selectedTemplateId, setSelectedTemplateId] = useState("");
  const [activeChecklistId, setActiveChecklistId] = useState<string | null>(null);
  const [editingNotesItemId, setEditingNotesItemId] = useState<string | null>(null);
  const [itemNotes, setItemNotes] = useState("");

  // Queries
  const { data: checklists = [], isLoading } = useLifecycleChecklists(employmentId ?? "");
  const { data: templates = [] } = useLifecycleTemplates();

  // Mutations
  const createChecklistMutation = useCreateLifecycleChecklist(employmentId ?? "");
  const updateItemStatusMutation = useUpdateLifecycleChecklistItem(activeChecklistId ?? "", employmentId ?? "");

  // Active checklist details
  const activeChecklist = checklists.find((c) => c.id === activeChecklistId) || checklists[0];

  // Sync active checklist id
  React.useEffect(() => {
    if (checklists.length > 0 && !activeChecklistId) {
      setActiveChecklistId(checklists[0].id);
    }
  }, [checklists, activeChecklistId]);

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
          setActiveChecklistId(res.id);
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

  // Filter templates that don't already have instantiated checklists of their type
  // (unless admin manually wants to re-add)
  const availableTemplates = templates.filter(t => t.is_active);

  if (isLoading) {
    return <div className="bg-slate-50 dark:bg-slate-900 border border-slate-100 dark:border-slate-800 p-8 rounded-[2.5rem] shadow-sm animate-pulse h-96" />;
  }

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
            value={selectedTemplateId}
            onChange={(e) => setSelectedTemplateId(e.target.value)}
          >
            <option value="">Select a template to assign...</option>
            {availableTemplates.map((temp) => (
              <option key={temp.id} value={temp.id}>
                {temp.name} ({temp.checklist_type.replace("_", " ")})
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
                  key={c.id}
                  className={cn(
                    "px-4 py-2 rounded-xl text-xs font-bold transition-all",
                    activeChecklistId === c.id
                      ? "bg-white dark:bg-slate-950 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800"
                      : "text-slate-500 hover:text-slate-700 dark:hover:text-slate-300"
                  )}
                  onClick={() => setActiveChecklistId(c.id)}
                  variant="ghost"
                >
                  {c.checklist_type.toUpperCase()} ({c.progress_percent}%)
                </Button>
              ))}
            </div>
          )}

          {activeChecklist && (
            <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-[2.5rem] shadow-sm p-6 space-y-6">
              {/* Header Info */}
              <div className="flex flex-col md:flex-row md:items-center justify-between gap-4 border-b border-slate-50 dark:border-slate-800/60 pb-6">
                <div className="space-y-1">
                  <Typography className="text-xl font-bold text-slate-800 dark:text-slate-200" variant="h3">
                    {activeChecklist.checklist_type.replace("_", " ").toUpperCase()} CHECKLIST
                  </Typography>
                  <Typography className="text-xs text-slate-400" variant="p">
                    Started on {new Date(activeChecklist.started_at).toLocaleDateString()}
                  </Typography>
                </div>

                {/* Progress bar */}
                <div className="flex items-center gap-3">
                  <div className="w-32 bg-slate-100 dark:bg-slate-800 h-2.5 rounded-full overflow-hidden">
                    <div
                      className="bg-emerald-500 h-full rounded-full transition-all duration-500"
                      style={{ width: `${activeChecklist.progress_percent}%` }}
                    />
                  </div>
                  <span className="font-extrabold text-sm text-slate-700 dark:text-slate-300">
                    {activeChecklist.progress_percent}%
                  </span>
                  <span className={cn(
                    "text-[10px] font-bold uppercase tracking-wider px-2 py-0.5 rounded-full",
                    activeChecklist.status === "completed"
                      ? "bg-emerald-50 text-emerald-600 border border-emerald-200"
                      : "bg-blue-50 text-blue-600 border border-blue-200"
                  )}>
                    {activeChecklist.status.replace("_", " ")}
                  </span>
                </div>
              </div>

              {/* Items List */}
              <div className="space-y-4">
                {activeChecklist.items
                  ?.sort((a: any, b: any) => a.display_order - b.display_order)
                  .map((item: any) => {
                    const isCompleted = item.status === "completed";
                    const isEditingNotes = editingNotesItemId === item.id;

                    return (
                      <div
                        key={item.id}
                        className={cn(
                          "p-4 rounded-2xl border transition-all flex flex-col gap-3",
                          isCompleted
                            ? "bg-slate-50/20 dark:bg-slate-900/40 border-slate-100 dark:border-slate-800/50 opacity-80"
                            : "bg-white dark:bg-slate-950 border-slate-100 dark:border-slate-800 hover:border-slate-200 hover:shadow-sm"
                        )}
                      >
                        <div className="flex items-start gap-4 justify-between">
                          <button
                            className="text-slate-400 hover:text-primary-600 transition-colors pt-0.5"
                            onClick={() => handleToggleItemStatus(item)}
                          >
                            {isCompleted ? (
                              <CheckCircle2 className="w-5 h-5 text-emerald-500" />
                            ) : (
                              <Circle className="w-5 h-5" />
                            )}
                          </button>

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
                                {item.category.replace("_", " ")}
                              </span>
                              {item.is_required && (
                                <span className="bg-rose-50 dark:bg-rose-950/20 text-rose-600 dark:text-rose-400 text-[9px] px-2 py-0.5 rounded-full font-bold uppercase tracking-wider">
                                  Required
                                </span>
                              )}
                            </div>

                            {item.description && (
                              <Typography className="text-xs text-slate-400 dark:text-slate-500" variant="p">
                                {item.description}
                              </Typography>
                            )}

                            {item.due_date && (
                              <div className="text-[10px] text-slate-400 font-semibold flex items-center gap-1">
                                <AlertCircle className="w-3.5 h-3.5 text-amber-500" />
                                <span>Due: {new Date(item.due_date).toLocaleDateString()}</span>
                              </div>
                            )}

                            {/* Item Notes Display / Edit */}
                            {item.notes && !isEditingNotes && (
                              <div className="bg-slate-50 dark:bg-slate-800/30 p-2.5 rounded-xl border border-slate-100 dark:border-slate-800 flex items-start gap-2 mt-2">
                                <FileText className="w-3.5 h-3.5 text-slate-400 mt-0.5" />
                                <div className="text-xs text-slate-500 dark:text-slate-400 italic">
                                  "{item.notes}"
                                </div>
                              </div>
                            )}

                            {isEditingNotes && (
                              <div className="space-y-2 mt-2">
                                <FormField
                                  label="Notes"
                                  type="textarea"
                                  placeholder="Add notes for this checklist item..."
                                  value={itemNotes}
                                  onChange={(e) => setItemNotes(e.target.value)}
                                />
                                <div className="flex gap-2 justify-end">
                                  <Button
                                    className="text-[10px] py-1 px-3 h-auto"
                                    onClick={() => setEditingNotesItemId(null)}
                                    variant="ghost"
                                  >
                                    CANCEL
                                  </Button>
                                  <Button
                                    className="text-[10px] py-1 px-3 h-auto"
                                    onClick={() => handleSaveNotes(item)}
                                  >
                                    SAVE NOTES
                                  </Button>
                                </div>
                              </div>
                            )}
                          </div>

                          {!isEditingNotes && (
                            <Button
                              className="text-[10px] py-1 px-2.5 h-auto text-slate-400 hover:text-slate-600 hover:bg-slate-50 dark:hover:bg-slate-800"
                              onClick={() => handleEditNotes(item)}
                              variant="ghost"
                            >
                              NOTES
                            </Button>
                          )}
                        </div>
                      </div>
                    );
                  })}
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
