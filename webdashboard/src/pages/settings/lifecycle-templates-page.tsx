import { ClipboardList, Plus } from "lucide-react";
import React from "react";

import Button from "@/components/ui/button";
import EmptyState from "@/components/ui/empty-state";
import PageHeader from "@/components/ui/page-header";
import { cn } from "@/lib/utils";

import { ChecklistItemsPanel } from "./components/checklist-items-panel";
import { TemplateDrawer } from "./components/template-drawer";
import { TemplateItemDrawer } from "./components/template-item-drawer";
import { TemplateListSidebar } from "./components/template-list-sidebar";
import { useLifecycleTemplatesPage } from "./hooks/use-lifecycle-templates-page";

type ChecklistType = "onboarding" | "offboarding" | "transfer" | "probation_end";

const TABS: ChecklistType[] = ["onboarding", "offboarding", "transfer", "probation_end"];

export default function LifecycleTemplatesPage() {
  const ctx = useLifecycleTemplatesPage();

  return (
    <div className="space-y-8 animate-in fade-in duration-500 pb-20">
      <PageHeader
        actions={
          <Button
            icon={<Plus size={18} />}
            onClick={ctx.handleCreateTemplate}
            variant="ghost"
          >
            CREATE TEMPLATE
          </Button>
        }
        icon={<ClipboardList size={24} />}
        subtitle="Configure onboarding, offboarding, and transition checklist templates for employees."
        title="Lifecycle Checklist Templates"
      />

      {/* Tabs */}
      <div className="flex bg-slate-100 dark:bg-slate-700/50 p-1.5 rounded-[22px] w-fit border border-slate-200 dark:border-slate-700/50 shadow-inner">
        {TABS.map((tab) => (
          <Button
            className={cn(
              "flex items-center gap-2 px-6 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all",
              ctx.activeTab === tab
                ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800"
                : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200"
            )}
            key={tab}
            onClick={() => { ctx.setActiveTab(tab); }}
            variant="ghost"
          >
            {tab.replace("_", " ")}
          </Button>
        ))}
      </div>

      {ctx.isTemplatesLoading ? (
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
          <div className="lg:col-span-1 bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 p-5 sm:p-6 rounded-2xl sm:rounded-[2.5rem] shadow-sm animate-pulse h-96" />
          <div className="lg:col-span-2 bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 p-5 sm:p-6 rounded-2xl sm:rounded-[2.5rem] shadow-sm animate-pulse h-96" />
        </div>
      ) : ctx.filteredTemplates.length === 0 ? (
        <EmptyState
          description={`No templates configured for ${ctx.activeTab.replace("_", " ")}.`}
          icon={<ClipboardList size={40} />}
          title="No Templates Configured"
        />
      ) : (
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-8 items-start">
          <TemplateListSidebar
            filteredTemplates={ctx.filteredTemplates}
            handleDeleteTemplate={ctx.handleDeleteTemplate}
            handleEditTemplate={ctx.handleEditTemplate}
            handleSetDefault={ctx.handleSetDefault}
            handleToggleActive={ctx.handleToggleActive}
            selectedTemplateId={ctx.selectedTemplateId}
            setSelectedTemplateId={ctx.setSelectedTemplateId}
          />
          <ChecklistItemsPanel
            activeTemplate={ctx.activeTemplate}
            handleCreateItem={ctx.handleCreateItem}
            handleDeleteItem={ctx.handleDeleteItem}
            handleEditItem={ctx.handleEditItem}
            isItemsLoading={ctx.isItemsLoading}
            items={ctx.items}
          />
        </div>
      )}

      {/* Template Drawer */}
      <TemplateDrawer
        activeTab={ctx.activeTab}
        editingTemplate={ctx.editingTemplate}
        isOpen={ctx.isTemplateDrawerOpen}
        onClose={() => { ctx.setIsTemplateDrawerOpen(false); }}
      />

      {/* Template Item Drawer */}
      {!!ctx.selectedTemplateId && (
        <TemplateItemDrawer
          editingItem={ctx.editingItem}
          isOpen={ctx.isItemDrawerOpen}
          onClose={() => { ctx.setIsItemDrawerOpen(false); }}
          templateId={ctx.selectedTemplateId}
        />
      )}
    </div>
  );
}
