import { Pencil, Trash2 } from "lucide-react";
import React from "react";

import type { LifecycleChecklistTemplate } from "@/lib/api/types";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import { cn } from "@/lib/utils";

interface TemplateListSidebarProps {
  filteredTemplates: LifecycleChecklistTemplate[];
  handleDeleteTemplate: (template: LifecycleChecklistTemplate, e: React.MouseEvent) => void;
  handleEditTemplate: (template: LifecycleChecklistTemplate, e: React.MouseEvent) => void;
  handleSetDefault: (template: LifecycleChecklistTemplate, e: React.MouseEvent) => void;
  handleToggleActive: (template: LifecycleChecklistTemplate, e: React.MouseEvent) => void;
  selectedTemplateId: string | null;
  setSelectedTemplateId: (id: string | null) => void;
}

export function TemplateListSidebar({
  filteredTemplates,
  selectedTemplateId,
  setSelectedTemplateId,
  handleEditTemplate,
  handleDeleteTemplate,
  handleSetDefault,
  handleToggleActive,
}: TemplateListSidebarProps) {
  return (
    <div className="lg:col-span-1 space-y-4">
      <Typography className="text-xs font-extrabold text-slate-400 uppercase tracking-widest px-2" variant="h3">
        Templates ({filteredTemplates.length})
      </Typography>
      <div className="space-y-3">
        {filteredTemplates.map((template) => {
          const isSelected = selectedTemplateId === template.id;
          return (
            <div
              aria-pressed={isSelected}
              className={cn(
                "p-5 rounded-3xl border transition-all duration-300 cursor-pointer flex flex-col gap-3 group relative overflow-hidden",
                isSelected
                  ? "bg-blue-50/40 dark:bg-slate-800/80 border-blue-200 dark:border-slate-700 shadow-sm"
                  : "bg-white dark:bg-slate-900 border-slate-100 dark:border-slate-800 hover:border-slate-200 dark:hover:border-slate-700 hover:shadow-sm"
              )}
              key={template.id}
              onClick={() => { setSelectedTemplateId(template.id); }}
              onKeyDown={(e) => { if (e.key === "Enter" || e.key === " ") setSelectedTemplateId(template.id); }}
              role="button"
              tabIndex={0}
            >
              <div className="flex items-start justify-between gap-4">
                <div>
                  <div className="flex items-center gap-2">
                    <span className="font-bold text-slate-800 dark:text-slate-200 text-sm">
                      {template.name}
                    </span>
                    {template.is_default ? (
                      <span className="bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-400 text-[10px] px-2 py-0.5 rounded-full font-bold uppercase tracking-wider">
                        Default
                      </span>
                    ) : null}
                  </div>
                  {!!template.description && (
                    <Typography className="text-xs text-slate-400 dark:text-slate-500 mt-1 line-clamp-2" variant="p">
                      {template.description}
                    </Typography>
                  )}
                </div>
              </div>

              <div className="flex items-center justify-between mt-2 pt-3 border-t border-slate-50 dark:border-slate-800/50">
                <Button
                  className={cn(
                    "text-[10px] font-bold tracking-wider px-2.5 py-1 rounded-full h-auto",
                    template.is_active
                      ? "bg-emerald-50 dark:bg-emerald-950/20 text-emerald-600 dark:text-emerald-400 hover:bg-emerald-100"
                      : "bg-slate-100 dark:bg-slate-800 text-slate-500 dark:text-slate-400 hover:bg-slate-200"
                  )}
                  onClick={(e) => { handleToggleActive(template, e); }}
                  variant="ghost"
                >
                  {template.is_active ? "ACTIVE" : "INACTIVE"}
                </Button>

                <div className="flex items-center gap-1">
                  {!template.is_default && (
                    <Button
                      className="text-[10px] font-bold tracking-wider px-2 py-1 rounded-full h-auto hover:bg-slate-100 dark:hover:bg-slate-800"
                      onClick={(e) => { handleSetDefault(template, e); }}
                      variant="ghost"
                    >
                      SET DEFAULT
                    </Button>
                  )}
                  <Button
                    icon={<Pencil size={12} />}
                    onClick={(e) => { handleEditTemplate(template, e); }}
                    variant="ghost"
                  />
                  <Button
                    className="text-rose-500 hover:text-rose-600 hover:bg-rose-50 dark:hover:bg-rose-950/20"
                    icon={<Trash2 size={12} />}
                    onClick={(e) => { handleDeleteTemplate(template, e); }}
                    variant="ghost"
                  />
                </div>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
