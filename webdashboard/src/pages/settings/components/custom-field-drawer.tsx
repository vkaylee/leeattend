import React, { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { Loader2 } from "lucide-react";

import Button from "@/components/ui/button";
import Sheet from "@/components/ui/sheet";
import FormField from "@/components/ui/form-field";
import { Combobox } from "@/components/ui/combobox";
import { useToast } from "@/components/ui/toast";
import type {
  CustomFieldDefinition,
  CreateCustomFieldDefinitionRequest,
  UpdateCustomFieldDefinitionRequest,
} from "@/lib/api/types";
import {
  useCreateCustomFieldDefinition,
  useUpdateCustomFieldDefinition,
} from "@/hooks/use-custom-fields";

interface CustomFieldDrawerProps {
  editingField: CustomFieldDefinition | null;
  isOpen: boolean;
  onClose: () => void;
}

export default function CustomFieldDrawer({
  isOpen,
  onClose,
  editingField,
}: CustomFieldDrawerProps) {
  const { t } = useTranslation("custom_fields");

  return (
    <Sheet
      isOpen={isOpen}
      onClose={onClose}
      title={editingField ? t("edit_field") : t("create_field")}
    >
      {isOpen ? (
        <CustomFieldFormContent editingField={editingField} onClose={onClose} />
      ) : null}
    </Sheet>
  );
}

interface ContentProps {
  editingField: CustomFieldDefinition | null;
  onClose: () => void;
}

function CustomFieldFormContent({ editingField, onClose }: ContentProps) {
  const { t } = useTranslation("custom_fields");
  const toast = useToast();

  const createMutation = useCreateCustomFieldDefinition();
  const updateMutation = useUpdateCustomFieldDefinition();

  // ── Form State ─────────────────────────────────────────────────────
  const [entityType, setEntityType] = useState(editingField?.entity_type ?? "employee");
  const [fieldKey, setFieldKey] = useState(editingField?.field_key ?? "");
  const [fieldLabel, setFieldLabel] = useState(editingField?.field_label ?? "");
  const [fieldType, setFieldType] = useState(editingField?.field_type ?? "text");

  // Options parsing
  const getInitialOptionsText = () => {
    if (!editingField?.options) return "";
    try {
      const opts = editingField.options;
      if (Array.isArray(opts)) {
        return opts
          .map((o) => {
            if (o && typeof o === "object" && "value" in o) {
              return (o as { value: string }).value;
            }
            return String(o);
          })
          .join("\n");
      }
    } catch {
      // Ignore
    }
    return "";
  };

  const [optionsText, setOptionsText] = useState(getInitialOptionsText());
  const [defaultValue, setDefaultValue] = useState(editingField?.default_value ?? "");
  const [isRequired, setIsRequired] = useState(editingField?.is_required ?? false);
  const [isVisibleToEmployee, setIsVisibleToEmployee] = useState(
    editingField?.is_visible_to_employee ?? false
  );
  const [isFilterable, setIsFilterable] = useState(editingField?.is_filterable ?? false);
  const [isExportable, setIsExportable] = useState(editingField?.is_exportable ?? true);
  const [displayOrder, setDisplayOrder] = useState(
    editingField?.display_order !== undefined ? String(editingField.display_order) : "0"
  );
  const [sectionName, setSectionName] = useState(editingField?.section_name ?? "");

  const handleSave = () => {
    // Validation
    if (!fieldLabel.trim()) {
      toast.error(t("errors.label_required"), "Validation Error");
      return;
    }

    if (!editingField && !fieldKey.trim()) {
      toast.error(t("errors.key_required"), "Validation Error");
      return;
    }

    // Verify key format
    const keyRegex = /^[a-z0-9_]+$/;
    if (!editingField && !keyRegex.test(fieldKey)) {
      toast.error(t("errors.key_format"), "Validation Error");
      return;
    }

    // Process select/multiselect options
    let optionsJson: unknown[] | null = null;
    const isOptionField = fieldType === "select" || fieldType === "multiselect";
    if (isOptionField) {
      const lines = optionsText
        .split("\n")
        .map((l) => l.trim())
        .filter(Boolean);
      if (lines.length === 0) {
        toast.error(t("errors.options_required"), "Validation Error");
        return;
      }
      optionsJson = lines.map((item) => ({ value: item, label: item }));
    }

    const isPending = createMutation.isPending || updateMutation.isPending;
    if (isPending) return;

    if (editingField) {
      const payload: UpdateCustomFieldDefinitionRequest = {
        field_label: fieldLabel,
        options: optionsJson ?? undefined,
        default_value: defaultValue.trim() !== "" ? defaultValue.trim() : null,
        is_required: isRequired,
        is_visible_to_employee: isVisibleToEmployee,
        is_filterable: isFilterable,
        is_exportable: isExportable,
        display_order: Number(displayOrder) || 0,
        section_name: sectionName.trim() !== "" ? sectionName.trim() : null,
        validation_rules: null,
      };

      updateMutation.mutate(
        { id: editingField.id, payload },
        {
          onSuccess: () => {
            toast.success(t("field_updated"));
            onClose();
          },
          onError: (err) => {
            toast.error(err instanceof Error ? err.message : "Failed to update field");
          },
        }
      );
    } else {
      const payload: CreateCustomFieldDefinitionRequest = {
        entity_type: entityType,
        field_key: fieldKey,
        field_label: fieldLabel,
        field_type: fieldType,
        options: optionsJson ?? undefined,
        default_value: defaultValue.trim() !== "" ? defaultValue.trim() : undefined,
        is_required: isRequired,
        is_visible_to_employee: isVisibleToEmployee,
        is_filterable: isFilterable,
        is_exportable: isExportable,
        display_order: Number(displayOrder) || 0,
        section_name: sectionName.trim() !== "" ? sectionName.trim() : undefined,
        validation_rules: undefined,
      };

      createMutation.mutate(payload, {
        onSuccess: () => {
          toast.success(t("field_created"));
          onClose();
        },
        onError: (err) => {
          toast.error(err instanceof Error ? err.message : "Failed to create field");
        },
      });
    }
  };

  const isOptionField = fieldType === "select" || fieldType === "multiselect";
  const loading = createMutation.isPending || updateMutation.isPending;

  const entityTypeOptions = [
    { value: "employee", label: t("entities.employee") },
    { value: "employment", label: t("entities.employment") },
    { value: "department", label: t("entities.department") },
  ];

  const fieldTypeOptions = [
    { value: "text", label: t("types.text") },
    { value: "number", label: t("types.number") },
    { value: "date", label: t("types.date") },
    { value: "boolean", label: t("types.boolean") },
    { value: "select", label: t("types.select") },
    { value: "multiselect", label: t("types.multiselect") },
    { value: "url", label: t("types.url") },
    { value: "email", label: t("types.email") },
    { value: "phone", label: t("types.phone") },
  ];

  const LABEL_BASE = "block text-[10px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-widest mb-2";

  return (
    <div className="space-y-6">
      {/* 1. Entity Type Selection */}
      <div className="space-y-2">
        <label className={LABEL_BASE}>{t("entity_type")}</label>
        <Combobox
          className="bg-white dark:bg-slate-900 border-slate-200 dark:border-slate-700 h-14"
          disabled={!!editingField}
          onChange={setEntityType}
          options={entityTypeOptions}
          value={entityType}
        />
      </div>

      {/* 2. Field Key */}
      <FormField
        disabled={!!editingField}
        label={t("field_key")}
        name="fieldKey"
        onChange={(e) => setFieldKey(e.target.value.toLowerCase().replace(/[^a-z0-9_]/g, ""))}
        placeholder={t("field_key_placeholder")}
        required={!editingField}
        value={fieldKey}
      />

      {/* 3. Field Label */}
      <FormField
        label={t("field_label")}
        name="fieldLabel"
        onChange={(e) => setFieldLabel(e.target.value)}
        placeholder={t("field_label_placeholder")}
        required
        value={fieldLabel}
      />

      {/* 4. Field Type */}
      <div className="space-y-2">
        <label className={LABEL_BASE}>{t("field_type")}</label>
        <Combobox
          className="bg-white dark:bg-slate-900 border-slate-200 dark:border-slate-700 h-14"
          disabled={!!editingField}
          onChange={setFieldType}
          options={fieldTypeOptions}
          value={fieldType}
        />
      </div>

      {/* 5. Options (Conditional) */}
      {isOptionField && (
        <div className="space-y-2">
          <label className={LABEL_BASE}>{t("options")} ({t("is_required")})</label>
          <textarea
            className="w-full px-6 py-4 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-2xl focus:ring-4 focus:ring-primary-500/10 outline-none transition-all font-medium text-sm shadow-sm min-h-[120px]"
            onChange={(e) => setOptionsText(e.target.value)}
            placeholder={t("options_placeholder")}
            value={optionsText}
          />
        </div>
      )}

      {/* 6. Default Value */}
      <FormField
        label={t("default_value")}
        name="defaultValue"
        onChange={(e) => setDefaultValue(e.target.value)}
        placeholder={t("default_value_placeholder")}
        value={defaultValue}
      />

      {/* 7. UI Group Section */}
      <FormField
        label={t("section_name")}
        name="sectionName"
        onChange={(e) => setSectionName(e.target.value)}
        placeholder={t("section_name_placeholder")}
        value={sectionName}
      />

      {/* 8. Display Order */}
      <FormField
        label={t("display_order")}
        name="displayOrder"
        onChange={(e) => setDisplayOrder(e.target.value.replace(/[^0-9]/g, ""))}
        type="number"
        value={displayOrder}
      />

      {/* 9. Boolean Flags checkboxes */}
      <div className="space-y-3 p-6 bg-slate-50 dark:bg-slate-800/40 border border-slate-100/50 dark:border-slate-800 rounded-3xl">
        {/* Required check */}
        <div className="flex items-center gap-4">
          <input
            checked={isRequired}
            className="w-5 h-5 rounded border-slate-300 text-primary-600 focus:ring-primary-500"
            id="is-required"
            onChange={(e) => setIsRequired(e.target.checked)}
            type="checkbox"
          />
          <label className="font-bold text-sm text-slate-700 dark:text-slate-300 cursor-pointer" htmlFor="is-required">
            {t("is_required")}
          </label>
        </div>

        {/* Visible to employee check */}
        <div className="flex items-center gap-4">
          <input
            checked={isVisibleToEmployee}
            className="w-5 h-5 rounded border-slate-300 text-primary-600 focus:ring-primary-500"
            id="is-visible"
            onChange={(e) => setIsVisibleToEmployee(e.target.checked)}
            type="checkbox"
          />
          <label className="font-bold text-sm text-slate-700 dark:text-slate-300 cursor-pointer" htmlFor="is-visible">
            {t("is_visible_to_employee")}
          </label>
        </div>

        {/* Filterable check */}
        <div className="flex items-center gap-4">
          <input
            checked={isFilterable}
            className="w-5 h-5 rounded border-slate-300 text-primary-600 focus:ring-primary-500"
            id="is-filterable"
            onChange={(e) => setIsFilterable(e.target.checked)}
            type="checkbox"
          />
          <label className="font-bold text-sm text-slate-700 dark:text-slate-300 cursor-pointer" htmlFor="is-filterable">
            {t("is_filterable")}
          </label>
        </div>

        {/* Exportable check */}
        <div className="flex items-center gap-4">
          <input
            checked={isExportable}
            className="w-5 h-5 rounded border-slate-300 text-primary-600 focus:ring-primary-500"
            id="is-exportable"
            onChange={(e) => setIsExportable(e.target.checked)}
            type="checkbox"
          />
          <label className="font-bold text-sm text-slate-700 dark:text-slate-300 cursor-pointer" htmlFor="is-exportable">
            {t("is_exportable")}
          </label>
        </div>
      </div>

      {/* Buttons */}
      <div className="pt-6 border-t border-slate-100 dark:border-slate-800 flex justify-end gap-3">
        <Button disabled={loading} onClick={onClose} variant="ghost">
          {t("cancel")}
        </Button>
        <Button loading={loading} onClick={handleSave} variant="primary">
          {loading ? (
            <Loader2 className="w-4 h-4 animate-spin" />
          ) : (
            t("save")
          )}
        </Button>
      </div>
    </div>
  );
}
