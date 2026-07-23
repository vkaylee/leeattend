import { Loader2 } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import type { CustomFieldDefinition } from "@/lib/api/types";

import Button from "@/components/ui/button";
import { Combobox } from "@/components/ui/combobox";
import FormField from "@/components/ui/form-field";
import { useCustomFieldForm } from "@/pages/settings/hooks/use-custom-field-form";

interface CustomFieldFormContentProps {
  editingField: CustomFieldDefinition | null;
  onClose: () => void;
}

// eslint-disable-next-line max-lines-per-function
export function CustomFieldFormContent({ editingField, onClose }: CustomFieldFormContentProps) {
  const { t } = useTranslation("custom_fields");
  const form = useCustomFieldForm(editingField, onClose);

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
          onChange={form.setEntityType}
          options={entityTypeOptions}
          value={form.entityType}
        />
      </div>

      {/* 2. Field Key */}
      <FormField
        disabled={!!editingField}
        label={t("field_key")}
        name="fieldKey"
        onChange={(e) => { form.setFieldKey(e.target.value.toLowerCase().replace(/[^a-z0-9_]/g, "")); }}
        placeholder={t("field_key_placeholder")}
        required={!editingField}
        value={form.fieldKey}
      />

      {/* 3. Field Label */}
      <FormField
        label={t("field_label")}
        name="fieldLabel"
        onChange={(e) => { form.setFieldLabel(e.target.value); }}
        placeholder={t("field_label_placeholder")}
        required
        value={form.fieldLabel}
      />

      {/* 4. Field Type */}
      <div className="space-y-2">
        <label className={LABEL_BASE}>{t("field_type")}</label>
        <Combobox
          className="bg-white dark:bg-slate-900 border-slate-200 dark:border-slate-700 h-14"
          disabled={!!editingField}
          onChange={form.setFieldType}
          options={fieldTypeOptions}
          value={form.fieldType}
        />
      </div>

      {/* 5. Options (Conditional) */}
      {form.isOptionField ? (
        <div className="space-y-2">
          <label className={LABEL_BASE}>{t("options")} ({t("is_required")})</label>
          <textarea
            className="w-full px-6 py-4 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-2xl focus:ring-4 focus:ring-primary-500/10 outline-none transition-all font-medium text-sm shadow-sm min-h-[120px]"
            onChange={(e) => { form.setOptionsText(e.target.value); }}
            placeholder={t("options_placeholder")}
            value={form.optionsText}
          />
        </div>
      ) : null}

      {/* 6. Default Value */}
      <FormField
        label={t("default_value")}
        name="defaultValue"
        onChange={(e) => { form.setDefaultValue(e.target.value); }}
        placeholder={t("default_value_placeholder")}
        value={form.defaultValue}
      />

      {/* 7. UI Group Section */}
      <FormField
        label={t("section_name")}
        name="sectionName"
        onChange={(e) => { form.setSectionName(e.target.value); }}
        placeholder={t("section_name_placeholder")}
        value={form.sectionName}
      />

      {/* 8. Display Order */}
      <FormField
        label={t("display_order")}
        name="displayOrder"
        onChange={(e) => { form.setDisplayOrder(e.target.value.replace(/[^0-9]/g, "")); }}
        type="number"
        value={form.displayOrder}
      />

      {/* 9. Boolean Flags checkboxes */}
      <div className="space-y-3 p-6 bg-slate-50 dark:bg-slate-800/40 border border-slate-100/50 dark:border-slate-800 rounded-3xl">
        {/* Required check */}
        <div className="flex items-center gap-4">
          <input
            checked={form.isRequired}
            className="w-5 h-5 rounded border-slate-300 text-primary-600 focus:ring-primary-500"
            id="is-required"
            onChange={(e) => { form.setIsRequired(e.target.checked); }}
            type="checkbox"
          />
          <label className="font-bold text-sm text-slate-700 dark:text-slate-300 cursor-pointer" htmlFor="is-required">
            {t("is_required")}
          </label>
        </div>

        {/* Visible to employee check */}
        <div className="flex items-center gap-4">
          <input
            checked={form.isVisibleToEmployee}
            className="w-5 h-5 rounded border-slate-300 text-primary-600 focus:ring-primary-500"
            id="is-visible"
            onChange={(e) => { form.setIsVisibleToEmployee(e.target.checked); }}
            type="checkbox"
          />
          <label className="font-bold text-sm text-slate-700 dark:text-slate-300 cursor-pointer" htmlFor="is-visible">
            {t("is_visible_to_employee")}
          </label>
        </div>

        {/* Filterable check */}
        <div className="flex items-center gap-4">
          <input
            checked={form.isFilterable}
            className="w-5 h-5 rounded border-slate-300 text-primary-600 focus:ring-primary-500"
            id="is-filterable"
            onChange={(e) => { form.setIsFilterable(e.target.checked); }}
            type="checkbox"
          />
          <label className="font-bold text-sm text-slate-700 dark:text-slate-300 cursor-pointer" htmlFor="is-filterable">
            {t("is_filterable")}
          </label>
        </div>

        {/* Exportable check */}
        <div className="flex items-center gap-4">
          <input
            checked={form.isExportable}
            className="w-5 h-5 rounded border-slate-300 text-primary-600 focus:ring-primary-500"
            id="is-exportable"
            onChange={(e) => { form.setIsExportable(e.target.checked); }}
            type="checkbox"
          />
          <label className="font-bold text-sm text-slate-700 dark:text-slate-300 cursor-pointer" htmlFor="is-exportable">
            {t("is_exportable")}
          </label>
        </div>
      </div>

      {/* Buttons */}
      <div className="pt-6 border-t border-slate-100 dark:border-slate-800 flex justify-end gap-3">
        <Button disabled={form.loading} onClick={onClose} variant="ghost">
          {t("cancel")}
        </Button>
        <Button loading={form.loading} onClick={form.handleSave} variant="primary">
          {form.loading ? (
            <Loader2 className="w-4 h-4 animate-spin" />
          ) : (
            t("save")
          )}
        </Button>
      </div>
    </div>
  );
}
