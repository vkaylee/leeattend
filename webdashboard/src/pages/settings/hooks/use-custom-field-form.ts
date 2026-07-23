import { useState } from "react";
import { useTranslation } from "react-i18next";

import type {
  CustomFieldDefinition,
  CreateCustomFieldDefinitionRequest,
  UpdateCustomFieldDefinitionRequest,
} from "@/lib/api/types";

import { useToast } from "@/components/ui/toast";
import {
  useCreateCustomFieldDefinition,
  useUpdateCustomFieldDefinition,
} from "@/hooks/use-custom-fields";

// eslint-disable-next-line complexity, max-lines-per-function
export function useCustomFieldForm(
  editingField: CustomFieldDefinition | null,
  onClose: () => void,
) {
  const { t } = useTranslation("custom_fields");
  const toast = useToast();

  const createMutation = useCreateCustomFieldDefinition();
  const updateMutation = useUpdateCustomFieldDefinition();

  // ── Form State ─────────────────────────────────────────────────────
  const [entityType, setEntityType] = useState(editingField?.entity_type ?? "employee");
  const [fieldKey, setFieldKey] = useState(editingField?.field_key ?? "");
  const [fieldLabel, setFieldLabel] = useState(editingField?.field_label ?? "");
  const [fieldType, setFieldType] = useState(editingField?.field_type ?? "text");

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

  // eslint-disable-next-line complexity
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

  return {
    // State
    entityType,
    setEntityType,
    fieldKey,
    setFieldKey,
    fieldLabel,
    setFieldLabel,
    fieldType,
    setFieldType,
    optionsText,
    setOptionsText,
    defaultValue,
    setDefaultValue,
    isRequired,
    setIsRequired,
    isVisibleToEmployee,
    setIsVisibleToEmployee,
    isFilterable,
    setIsFilterable,
    isExportable,
    setIsExportable,
    displayOrder,
    setDisplayOrder,
    sectionName,
    setSectionName,
    // Derived
    isOptionField,
    loading,
    editingField,
    // Actions
    handleSave,
  };
}
