import { Braces, Pencil, Plus, Trash2 } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import EmptyState from "@/components/ui/empty-state";
import PageHeader from "@/components/ui/page-header";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import { useConfirm } from "@/hooks/use-confirm";
import {
  useCustomFieldDefinitions,
  useDeleteCustomFieldDefinition,
} from "@/hooks/use-custom-fields";
import type { CustomFieldDefinition } from "@/lib/api/types";
import { cn } from "@/lib/utils";

import CustomFieldDrawer from "./components/custom-field-drawer";

type EntityType = "employee" | "employment" | "department";

export default function CustomFieldsPage() {
  const { t } = useTranslation("custom_fields");
  const { confirm, ConfirmationDialog } = useConfirm();
  const toast = useToast();

  const [activeTab, setActiveTab] = useState<EntityType>("employee");
  const [isDrawerOpen, setIsDrawerOpen] = useState(false);
  const [editingField, setEditingField] = useState<CustomFieldDefinition | null>(null);

  // Queries
  const { data: fields = [], isLoading } = useCustomFieldDefinitions(activeTab);

  // Mutations
  const deleteMutation = useDeleteCustomFieldDefinition();

  const handleDelete = async (field: CustomFieldDefinition) => {
    const ok = await confirm({
      title: t("delete"),
      message: t("delete_confirm"),
      variant: "danger",
    });

    if (ok) {
      deleteMutation.mutate(field.id, {
        onSuccess: () => {
          toast.success(t("field_deleted"));
        },
        onError: (err) => {
          toast.error(err instanceof Error ? err.message : "Failed to delete field");
        },
      });
    }
  };

  const handleEdit = (field: CustomFieldDefinition) => {
    setEditingField(field);
    setIsDrawerOpen(true);
  };

  const handleCreate = () => {
    setEditingField(null);
    setIsDrawerOpen(true);
  };

  return (
    <div className="space-y-8 animate-in fade-in duration-500 pb-20">
      <PageHeader
        actions={
          <Button
            icon={<Plus size={18} />}
            onClick={handleCreate}
            variant="ghost"
          >
            {t("create_field").toUpperCase()}
          </Button>
        }
        icon={<Braces size={24} />}
        subtitle={t("description")}
        title={t("title")}
      />

      {/* Entity Tabs Selection */}
      <div className="flex bg-slate-100 dark:bg-slate-700/50 p-1.5 rounded-[22px] w-fit border border-slate-200 dark:border-slate-700/50 shadow-inner">
        <Button
          className={cn(
            "flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all",
            activeTab === "employee"
              ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800"
              : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200"
          )}
          onClick={() => setActiveTab("employee")}
          variant="ghost"
        >
          {t("entities.employee")}
        </Button>
        <Button
          className={cn(
            "flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all",
            activeTab === "employment"
              ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800"
              : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200"
          )}
          onClick={() => setActiveTab("employment")}
          variant="ghost"
        >
          {t("entities.employment")}
        </Button>
        <Button
          className={cn(
            "flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all",
            activeTab === "department"
              ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800"
              : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200"
          )}
          onClick={() => setActiveTab("department")}
          variant="ghost"
        >
          {t("entities.department")}
        </Button>
      </div>

      {/* Main Content Area */}
      <div className="animate-in slide-in-from-bottom-4 duration-500">
        {isLoading ? (
          <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 p-10 rounded-[2.5rem] shadow-sm animate-pulse h-80" />
        ) : fields.length === 0 ? (
          <EmptyState
            description={t("no_fields")}
            icon={<Braces size={40} />}
            title={t("title")}
          />
        ) : (
          <div className="bg-white dark:bg-slate-900 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 overflow-hidden shadow-sm p-4">
            <div className="overflow-x-auto">
              <table className="w-full text-left border-collapse">
                <thead>
                  <tr className="border-b border-slate-100 dark:border-slate-800">
                    <th className="px-6 py-4 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">
                      {t("field_label")} / {t("field_key")}
                    </th>
                    <th className="px-6 py-4 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">
                      {t("field_type")}
                    </th>
                    <th className="px-6 py-4 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">
                      {t("section_name")}
                    </th>
                    <th className="px-6 py-4 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">
                      {t("display_order")}
                    </th>
                    <th className="px-6 py-4 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">
                      Settings
                    </th>
                    <th className="px-6 py-4 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest text-right">
                      Actions
                    </th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-slate-100 dark:divide-slate-800">
                  {fields.map((field) => (
                    <tr
                      className="hover:bg-slate-50/50 dark:hover:bg-slate-800/20 transition-all duration-300"
                      key={field.id}
                    >
                      <td className="px-6 py-4">
                        <div className="flex flex-col">
                          <span className="font-bold text-sm text-slate-800 dark:text-slate-200">
                            {field.field_label}
                          </span>
                          <code className="text-xs text-slate-400 mt-0.5 font-mono">
                            {field.field_key}
                          </code>
                        </div>
                      </td>
                      <td className="px-6 py-4">
                        <span className="text-xs font-semibold px-2.5 py-1 rounded-full bg-slate-100 dark:bg-slate-800 text-slate-600 dark:text-slate-300 border border-slate-200/50 dark:border-slate-700/50">
                          {t(`types.${field.field_type}`)}
                        </span>
                      </td>
                      <td className="px-6 py-4">
                        <span className="text-xs text-slate-600 dark:text-slate-300 font-medium">
                          {field.section_name || "-"}
                        </span>
                      </td>
                      <td className="px-6 py-4">
                        <span className="text-xs font-bold text-slate-500 dark:text-slate-400">
                          {field.display_order}
                        </span>
                      </td>
                      <td className="px-6 py-4">
                        <div className="flex flex-wrap gap-1.5">
                          {field.is_required && (
                            <span className="text-[10px] font-bold tracking-tight px-2 py-0.5 rounded bg-red-50 dark:bg-red-950/30 text-red-600 dark:text-red-400 border border-red-100 dark:border-red-900/30">
                              {t("is_required")}
                            </span>
                          )}
                          {field.is_visible_to_employee && (
                            <span className="text-[10px] font-bold tracking-tight px-2 py-0.5 rounded bg-blue-50 dark:bg-blue-950/30 text-blue-600 dark:text-blue-400 border border-blue-100 dark:border-blue-900/30">
                              ESS
                            </span>
                          )}
                          {field.is_filterable && (
                            <span className="text-[10px] font-bold tracking-tight px-2 py-0.5 rounded bg-emerald-50 dark:bg-emerald-950/30 text-emerald-600 dark:text-emerald-400 border border-emerald-100 dark:border-emerald-900/30">
                              Filter
                            </span>
                          )}
                          {field.is_exportable && (
                            <span className="text-[10px] font-bold tracking-tight px-2 py-0.5 rounded bg-amber-50 dark:bg-amber-950/30 text-amber-600 dark:text-amber-400 border border-amber-100 dark:border-amber-900/30">
                              Export
                            </span>
                          )}
                        </div>
                      </td>
                      <td className="px-6 py-4 text-right">
                        <div className="flex justify-end gap-2">
                          <Button
                            icon={<Pencil size={14} />}
                            onClick={() => handleEdit(field)}
                            size="icon"
                            variant="outline"
                          />
                          <Button
                            className="hover:text-rose-600 dark:hover:text-rose-400"
                            icon={<Trash2 size={14} />}
                            onClick={() => void handleDelete(field)}
                            size="icon"
                            variant="outline"
                          />
                        </div>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        )}
      </div>

      <CustomFieldDrawer
        editingField={editingField}
        isOpen={isDrawerOpen}
        onClose={() => setIsDrawerOpen(false)}
      />

      <ConfirmationDialog />
    </div>
  );
}
