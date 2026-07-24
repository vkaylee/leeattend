import React, { useMemo } from "react";

import type { CustomFieldDefinition } from "@/lib/api/types";

import { Combobox } from "@/components/ui/combobox";
import FormField from "@/components/ui/form-field";
import { Typography } from "@/components/ui/typography";

import { MultiselectFieldEditor } from "./multiselect-field-editor";

// ── Types ──────────────────────────────────────────────────────────────────

interface CustomFieldEditorProps {
  definition: CustomFieldDefinition;
  error?: string;
  onChange: (value: unknown) => void;
  value: unknown;
}

interface Option {
  label: string;
  value: string;
}

// ── Helpers ────────────────────────────────────────────────────────────────

interface OptionLike {
  label?: unknown;
  value: unknown;
}

function isOptionLike(opt: unknown): opt is OptionLike {
  return !!opt && typeof opt === "object" && "value" in opt;
}

function parseOptions(opts: unknown): Option[] {
  if (!Array.isArray(opts)) return [];
  return opts.map((opt) => {
    if (isOptionLike(opt)) {
      return {
        value: String(opt.value),
        label: String(opt.label ?? opt.value),
      };
    }
    return { value: String(opt), label: String(opt) };
  });
}

function toStringValue(value: unknown): string {
  if (value === null || value === undefined) return "";
  if (typeof value === "object") return "";
  // eslint-disable-next-line @typescript-eslint/no-base-to-string -- objects are handled above; remaining primitives stringify safely
  return String(value);
}

// ── Shared Sub-Components ──────────────────────────────────────────────────

function RequiredMark({ isRequired }: { isRequired: boolean }) {
  if (!isRequired) return null;
  return <span className="text-rose-500 ml-0.5">*</span>;
}

function FieldError({ error }: { error?: string }) {
  if (!error) return null;
  return (
    <Typography className="text-xs text-rose-500 font-medium px-1 mt-1" variant="p">
      {error}
    </Typography>
  );
}

function LabelRow({ definition, htmlFor }: { definition: CustomFieldDefinition; htmlFor?: string }) {
  return (
    <label
      className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1"
      htmlFor={htmlFor}
    >
      {definition.field_label}
      <RequiredMark isRequired={definition.is_required} />
    </label>
  );
}

// ── Field-Type Editors ─────────────────────────────────────────────────────

function BooleanEditor({ definition, value, onChange, error }: CustomFieldEditorProps) {
  const isChecked = value === true || value === "true";
  const fieldId = `cf-${definition.field_key}`;
  return (
    <div className="flex flex-col gap-2 px-1">
      <LabelRow definition={definition} htmlFor={fieldId} />
      <div className="flex items-center gap-3">
        <input
          checked={isChecked}
          className="w-7 h-7 rounded-xl text-primary-600 dark:text-primary-400 focus:ring-primary-500/20 transition-all border-slate-200 dark:border-slate-700 shadow-inner cursor-pointer"
          id={fieldId}
          onChange={(e) => { onChange(e.target.checked); }}
          type="checkbox"
        />
        <label
          className="text-sm font-semibold text-slate-700 dark:text-slate-300 cursor-pointer select-none"
          htmlFor={fieldId}
        >
          {isChecked ? "Yes" : "No"}
        </label>
      </div>
      <FieldError error={error} />
    </div>
  );
}

function SelectEditor({ definition, value, onChange, error }: CustomFieldEditorProps) {
  const options = useMemo(() => parseOptions(definition.options), [definition.options]);
  const comboboxOpts = useMemo(
    () => options.map((opt) => ({ value: opt.value, label: opt.label })),
    [options],
  );
  return (
    <div className="flex flex-col gap-3 px-1">
      <LabelRow definition={definition} />
      <Combobox
        emptyText="No options found"
        onChange={onChange}
        options={comboboxOpts}
        placeholder={`Select ${definition.field_label}`}
        searchPlaceholder="Search options..."
        value={toStringValue(value)}
      />
      <FieldError error={error} />
    </div>
  );
}

interface FormFieldEditorConfig {
  placeholder: string;
  type: "date" | "email" | "number" | "tel" | "text";
}

const FIELD_TYPE_FORM_CONFIG: Record<string, FormFieldEditorConfig> = {
  date: { type: "date", placeholder: "" },
  email: { type: "email", placeholder: "example@domain.com" },
  number: { type: "number", placeholder: "Enter number" },
  phone: { type: "tel", placeholder: "+84..." },
};

function FormFieldEditor({ definition, value, onChange, error }: CustomFieldEditorProps) {
  const config = FIELD_TYPE_FORM_CONFIG[definition.field_type] ?? { type: "text" as const, placeholder: "Enter text" };
  return (
    <FormField
      error={error}
      label={definition.field_label}
      name={definition.field_key}
      onChange={(e) => {
        const val = e.target.value;
        if (config.type === "number") {
          onChange(val === "" ? null : Number(val));
        } else {
          onChange(val || null);
        }
      }}
      placeholder={definition.default_value ?? config.placeholder}
      required={definition.is_required}
      type={config.type}
      value={toStringValue(value)}
    />
  );
}

// ── Main Component ─────────────────────────────────────────────────────────

export const CustomFieldEditor: React.FC<CustomFieldEditorProps> = (props) => {
  const { definition } = props;

  switch (definition.field_type) {
    case "boolean":
      return <BooleanEditor {...props} />;
    case "select":
      return <SelectEditor {...props} />;
    case "multiselect":
      return <MultiselectFieldEditor {...props} options={parseOptions(definition.options)} />;
    case "text":
    case "number":
    case "date":
    case "email":
    case "phone":
      return <FormFieldEditor {...props} />;
    default:
      return <FormFieldEditor {...props} />;
  }
};
