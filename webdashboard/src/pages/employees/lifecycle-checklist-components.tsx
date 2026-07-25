import { CheckCircle2, Circle, AlertCircle, FileText } from "lucide-react";

import type { LifecycleChecklistItem } from "@/lib/api/types";

import Button from "@/components/ui/button";
import FormField from "@/components/ui/form-field";
import { Typography } from "@/components/ui/typography";
import { cn } from "@/lib/utils";

// eslint-disable-next-line react-refresh/only-export-components
export function formatChecklistType(raw: string): string {
  return raw.replace(/_/g, " ").toUpperCase();
}

/** Checklist progress bar + status badge. */
export function ChecklistHeader({ checklist }: { checklist: { checklist_type: string; progress_percent: number; started_at: string; status: string } }) {
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

/** Notes display + editing section for a checklist item. */
function ItemNotesSection({
  item,
  isEditingNotes,
  itemNotes,
  onSaveNotes,
  onCancelEdit,
  onNotesChange,
}: {
  item: LifecycleChecklistItem;
  isEditingNotes: boolean;
  itemNotes: string;
  onSaveNotes: (item: LifecycleChecklistItem) => void;
  onCancelEdit: () => void;
  onNotesChange: (value: string) => void;
}) {
  if (isEditingNotes) {
    return (
      <ItemNotesEditor
        item={item}
        itemNotes={itemNotes}
        onCancelEdit={onCancelEdit}
        onNotesChange={onNotesChange}
        onSaveNotes={onSaveNotes}
      />
    );
  }
  if (item.notes) {
    return (
      <div className="bg-slate-50 dark:bg-slate-800/30 p-2.5 rounded-xl border border-slate-100 dark:border-slate-800 flex items-start gap-2 mt-2">
        <FileText className="w-3.5 h-3.5 text-slate-400 mt-0.5" />
        <div className="text-xs text-slate-500 dark:text-slate-400 italic">
          &ldquo;{item.notes}&rdquo;
        </div>
      </div>
    );
  }
  return null;
}

function ItemNotesEditor({
  item,
  itemNotes,
  onNotesChange,
  onSaveNotes,
  onCancelEdit,
}: {
  item: LifecycleChecklistItem;
  itemNotes: string;
  onNotesChange: (value: string) => void;
  onSaveNotes: (item: LifecycleChecklistItem) => void;
  onCancelEdit: () => void;
}) {
  return (
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
  );
}

/** A single checklist item row with toggle, notes, edit. */
export function ChecklistItemRow({
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
  const rowBg = isCompleted
    ? "bg-slate-50/20 dark:bg-slate-900/40 border-slate-100 dark:border-slate-800/50 opacity-80"
    : "bg-white dark:bg-slate-950 border-slate-100 dark:border-slate-800 hover:border-slate-200 hover:shadow-sm";
  const titleCls = isCompleted
    ? "text-slate-400 line-through"
    : "text-slate-800 dark:text-slate-200";

  return (
    <div className={cn("p-4 rounded-2xl border transition-all flex flex-col gap-3", rowBg)}>
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
            <span className={cn("font-semibold text-sm", titleCls)}>
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

          {/* Item Notes */}
          <ItemNotesSection
            isEditingNotes={isEditingNotes}
            item={item}
            itemNotes={itemNotes}
            onCancelEdit={onCancelEdit}
            onNotesChange={onNotesChange}
            onSaveNotes={onSaveNotes}
          />
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
