import { Edit2, Check, X, Loader2 } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import type { CustomFieldDefinition, CustomFieldValueResponse } from "@/lib/api/types";

import Button from "@/components/ui/button";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import {
  useCustomFieldDefinitions,
  useCustomFieldValues,
  useSetCustomFieldValues,
} from "@/hooks/use-custom-fields";
import { useMyPermissions } from "@/hooks/use-permissions";

import { CustomFieldEditor } from "./custom-field-editor";
import { CustomFieldRenderer } from "./custom-field-renderer";

interface CustomFieldSectionProps {
  entityId: string;
  entityType: "employee" | "employment" | "department";
}

/** Allowed value types for custom field form state. */
type CustomFieldFormValue = string | number | boolean | null;

function buildInitialFormValues(
  definitions: CustomFieldDefinition[],
  values: CustomFieldValueResponse[],
): Record<string, CustomFieldFormValue> {
  const initial: Record<string, CustomFieldFormValue> = {};
  definitions.forEach((def) => {
    const valueObj = values.find((v) => v.definition_id === def.id);
    const rawValue: unknown = valueObj ? valueObj.value : def.default_value ?? null;
    initial[def.id] = rawValue as CustomFieldFormValue;
  });
  return initial;
}

function buildValidationErrors(
  definitions: CustomFieldDefinition[],
  formValues: Record<string, CustomFieldFormValue>,
): Record<string, string> {
  const nextErrors: Record<string, string> = {};
  definitions.forEach((def) => {
    const val = formValues[def.id];
    if (def.is_required && (val === null || val === undefined || val === "")) {
      nextErrors[def.id] = `${def.field_label} is required`;
    }
  });
  return nextErrors;
}

  // eslint-disable-next-line max-lines-per-function
export const CustomFieldSection: React.FC<CustomFieldSectionProps> = ({
  entityType,
  entityId,
  // eslint-disable-next-line complexity
}) => {
  const { t } = useTranslation("custom_fields");
  const { hasPermission } = useMyPermissions();
  const toast = useToast();

  const [isEditing, setIsEditing] = useState(false);
  const [formValues, setFormValues] = useState<Record<string, CustomFieldFormValue>>({});
  const [errors, setErrors] = useState<Record<string, string>>({});

  // 1. Fetch definitions and values
  const { data: definitions = [], isLoading: isLoadingDefs } =
    useCustomFieldDefinitions(entityType);
  const { data: values = [], isLoading: isLoadingValues } = useCustomFieldValues(
    entityType,
    entityId
  );

  const { mutate: setValues, isPending: isSaving } = useSetCustomFieldValues(
    entityType,
    entityId
  );

  // Check update permission
  const canEdit =
    entityType === "department"
      ? hasPermission("departments:update")
      : hasPermission("employees:update");

  const startEditing = () => {
    setFormValues(buildInitialFormValues(definitions, values));
    setErrors({});
    setIsEditing(true);
  };

  const handleValueChange = (defId: string, val: unknown) => {
    setFormValues((prev) => ({ ...prev, [defId]: val as CustomFieldFormValue }));
    if (errors[defId]) {
      setErrors((prev) => {
        const next = { ...prev };
  // eslint-disable-next-line @typescript-eslint/no-dynamic-delete
        delete next[defId];
        return next;
      });
    }
  };

  const handleSave = () => {
    const nextErrors = buildValidationErrors(definitions, formValues);
    if (Object.keys(nextErrors).length > 0) {
      setErrors(nextErrors);
      toast.error("Please fill in all required custom fields.", "Validation Error");
      return;
    }

    const payload = Object.entries(formValues).map(([defId, val]) => ({
      definition_id: defId,
      value: val,
    }));

    setValues(payload, {
      onSuccess: () => {
        setIsEditing(false);
        toast.success(t("values_updated"));
      },
      onError: (err) => {
        toast.error(err instanceof Error ? err.message : "Failed to update custom fields");
      },
    });
  };

  if (isLoadingDefs || isLoadingValues) {
    return (
      <div className="flex items-center gap-2 py-6 justify-center">
        <Loader2 className="w-5 h-5 animate-spin text-primary-500" />
        <span className="text-xs font-semibold text-slate-400 dark:text-slate-500 uppercase tracking-wider">
          Loading Custom Fields...
        </span>
      </div>
    );
  }

  if (definitions.length === 0) {
    return null; // Don't show section if no fields configured
  }

  return (
    <div className="p-8 bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-3xl space-y-6 shadow-sm transition-all hover:shadow-md">
      {/* Header */}
      <div className="flex items-center justify-between border-b border-slate-50 dark:border-slate-800/50 pb-4">
        <Typography className="text-slate-800 dark:text-slate-200" variant="h3">
          {t("section_title")}
        </Typography>

        {canEdit ? (
          <div className="flex items-center gap-2">
            {isEditing ? (
              <>
                <Button
                  disabled={isSaving}
                  icon={<X className="w-3.5 h-3.5" />}
                  onClick={() => { setIsEditing(false); }}
                  size="sm"
                  variant="outline"
                >
                  {t("cancel")}
                </Button>
                <Button
                  disabled={isSaving}
                  icon={!isSaving && <Check className="w-3.5 h-3.5" />}
                  loading={isSaving}
                  onClick={handleSave}
                  size="sm"
                  variant="primary"
                >
                  Save
                </Button>
              </>
            ) : (
              <Button
                icon={<Edit2 className="w-3.5 h-3.5" />}
                onClick={startEditing}
                size="sm"
                variant="outline"
              >
                Edit
              </Button>
            )}
          </div>
        ) : null}
      </div>

      {/* Grid Content */}
      {isEditing ? (
        <div className="grid grid-cols-1 md:grid-cols-2 gap-x-8 gap-y-6">
          {definitions.map((def) => (
            <CustomFieldEditor
              definition={def}
              error={errors[def.id]}
              key={def.id}
              onChange={(val) => { handleValueChange(def.id, val); }}
              value={formValues[def.id]}
            />
          ))}
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 gap-x-8 gap-y-6">
          {definitions.map((def) => {
            const valueObj = values.find((v) => v.definition_id === def.id);
            return (
              <div className="space-y-2 px-1 py-1" key={def.id}>
                <label className="text-[10px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-widest block ml-1">
                  {def.field_label}
                </label>
                <div className="min-h-[2.5rem] flex items-center px-1">
                  {valueObj ? (
                    <CustomFieldRenderer value={valueObj} />
                  ) : (
                    <span className="text-slate-400 dark:text-slate-600 italic text-xs">
                      —
                    </span>
                  )}
                </div>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
};
