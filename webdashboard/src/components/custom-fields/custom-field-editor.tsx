import * as Popover from "@radix-ui/react-popover";
import { Check, ChevronDown, X } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import type { CustomFieldDefinition } from "@/lib/api/types";

import Button from "@/components/ui/button";
import { Combobox } from "@/components/ui/combobox";
import FormField from "@/components/ui/form-field";
import { Typography } from "@/components/ui/typography";
import { cn } from "@/lib/utils";

import { MultiselectFieldEditor } from "./multiselect-field-editor";

interface CustomFieldEditorProps {
  definition: CustomFieldDefinition;
  error?: string;
  onChange: (value: unknown) => void;
  value: unknown;
}

export const CustomFieldEditor: React.FC<CustomFieldEditorProps> = ({
  definition,
  value,
  onChange,
  error,
}) => {
  const { t } = useTranslation("custom_fields");

  // Parse options list
  const getOptions = () => {
    const opts = definition.options;
    if (!opts) return [];
    try {
      if (Array.isArray(opts)) {
        return opts.map((opt) => {
          if (opt && typeof opt === "object" && "value" in opt) {
            return {
              value: String((opt as { value: unknown }).value),
              label: String((opt as { label: unknown }).label ?? (opt as { value: unknown }).value),
            };
          }
          return { value: String(opt), label: String(opt) };
        });
      }
    } catch {
      // Ignore
    }
    return [];
  };

  const options = getOptions();

  switch (definition.field_type) {
    case "boolean": {
      const isChecked = value === true || value === "true";
      return (
        <div className="flex flex-col gap-2 px-1">
          <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">
            {definition.field_label}
            {definition.is_required ? <span className="text-rose-500 ml-0.5">*</span> : null}
          </label>
          <div className="flex items-center gap-3">
            <input
              checked={isChecked}
              className="w-7 h-7 rounded-xl text-primary-600 dark:text-primary-400 focus:ring-primary-500/20 transition-all border-slate-200 dark:border-slate-700 shadow-inner cursor-pointer"
              id={`cf-${definition.field_key}`}
              onChange={(e) => { onChange(e.target.checked); }}
              type="checkbox"
            />
            <label
              className="text-sm font-semibold text-slate-700 dark:text-slate-300 cursor-pointer select-none"
              htmlFor={`cf-${definition.field_key}`}
            >
              {isChecked ? "Yes" : "No"}
            </label>
          </div>
          {!!error && (
            <Typography className="text-xs text-rose-500 font-medium px-1 mt-1" variant="p">
              {error}
            </Typography>
          )}
        </div>
      );
    }

    case "select": {
      const comboboxOpts = options.map((opt) => ({
        value: opt.value,
        label: opt.label,
      }));
      return (
        <div className="flex flex-col gap-3 px-1">
          <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">
            {definition.field_label}
            {definition.is_required ? <span className="text-rose-500 ml-0.5">*</span> : null}
          </label>
          <Combobox
            emptyText="No options found"
            onChange={onChange}
            options={comboboxOpts}
            placeholder={`Select ${definition.field_label}`}
            searchPlaceholder="Search options..."
            value={value !== null && value !== undefined ? String(value) : ""}
          />
          {!!error && (
            <Typography className="text-xs text-rose-500 font-medium px-1 mt-1" variant="p">
              {error}
            </Typography>
          )}
        </div>
      );
    }

    case "multiselect": {
      return (
        <MultiselectFieldEditor
          definition={definition}
          error={error}
          onChange={onChange}
          options={options}
          value={value}
        />
      );
    }

    case "number":
      return (
        <FormField
          error={error}
          label={definition.field_label}
          name={definition.field_key}
          onChange={(e) => {
            const val = e.target.value;
            onChange(val === "" ? null : Number(val));
          }}
          placeholder={definition.default_value ?? "Enter number"}
          required={definition.is_required}
          type="number"
          value={value !== null && value !== undefined ? String(value) : ""}
        />
      );

    case "date":
      return (
        <FormField
          error={error}
          label={definition.field_label}
          name={definition.field_key}
          onChange={(e) => { onChange(e.target.value || null); }}
          required={definition.is_required}
          type="date"
          value={value !== null && value !== undefined ? String(value) : ""}
        />
      );

    case "email":
      return (
        <FormField
          error={error}
          label={definition.field_label}
          name={definition.field_key}
          onChange={(e) => { onChange(e.target.value || null); }}
          placeholder={definition.default_value ?? "example@domain.com"}
          required={definition.is_required}
          type="email"
          value={value !== null && value !== undefined ? String(value) : ""}
        />
      );

    case "phone":
      return (
        <FormField
          error={error}
          label={definition.field_label}
          name={definition.field_key}
          onChange={(e) => { onChange(e.target.value || null); }}
          placeholder={definition.default_value ?? "+84..."}
          required={definition.is_required}
          type="tel"
          value={value !== null && value !== undefined ? String(value) : ""}
        />
      );

    default:
      return (
        <FormField
          error={error}
          label={definition.field_label}
          name={definition.field_key}
          onChange={(e) => { onChange(e.target.value || null); }}
          placeholder={definition.default_value ?? "Enter text"}
          required={definition.is_required}
          type="text"
          value={value !== null && value !== undefined ? String(value) : ""}
        />
      );
  }
};
