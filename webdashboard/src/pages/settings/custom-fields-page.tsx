 
/* eslint-disable max-lines-per-function */
import { Braces, Pencil, Plus, Trash2 } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import type { CustomFieldDefinition } from "@/lib/api/types";

import Button from "@/components/ui/button";
import EmptyState from "@/components/ui/empty-state";
import PageHeader from "@/components/ui/page-header";
import ResponsiveTable from "@/components/ui/responsive-table";
import {
  Table,
  TableHeader,
  TableBody,
  TableRow,
  TableHead,
  TableCell,
} from "@/components/ui/table";
import { useToast } from "@/components/ui/toast";
import { useConfirm } from "@/hooks/use-confirm";
import {
  useCustomFieldDefinitions,
  useDeleteCustomFieldDefinition,
} from "@/hooks/use-custom-fields";
import { cn } from "@/lib/utils";

import CustomFieldDrawer from "./components/custom-field-drawer";

type EntityType = "employee" | "employment" | "department";

function CustomFieldMobileCard({
  field,
  onEdit,
  onDelete,
  t,
}: {
  field: CustomFieldDefinition;
  onEdit: (field: CustomFieldDefinition) => void;
  onDelete: (field: CustomFieldDefinition) => void;
  t: (key: string) => string;
}) {
  return (
    <div className="rounded-2xl border border-slate-100 bg-white p-5 shadow-sm transition-all active:scale-[0.99] dark:border-slate-800 dark:bg-slate-900">
      {/* Header: Field label and type */}
      <div className="flex items-start justify-between gap-3">
        <div className="min-w-0 flex-1">
          <div className="truncate text-sm font-bold uppercase tracking-tight text-slate-800 dark:text-slate-100">
            {field.field_label}
          </div>
          <code className="mt-1 block truncate font-mono text-xs text-slate-400">
            {field.field_key}
          </code>
        </div>
        <span className="shrink-0 rounded-full border border-slate-200/50 bg-slate-100 px-2.5 py-1 text-xs font-semibold text-slate-600 dark:border-slate-700/50 dark:bg-slate-800 dark:text-slate-300">
          {t(`types.${field.field_type}`)}
        </span>
      </div>

      {/* Details */}
      <div className="mt-4 space-y-3 border-t border-slate-100 pt-4 dark:border-slate-800">
        <div className="flex items-center justify-between text-xs">
          <span className="font-medium text-slate-600 dark:text-slate-300">
            {t("section_name")}
          </span>
          <span className="text-slate-500 dark:text-slate-400">
            {field.section_name ?? "-"}
          </span>
        </div>
        <div className="flex items-center justify-between text-xs">
          <span className="font-medium text-slate-600 dark:text-slate-300">
            {t("display_order")}
          </span>
          <span className="font-bold text-slate-500 dark:text-slate-400">
            {field.display_order}
          </span>
        </div>
      </div>

      {/* Settings badges */}
      <div className="mt-4 flex flex-wrap gap-1.5 border-t border-slate-100 pt-4 dark:border-slate-800">
        {field.is_required ? (
          <span className="rounded border border-red-100 bg-red-50 px-2 py-0.5 text-[10px] font-bold tracking-tight text-red-600 dark:border-red-900/30 dark:bg-red-950/30 dark:text-red-400">
            {t("is_required")}
          </span>
        ) : null}
        {field.is_visible_to_employee ? (
          <span className="rounded border border-blue-100 bg-blue-50 px-2 py-0.5 text-[10px] font-bold tracking-tight text-blue-600 dark:border-blue-900/30 dark:bg-blue-950/30 dark:text-blue-400">
            ESS
          </span>
        ) : null}
        {field.is_filterable ? (
          <span className="rounded border border-emerald-100 bg-emerald-50 px-2 py-0.5 text-[10px] font-bold tracking-tight text-emerald-600 dark:border-emerald-900/30 dark:bg-emerald-950/30 dark:text-emerald-400">
            Filter
          </span>
        ) : null}
        {field.is_exportable ? (
          <span className="rounded border border-amber-100 bg-amber-50 px-2 py-0.5 text-[10px] font-bold tracking-tight text-amber-600 dark:border-amber-900/30 dark:bg-amber-950/30 dark:text-amber-400">
            Export
          </span>
        ) : null}
      </div>

      {/* Actions */}
      <div className="mt-4 flex justify-end gap-2 border-t border-slate-100 pt-4 dark:border-slate-800">
        <Button
          icon={<Pencil size={14} />}
          onClick={() => { onEdit(field); }}
          size="icon"
          variant="outline"
        />
        <Button
          className="hover:text-rose-600 dark:hover:text-rose-400"
          icon={<Trash2 size={14} />}
          onClick={() => void onDelete(field)}
          size="icon"
          variant="outline"
        />
      </div>
    </div>
  );
}

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
          onClick={() => { setActiveTab("employee"); }}
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
          onClick={() => { setActiveTab("employment"); }}
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
          onClick={() => { setActiveTab("department"); }}
          variant="ghost"
        >
          {t("entities.department")}
        </Button>
      </div>

      {/* Main Content Area */}
      <div className="animate-in slide-in-from-bottom-4 duration-500">
        {isLoading ? (
          <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 p-6 sm:p-10 rounded-2xl sm:rounded-[2.5rem] shadow-sm animate-pulse h-80" />
        ) : fields.length === 0 ? (
          <EmptyState
            description={t("no_fields")}
            icon={<Braces size={40} />}
            title={t("title")}
          />
        ) : (
          <ResponsiveTable
            data={fields}
            getCardKey={(field) => field.id}
            renderCard={(field) => (
              <CustomFieldMobileCard
                field={field}
                onDelete={handleDelete}
                onEdit={handleEdit}
                t={t}
              />
            )}
            renderDesktopTable={() => (
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>{t("field_label")} / {t("field_key")}</TableHead>
                    <TableHead>{t("field_type")}</TableHead>
                    <TableHead>{t("section_name")}</TableHead>
                    <TableHead>{t("display_order")}</TableHead>
                    <TableHead>Settings</TableHead>
                    <TableHead className="text-right">Actions</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {fields.map((field) => (
                    <TableRow key={field.id}>
                      <TableCell>
                        <div className="flex flex-col">
                          <span className="font-bold text-sm text-slate-800 dark:text-slate-200">
                            {field.field_label}
                          </span>
                          <code className="text-xs text-slate-400 mt-0.5 font-mono">
                            {field.field_key}
                          </code>
                        </div>
                      </TableCell>
                      <TableCell>
                        <span className="text-xs font-semibold px-2.5 py-1 rounded-full bg-slate-100 dark:bg-slate-800 text-slate-600 dark:text-slate-300 border border-slate-200/50 dark:border-slate-700/50">
                          {t(`types.${field.field_type}`)}
                        </span>
                      </TableCell>
                      <TableCell>
                        <span className="text-xs text-slate-600 dark:text-slate-300 font-medium">
                          {field.section_name ?? "-"}
                        </span>
                      </TableCell>
                      <TableCell>
                        <span className="text-xs font-bold text-slate-500 dark:text-slate-400">
                          {field.display_order}
                        </span>
                      </TableCell>
                      <TableCell>
                        <div className="flex flex-wrap gap-1.5">
                          {field.is_required ? (
                            <span className="text-[10px] font-bold tracking-tight px-2 py-0.5 rounded bg-red-50 dark:bg-red-950/30 text-red-600 dark:text-red-400 border border-red-100 dark:border-red-900/30">
                              {t("is_required")}
                            </span>
                          ) : null}
                          {field.is_visible_to_employee ? (
                            <span className="text-[10px] font-bold tracking-tight px-2 py-0.5 rounded bg-blue-50 dark:bg-blue-950/30 text-blue-600 dark:text-blue-400 border border-blue-100 dark:border-blue-900/30">
                              ESS
                            </span>
                          ) : null}
                          {field.is_filterable ? (
                            <span className="text-[10px] font-bold tracking-tight px-2 py-0.5 rounded bg-emerald-50 dark:bg-emerald-950/30 text-emerald-600 dark:text-emerald-400 border border-emerald-100 dark:border-emerald-900/30">
                              Filter
                            </span>
                          ) : null}
                          {field.is_exportable ? (
                            <span className="text-[10px] font-bold tracking-tight px-2 py-0.5 rounded bg-amber-50 dark:bg-amber-950/30 text-amber-600 dark:text-amber-400 border border-amber-100 dark:border-amber-900/30">
                              Export
                            </span>
                          ) : null}
                        </div>
                      </TableCell>
                      <TableCell className="text-right">
                        <div className="flex justify-end gap-2">
                          <Button
                            icon={<Pencil size={14} />}
                            onClick={() => { handleEdit(field); }}
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
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            )}
          />
        )}
      </div>

      <CustomFieldDrawer
        editingField={editingField}
        isOpen={isDrawerOpen}
        onClose={() => { setIsDrawerOpen(false); }}
      />

      <ConfirmationDialog />
    </div>
  );
}
