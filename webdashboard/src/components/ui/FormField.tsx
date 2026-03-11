import React from "react";
import { cn } from "@/lib/utils";

// ── Shared input class ─────────────────────────────────────────────

const INPUT_BASE =
  "w-full px-6 py-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all shadow-sm";

const LABEL_BASE =
  "text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1";

// ── Component ──────────────────────────────────────────────────────

export interface FormFieldProps {
  label: string;
  /** HTML name attribute */
  name?: string;
  /** Input type — "textarea" renders a <textarea> */
  type?: "text" | "date" | "tel" | "email" | "number" | "textarea";
  required?: boolean;
  defaultValue?: string;
  value?: string;
  onChange?: (
    e: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement>,
  ) => void;
  placeholder?: string;
  /** Extra className for the input element (e.g. "font-mono") */
  inputClassName?: string;
  /** HTML id */
  id?: string;
  "data-testid"?: string;
  /** Textarea rows (default: 5) */
  rows?: number;
  /** Wrapper className */
  className?: string;
}

const FormField = React.memo(function FormField({
  label,
  name,
  type = "text",
  required,
  defaultValue,
  value,
  onChange,
  placeholder,
  inputClassName,
  id,
  "data-testid": testId,
  rows = 5,
  className,
}: FormFieldProps) {
  const inputId = id || name;

  const sharedProps = {
    id: inputId,
    name,
    required,
    defaultValue,
    value,
    onChange,
    placeholder,
    "data-testid": testId,
  };

  return (
    <div className={cn("space-y-3 px-1", className)}>
      <label htmlFor={inputId} className={LABEL_BASE}>
        {label}
      </label>
      {type === "textarea" ? (
        <textarea
          {...sharedProps}
          onChange={onChange as React.ChangeEventHandler<HTMLTextAreaElement>}
          rows={rows}
          className={cn(
            INPUT_BASE,
            "font-medium text-sm resize-none",
            inputClassName,
          )}
        />
      ) : (
        <input
          {...sharedProps}
          type={type}
          onChange={onChange as React.ChangeEventHandler<HTMLInputElement>}
          className={cn(
            INPUT_BASE,
            "font-bold text-sm uppercase tracking-tight",
            inputClassName,
          )}
        />
      )}
    </div>
  );
});

export { INPUT_BASE, LABEL_BASE };
export default FormField;
