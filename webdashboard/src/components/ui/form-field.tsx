import React, { memo } from "react";

import { cn } from "@/lib/utils";

// ── Shared input class ─────────────────────────────────────────────

const INPUT_BASE =
  "w-full px-6 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all shadow-sm";

const LABEL_BASE =
  "text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1";

// ── Component ──────────────────────────────────────────────────────

export interface FormFieldProps {
  /** Wrapper className */
  className?: string;
  "data-testid"?: string;
  defaultValue?: string;
  disabled?: boolean;
  /** Error message to display */
  error?: string;
  /** HTML id */
  id?: string;
  /** Extra className for the input element (e.g. "font-mono") */
  inputClassName?: string;
  label: string;
  /** HTML name attribute */
  name?: string;
  onBlur?: React.FocusEventHandler<HTMLInputElement | HTMLTextAreaElement>;
  onChange?: (
    e: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement>,
  ) => void;
  placeholder?: string;
  required?: boolean;
  /** Textarea rows (default: 5) */
  rows?: number;
  /** Input type — "textarea" renders a <textarea> */
  type?: "text" | "date" | "time" | "tel" | "email" | "number" | "textarea";
  value?: string;
}

const FormField = memo(
  React.forwardRef<HTMLInputElement | HTMLTextAreaElement, FormFieldProps>(
    function FormField(
      {
        label,
        name,
        type = "text",
        required,
        defaultValue,
        value,
        onChange,
        onBlur,
        placeholder,
        inputClassName,
        id,
        "data-testid": testId,
        rows = 5,
        className,
        disabled,
        error,
      },
      ref,
    ) {
      const inputId = id ?? name;

      const sharedProps = {
        id: inputId,
        name,
        required,
        defaultValue,
        value,
        onChange,
        onBlur,
        placeholder,
        "data-testid": testId,
        disabled,
      };

      return (
        <div className={cn("space-y-3 px-1", className)}>
          <label className={LABEL_BASE} htmlFor={inputId}>
            {label}
          </label>
          {type === "textarea" ? (
            <textarea
              ref={ref as React.Ref<HTMLTextAreaElement>}
              {...sharedProps}
              className={cn(
                INPUT_BASE,
                "py-4 font-medium text-sm resize-none",
                inputClassName,
              )}
              onChange={onChange}
              rows={rows}
            />
          ) : (
            <input
              ref={ref as React.Ref<HTMLInputElement>}
              {...sharedProps}
              className={cn(
                INPUT_BASE,
                "h-[var(--ui-height-md)] font-medium text-sm",
                inputClassName,
              )}
              onChange={onChange}
              type={type}
            />
          )}
          {!!error && (
            <p className="text-xs text-rose-500 font-medium px-1 mt-1">{error}</p>
          )}
        </div>
      );
    },
  ),
);

export { INPUT_BASE, LABEL_BASE };
export default FormField;

