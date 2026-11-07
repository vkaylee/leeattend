import { ClipboardList, Plus } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import EmptyState from "@/components/ui/empty-state";
import { SettingsHeader } from "@/components/ui/settings/settings-header";
import { SettingsSegmentedControl } from "@/components/ui/settings/settings-segmented-control";
import { cn } from "@/lib/utils";
import { ChecklistItemsPanel } from "./components/checklist-items-panel";
import { TemplateDrawer } from "./components/template-drawer";
import { TemplateItemDrawer } from "./components/template-item-drawer";
import { TemplateListSidebar } from "./components/template-list-sidebar";
import { useLifecycleTemplatesPage } from "./hooks/use-lifecycle-templates-page";

type ChecklistType = "onboarding" | "offboarding" | "transfer" | "probation_end";

const TABS: ChecklistType[] = ["onboarding", "offboarding", "transfer", "probation_end"];

const TAB_LABELS: Record<ChecklistType, string> = {
  onboarding: "settings.lifecycle.tabs.onboarding",
  offboarding: "settings.lifecycle.tabs.offboarding",
  transfer: "settings.lifecycle.tabs.transfer",
  probation_end: "settings.lifecycle.tabs.probation_end",
};

export default function LifecycleTemplatesPage() {
  const { t } = useTranslation();
  const ctx = useLifecycleTemplatesPage();

  return (
    <div className="space-y-8 animate-in fade-in duration-500 pb-20">
      <SettingsHeader
        actions={
          <Button
            className="rounded-xl px-4 py-2 text-xs font-semibold shadow-xs normal-case tracking-normal"
            icon={<Plus size={16} />}
            onClick={ctx.handleCreateTemplate}
            variant="primary"
          >
            {t("settings.lifecycle.create_template", { defaultValue: "Tạo mẫu quy trình" })}
          </Button>
        }
        description={t("settings.lifecycle.subtitle", { defaultValue: "Cấu hình danh mục kiểm tra tiếp nhận, thôi việc và luân chuyển nhân sự cho nhân viên." })}
        icon={<ClipboardList size={20} />}
        title={t("settings.lifecycle.title", { defaultValue: "Mẫu quy trình nhân sự" })}
      />

      {/* Tabs */}
      <div className="overflow-x-auto max-w-full pb-1">
        <SettingsSegmentedControl
          onChange={(tab) => ctx.setActiveTab(tab as typeof TABS[number])}
          options={TABS.map((tab) => ({
            label: t(TAB_LABELS[tab], { defaultValue: tab.replace("_", " ") }),
            value: tab,
          }))}
          testId="lifecycle-tabs"
          value={ctx.activeTab}
        />
      </div>

      {ctx.isTemplatesLoading ? (
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
          <div className="lg:col-span-1 bg-white dark:bg-slate-900 border border-slate-200/80 dark:border-slate-800/80 p-6 rounded-2xl shadow-2xs animate-pulse h-96" />
          <div className="lg:col-span-2 bg-white dark:bg-slate-900 border border-slate-200/80 dark:border-slate-800/80 p-6 rounded-2xl shadow-2xs animate-pulse h-96" />
        </div>
      ) : ctx.filteredTemplates.length === 0 ? (
        <EmptyState
          action={
            <Button
              className="rounded-xl px-4 py-2 text-xs font-semibold shadow-xs normal-case tracking-normal"
              icon={<Plus size={16} />}
              onClick={ctx.handleCreateTemplate}
              variant="primary"
            >
              {t("settings.lifecycle.create_template", { defaultValue: "Tạo mẫu quy trình" })}
            </Button>
          }
          description={t("settings.lifecycle.empty.description", { defaultValue: "Chưa có mẫu danh mục nào được cấu hình cho quy trình này." })}
          icon={<ClipboardList size={40} />}
          title={t("settings.lifecycle.empty.title", { defaultValue: "Chưa có mẫu quy trình" })}
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
