import { ClipboardList, Plus } from "lucide-react";

import type { LifecycleChecklistItem } from "@/lib/api/types";

import { maybeNull } from "@/types/api-overrides";
import Button from "@/components/ui/button";
import EmptyState from "@/components/ui/empty-state";
import { Typography } from "@/components/ui/typography";
import { cn } from "@/lib/utils";

import { useLifecycleChecklistState } from "./hooks/use-lifecycle-checklist-state";
import { formatChecklistType, ChecklistHeader, ChecklistItemRow } from "./lifecycle-checklist-components";

interface EmployeeLifecycleTabProps {
  employmentId?: string;
  wsId: string;
}

export default function EmployeeLifecycleTab({ employmentId }: EmployeeLifecycleTabProps) {
  const state = useLifecycleChecklistState(employmentId ?? "");

  if (!employmentId) {
    return (
      <div className="p-6 text-center text-slate-400">
        No active employment profile to configure checklists.
      </div>
    );
  }

  if (state.isLoading) {
    return (
      <div className="bg-slate-50 dark:bg-slate-900 border border-slate-100 dark:border-slate-800 p-6 sm:p-8 rounded-2xl sm:rounded-[2.5rem] shadow-sm animate-pulse h-96" />
    );
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
            onChange={(e) => { state.setSelectedTemplateId(e.target.value); }}
            value={state.selectedTemplateId}
          >
            <option value="">Select a template to assign...</option>
            {state.availableTemplates.map((temp) => (
              <option key={temp.id} value={temp.id}>
                {temp.name} ({formatChecklistType(temp.checklist_type)})
              </option>
            ))}
          </select>
        </div>
        <Button
          icon={<Plus size={16} />}
          onClick={state.handleCreateChecklist}
        >
          ASSIGN TEMPLATE
        </Button>
      </div>

      {state.checklists.length === 0 ? (
        <EmptyState
          description="This employee does not have any active lifecycle checklists assigned."
          icon={<ClipboardList size={40} />}
          title="No Checklists Assigned"
        />
      ) : (
        <div className="space-y-6">
          {/* Checklist Tab selector if multiple checklists exist */}
          {state.checklists.length > 1 && (
            <div className="flex bg-slate-100 dark:bg-slate-800/50 p-1 rounded-2xl w-fit border border-slate-200 dark:border-slate-700/50 shadow-inner">
              {state.checklists.map((c) => (
                <Button
                  className={cn(
                    "px-4 py-2 rounded-xl text-xs font-bold transition-all",
                    state.activeChecklistId === c.id
                      ? "bg-white dark:bg-slate-950 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800"
                      : "text-slate-500 hover:text-slate-700 dark:hover:text-slate-300"
                  )}
                  key={c.id}
                  onClick={() => { state.setUserSelectedChecklistId(c.id); }}
                  variant="ghost"
                >
                  {formatChecklistType(c.checklist_type)} ({c.progress_percent}%)
                </Button>
              ))}
            </div>
          )}

          {/* activeChecklist may be null before data loads */}
          {!!maybeNull(state.activeChecklist) && (
            <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-2xl sm:rounded-[2.5rem] shadow-sm p-5 sm:p-6 space-y-6">
              <ChecklistHeader checklist={state.activeChecklist} />

              {/* Items List */}
              <div className="space-y-4">
                {state.sortedItems.map((item: LifecycleChecklistItem) => (
                  <ChecklistItemRow
                    editingNotesItemId={state.editingNotesItemId}
                    isEditingNotes={state.editingNotesItemId === item.id}
                    item={item}
                    itemNotes={state.itemNotes}
                    key={item.id}
                    onCancelEdit={() => { state.setEditingNotesItemId(null); }}
                    onEditNotes={state.handleEditNotes}
                    onNotesChange={state.setItemNotes}
                    onSaveNotes={state.handleSaveNotes}
                    onToggle={state.handleToggleItemStatus}
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
