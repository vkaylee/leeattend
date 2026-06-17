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

interface CustomFieldEditorProps {
  definition: CustomFieldDefinition;
  error?: string;
  onChange: (value: unknown) => void;
  value: unknown;
}

interface MultiselectFieldEditorProps {
  definition: CustomFieldDefinition;
  error?: string;
  onChange: (value: unknown) => void;
  options: { value: string; label: string }[];
  value: unknown;
}

const MultiselectFieldEditor: React.FC<MultiselectFieldEditorProps> = ({
  definition,
  value,
  onChange,
  options,
  error,
}) => {
  const getSelectedValues = (): string[] => {
    if (Array.isArray(value)) {
      return value.map(String);
    }
    if (typeof value === "string") {
      try {
        const parsed = JSON.parse(value);
        if (Array.isArray(parsed)) return parsed.map(String);
      } catch {
        return value ? [value] : [];
      }
    }
    return [];
  };

  const selected = getSelectedValues();

  const toggleOption = (optValue: string) => {
    const next = selected.includes(optValue)
      ? selected.filter((v) => v !== optValue)
      : [...selected, optValue];
    onChange(next);
  };

  return (
    <div className="flex flex-col gap-3 px-1">
      <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">
        {definition.field_label}
        {definition.is_required ? <span className="text-rose-500 ml-0.5">*</span> : null}
      </label>

      <Popover.Root>
        <Popover.Trigger asChild>
          <Button
            className="flex min-h-[var(--ui-height-md)] w-full items-center justify-between rounded-2xl border border-slate-100 dark:border-slate-700 bg-slate-50 dark:bg-slate-800 px-6 py-3.5 text-sm font-bold text-primary-600 dark:text-primary-400 transition-all hover:border-primary-200 dark:hover:border-primary-800 hover:bg-white dark:hover:bg-slate-700 focus:outline-none focus:ring-4 focus:ring-primary-500/10 shadow-inner uppercase tracking-tight h-auto"
            type="button"
            variant="ghost"
          >
            <div className="flex flex-wrap gap-1.5 truncate max-w-[90%]">
              {selected.length > 0 ? (
                selected.map((val) => {
                  const label = options.find((o) => o.value === val)?.label ?? val;
                  return (
                    <span
                      className="inline-flex items-center gap-1 px-2.5 py-0.5 bg-primary-100 dark:bg-primary-950 text-primary-700 dark:text-primary-300 text-xs font-bold uppercase rounded-lg border border-primary-200/50 dark:border-primary-900/50 tracking-tight"
                      key={val}
                    >
                      {label}
                      <X
                        className="w-3.5 h-3.5 hover:text-primary-900 dark:hover:text-white shrink-0 cursor-pointer"
                        onClick={(e) => {
                          e.stopPropagation();
                          toggleOption(val);
                        }}
                      />
                    </span>
                  );
                })
              ) : (
                <span className="text-primary-400 font-bold opacity-60">
                  Select options
                </span>
              )}
            </div>
            <ChevronDown className="ml-2 h-4 w-4 shrink-0 text-slate-300" />
          </Button>
        </Popover.Trigger>

        <Popover.Portal>
          <Popover.Content
            className="z-[10000] w-[var(--radix-popover-trigger-width)] min-w-[240px] overflow-hidden rounded-[2rem] border border-slate-100 dark:border-slate-700 bg-white dark:bg-slate-900 p-2.5 shadow-2xl animate-in fade-in zoom-in-95 duration-300"
            sideOffset={8}
          >
            <div className="max-h-[280px] overflow-y-auto p-1 custom-scrollbar space-y-1">
              {options.length > 0 ? (
                options.map((option) => {
                  const isSelected = selected.includes(option.value);
                  return (
                    <Button
                      className={cn(
                        "relative flex w-full cursor-pointer select-none items-center rounded-2xl px-4 py-3.5 text-sm font-bold text-left outline-none transition-all hover:bg-primary-50 dark:hover:bg-primary-950 hover:text-primary-600 dark:hover:text-primary-300 uppercase tracking-tight h-auto justify-start border-0 shadow-none normal-case",
                        isSelected &&
                          "bg-primary-50 dark:bg-primary-950 text-primary-600 dark:text-primary-300 shadow-inner"
                      )}
                      key={option.value}
                      onClick={() => { toggleOption(option.value); }}
                      type="button"
                      variant="ghost"
                    >
                      <span>{option.label}</span>
                      {isSelected ? (
                        <Check className="ml-auto h-4 w-4 text-primary-600 dark:text-primary-300" />
                      ) : null}
                    </Button>
                  );
                })
              ) : (
                <div className="py-6 text-center text-xs font-bold text-slate-400 dark:text-slate-500 uppercase tracking-wider">
                  No options configured
                </div>
              )}
            </div>
          </Popover.Content>
        </Popover.Portal>
      </Popover.Root>
      {!!error && (
        <Typography className="text-xs text-rose-500 font-medium px-1 mt-1" variant="p">
          {error}
        </Typography>
      )}
    </div>
  );
};

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
