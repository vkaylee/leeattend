import { Plus, Pencil, Trash2 } from "lucide-react";
import React from "react";

import type {
  LifecycleChecklistTemplate,
  LifecycleChecklistTemplateItem,
} from "@/lib/api/types";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";

interface ChecklistItemsPanelProps {
  activeTemplate: LifecycleChecklistTemplate | undefined;
  handleCreateItem: () => void;
  handleDeleteItem: (item: LifecycleChecklistTemplateItem) => void;
  handleEditItem: (item: LifecycleChecklistTemplateItem) => void;
  isItemsLoading: boolean;
  items: LifecycleChecklistTemplateItem[];
}

export function ChecklistItemsPanel({
  activeTemplate,
  handleCreateItem,
  handleDeleteItem,
  handleEditItem,
  isItemsLoading,
  items,
}: ChecklistItemsPanelProps) {
  return (
    <div className="lg:col-span-2 space-y-4">
      <div className="flex items-center justify-between px-2">
        <Typography className="text-xs font-extrabold text-slate-400 uppercase tracking-widest" variant="h3">
          Checklist Items
        </Typography>
        {!!activeTemplate && (
          <Button
            icon={<Plus size={16} />}
            onClick={handleCreateItem}
            variant="ghost"
          >
            ADD ITEM
          </Button>
        )}
      </div>

      {isItemsLoading ? (
        <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 p-6 sm:p-8 rounded-2xl sm:rounded-[2.5rem] shadow-sm animate-pulse h-80" />
      ) : !activeTemplate ? (
        <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 p-8 sm:p-12 rounded-2xl sm:rounded-[2.5rem] text-center text-slate-400">
          Select a template to view and manage its checklist items.
        </div>
      ) : items.length === 0 ? (
        <div className="bg-white dark:bg-slate-900 rounded-2xl sm:rounded-[2.5rem] border border-slate-100 dark:border-slate-800 p-8 sm:p-16 text-center space-y-4">
          <Typography className="text-slate-500 font-semibold" variant="h4">
            No items in this checklist template.
          </Typography>
          <Typography className="text-slate-400 text-xs max-w-sm mx-auto" variant="p">
            Create task items for this template to automatically assign them to staff during employee onboarding/offboarding.
          </Typography>
          <Button
            icon={<Plus size={16} />}
            onClick={handleCreateItem}
            variant="ghost"
          >
            CREATE FIRST ITEM
          </Button>
        </div>
      ) : (
        <div className="bg-white dark:bg-slate-900 rounded-2xl sm:rounded-[2.5rem] border border-slate-100 dark:border-slate-800 overflow-hidden shadow-sm p-4 space-y-3">
          {items
            .sort((a, b) => a.display_order - b.display_order)
            .map((item) => (
              <div
                className="p-4 rounded-2xl border border-slate-50 dark:border-slate-800/60 bg-slate-50/30 dark:bg-slate-900 hover:border-slate-200 dark:hover:border-slate-700 transition-all flex flex-col md:flex-row md:items-center justify-between gap-4"
                key={item.id}
              >
                <div className="flex-1 space-y-1">
                  <div className="flex items-center gap-2 flex-wrap">
                    <span className="font-semibold text-slate-800 dark:text-slate-200 text-sm">
                      {item.title}
                    </span>
                    <span className="bg-slate-100 dark:bg-slate-800 text-slate-600 dark:text-slate-400 text-[10px] px-2 py-0.5 rounded-full font-bold uppercase tracking-wider">
                      {item.category.replace("_", " ")}
                    </span>
                    <span className="bg-blue-50 dark:bg-blue-950/20 text-blue-600 dark:text-blue-400 text-[10px] px-2 py-0.5 rounded-full font-bold uppercase tracking-wider">
                      Assignee: {item.assignee_type.replace("_", " ")}
                    </span>
                    {item.is_required ? (
                      <span className="bg-rose-50 dark:bg-rose-950/20 text-rose-600 dark:text-rose-400 text-[10px] px-2 py-0.5 rounded-full font-bold uppercase tracking-wider">
                        Required
                      </span>
                    ) : null}
                  </div>
                  {!!item.description && (
                    <Typography className="text-xs text-slate-400 dark:text-slate-500" variant="p">
                      {item.description}
                    </Typography>
                  )}
                  <div className="text-[10px] text-slate-400 dark:text-slate-500 font-medium">
                    Due offset: <span className="font-bold">{item.due_days_offset} days</span>
                  </div>
                </div>

                <div className="flex items-center gap-1 justify-end">
                  <Button
                    icon={<Pencil size={14} />}
                    onClick={() => { handleEditItem(item); }}
                    variant="ghost"
                  />
                  <Button
                    className="text-rose-500 hover:text-rose-600 hover:bg-rose-50 dark:hover:bg-rose-950/20"
                    icon={<Trash2 size={14} />}
                    onClick={() => handleDeleteItem(item)}
                    variant="ghost"
                  />
                </div>
              </div>
            ))}
        </div>
      )}
    </div>
  );
}
