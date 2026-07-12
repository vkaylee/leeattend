import { ClipboardList, Plus, Pencil, Trash2 } from "lucide-react";
import React, { useState, useEffect } from "react";

import type {
  LifecycleChecklistTemplate,
  LifecycleChecklistTemplateItem,
} from "@/lib/api/types";

import Button from "@/components/ui/button";
import EmptyState from "@/components/ui/empty-state";
import PageHeader from "@/components/ui/page-header";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import { useConfirm } from "@/hooks/use-confirm";
import {
  useLifecycleTemplates,
  useUpdateLifecycleTemplate,
  useDeleteLifecycleTemplate,
  useLifecycleTemplateItems,
  useDeleteLifecycleTemplateItem,
} from "@/hooks/use-lifecycle";
import { cn } from "@/lib/utils";

import { TemplateDrawer } from "./components/template-drawer";
import { TemplateItemDrawer } from "./components/template-item-drawer";

type ChecklistType = "onboarding" | "offboarding" | "transfer" | "probation_end";

export default function LifecycleTemplatesPage() {
  const { confirm } = useConfirm();
  const toast = useToast();

  const [activeTab, setActiveTab] = useState<ChecklistType>("onboarding");
  const [selectedTemplateId, setSelectedTemplateId] = useState<string | null>(null);

  // Modals / Drawers State
  const [isTemplateDrawerOpen, setIsTemplateDrawerOpen] = useState(false);
  const [editingTemplate, setEditingTemplate] = useState<LifecycleChecklistTemplate | null>(null);

  const [isItemDrawerOpen, setIsItemDrawerOpen] = useState(false);
  const [editingItem, setEditingItem] = useState<LifecycleChecklistTemplateItem | null>(null);

  // Queries
  const { data: templates = [], isLoading: isTemplatesLoading } = useLifecycleTemplates();
  const { data: items = [], isLoading: isItemsLoading } = useLifecycleTemplateItems(selectedTemplateId ?? "");

  // Mutations
  const updateTemplateMutation = useUpdateLifecycleTemplate();
  const deleteTemplateMutation = useDeleteLifecycleTemplate();

  const deleteItemMutation = useDeleteLifecycleTemplateItem(selectedTemplateId ?? "");

  const filteredTemplates = templates.filter(
    (temp) => temp.checklist_type === activeTab
  );

  // Set default selected template if none is selected or if active tab changed
  useEffect(() => {
    if (filteredTemplates.length > 0) {
      const defaultTemp = filteredTemplates.find((t) => t.is_default) ?? filteredTemplates[0];
      setSelectedTemplateId(defaultTemp.id);
    } else {
      setSelectedTemplateId(null);
    }
  }, [activeTab, templates.length]);

  const activeTemplate = templates.find((t) => t.id === selectedTemplateId);

  // Handlers
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
          if (selectedTemplateId === template.id) {
            setSelectedTemplateId(null);
          }
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
      {
        id: template.id,
        payload: { is_default: true },
      },
      {
        onSuccess: () => {
          toast.success(`Set "${template.name}" as default template`);
        },
        onError: (err) => {
          toast.error(err instanceof Error ? err.message : "Failed to set default template");
        },
      }
    );
  };

  const handleToggleActive = (template: LifecycleChecklistTemplate, e: React.MouseEvent) => {
    e.stopPropagation();
    updateTemplateMutation.mutate(
      {
        id: template.id,
        payload: { is_active: !template.is_active },
      },
      {
        onSuccess: () => {
          toast.success(`Template ${template.is_active ? "disabled" : "enabled"} successfully`);
        },
        onError: (err) => {
          toast.error(err instanceof Error ? err.message : "Failed to update template status");
        },
      }
    );
  };

  // Item CRUD
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
        onSuccess: () => {
          toast.success("Checklist item deleted successfully");
        },
        onError: (err) => {
          toast.error(err instanceof Error ? err.message : "Failed to delete checklist item");
        },
      });
    }
  };

  return (
    <div className="space-y-8 animate-in fade-in duration-500 pb-20">
      <PageHeader
        actions={
          <Button
            icon={<Plus size={18} />}
            onClick={handleCreateTemplate}
            variant="ghost"
          >
            CREATE TEMPLATE
          </Button>
        }
        icon={<ClipboardList size={24} />}
        subtitle="Configure onboarding, offboarding, and transition checklist templates for employees."
        title="Lifecycle Checklist Templates"
      />

      {/* Tabs Selection */}
      <div className="flex bg-slate-100 dark:bg-slate-700/50 p-1.5 rounded-[22px] w-fit border border-slate-200 dark:border-slate-700/50 shadow-inner">
        {(["onboarding", "offboarding", "transfer", "probation_end"] as ChecklistType[]).map((tab) => (
          <Button
            className={cn(
              "flex items-center gap-2 px-6 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all",
              activeTab === tab
                ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800"
                : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200"
            )}
            key={tab}
            onClick={() => { setActiveTab(tab); }}
            variant="ghost"
          >
            {tab.replace("_", " ")}
          </Button>
        ))}
      </div>

      {isTemplatesLoading ? (
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
          <div className="lg:col-span-1 bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 p-6 rounded-[2.5rem] shadow-sm animate-pulse h-96" />
          <div className="lg:col-span-2 bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 p-6 rounded-[2.5rem] shadow-sm animate-pulse h-96" />
        </div>
      ) : filteredTemplates.length === 0 ? (
        <EmptyState
          description={`No templates configured for ${activeTab.replace("_", " ")}.`}
          icon={<ClipboardList size={40} />}
          title="No Templates Configured"
        />
      ) : (
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-8 items-start">
          {/* Templates List Side */}
          <div className="lg:col-span-1 space-y-4">
            <Typography className="text-xs font-extrabold text-slate-400 uppercase tracking-widest px-2" variant="h3">
              Templates ({filteredTemplates.length})
            </Typography>
            <div className="space-y-3">
              {filteredTemplates.map((template) => {
                const isSelected = selectedTemplateId === template.id;
                return (
                  <div
                    className={cn(
                      "p-5 rounded-3xl border transition-all duration-300 cursor-pointer flex flex-col gap-3 group relative overflow-hidden",
                      isSelected
                        ? "bg-blue-50/40 dark:bg-slate-800/80 border-blue-200 dark:border-slate-700 shadow-sm"
                        : "bg-white dark:bg-slate-900 border-slate-100 dark:border-slate-800 hover:border-slate-200 dark:hover:border-slate-700 hover:shadow-sm"
                    )}
                    aria-pressed={isSelected}
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
                          onClick={(e) => handleDeleteTemplate(template, e)}
                          variant="ghost"
                        />
                      </div>
                    </div>
                  </div>
                );
              })}
            </div>
          </div>

          {/* Checklist Items Side */}
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
              <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 p-8 rounded-[2.5rem] shadow-sm animate-pulse h-80" />
            ) : !activeTemplate ? (
              <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 p-12 rounded-[2.5rem] text-center text-slate-400">
                Select a template to view and manage its checklist items.
              </div>
            ) : items.length === 0 ? (
              <div className="bg-white dark:bg-slate-900 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 p-16 text-center space-y-4">
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
              <div className="bg-white dark:bg-slate-900 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 overflow-hidden shadow-sm p-4 space-y-3">
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
        </div>
      )}

      {/* Template Drawer */}
      <TemplateDrawer
        activeTab={activeTab}
        editingTemplate={editingTemplate}
        isOpen={isTemplateDrawerOpen}
        onClose={() => { setIsTemplateDrawerOpen(false); }}
      />

      {/* Template Item Drawer */}
      {!!selectedTemplateId && (
        <TemplateItemDrawer
          editingItem={editingItem}
          isOpen={isItemDrawerOpen}
          onClose={() => { setIsItemDrawerOpen(false); }}
          templateId={selectedTemplateId}
        />
      )}
    </div>
  );
}
