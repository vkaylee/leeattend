import React from "react";
import { Link2, Mail, Phone, Check, X } from "lucide-react";

import type { CustomFieldValueResponse } from "@/lib/api/types";

interface CustomFieldRendererProps {
  className?: string;
  value: CustomFieldValueResponse;
}

export const CustomFieldRenderer: React.FC<CustomFieldRendererProps> = ({
  value,
  className = "",
}) => {
  const val = value.value;

  if (val === null || val === undefined || val === "") {
    return <span className="text-slate-400 dark:text-slate-600 italic text-xs">—</span>;
  }

  // Helper to format option labels for select/multiselect (if options metadata is somehow enriched,
  // but usually values.field_label or formatting is handled on client).
  // In our schema, the definition is not fully passed inside value, but we have value.field_type.

  switch (value.field_type) {
    case "boolean": {
      const boolVal = typeof val === "boolean" ? val : val === "true";
      return (
        <span
          className={`inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-bold uppercase tracking-wider ${
            boolVal
              ? "bg-emerald-50 dark:bg-emerald-950/30 text-emerald-600 dark:text-emerald-400"
              : "bg-rose-50 dark:bg-rose-950/30 text-rose-600 dark:text-rose-400"
          }`}
        >
          {boolVal ? (
            <>
              <Check className="w-3.5 h-3.5" />
              Yes
            </>
          ) : (
            <>
              <X className="w-3.5 h-3.5" />
              No
            </>
          )}
        </span>
      );
    }

    case "url": {
      const href = String(val).startsWith("http") ? String(val) : `https://${val}`;
      return (
        <a
          className="inline-flex items-center gap-1.5 text-primary-600 dark:text-primary-400 hover:text-primary-800 dark:hover:text-primary-300 font-medium hover:underline text-sm break-all"
          href={href}
          rel="noopener noreferrer"
          target="_blank"
        >
          <Link2 className="w-3.5 h-3.5 shrink-0" />
          {String(val)}
        </a>
      );
    }

    case "email":
      return (
        <a
          className="inline-flex items-center gap-1.5 text-primary-600 dark:text-primary-400 hover:text-primary-800 dark:hover:text-primary-300 font-medium hover:underline text-sm break-all"
          href={`mailto:${val}`}
        >
          <Mail className="w-3.5 h-3.5 shrink-0" />
          {String(val)}
        </a>
      );

    case "phone":
      return (
        <a
          className="inline-flex items-center gap-1.5 text-primary-600 dark:text-primary-400 hover:text-primary-800 dark:hover:text-primary-300 font-medium hover:underline text-sm break-all"
          href={`tel:${val}`}
        >
          <Phone className="w-3.5 h-3.5 shrink-0" />
          {String(val)}
        </a>
      );

    case "multiselect": {
      try {
        const list = Array.isArray(val)
          ? val
          : typeof val === "string"
            ? JSON.parse(val)
            : [];
        if (!Array.isArray(list) || list.length === 0) {
          return <span className="text-slate-400 dark:text-slate-600 italic text-xs">—</span>;
        }
        return (
          <div className="flex flex-wrap gap-1.5 max-w-full">
            {list.map((item, idx) => (
              <span
                className="inline-flex items-center px-2.5 py-1 bg-slate-100 dark:bg-slate-800 border border-slate-200/50 dark:border-slate-700/50 text-slate-700 dark:text-slate-300 text-xs font-bold uppercase rounded-lg tracking-tight"
                key={idx}
              >
                {String(item)}
              </span>
            ))}
          </div>
        );
      } catch {
        return <span className="text-slate-700 dark:text-slate-300 text-sm">{String(val)}</span>;
      }
    }

    case "number":
      return (
        <span className="text-slate-800 dark:text-slate-200 font-bold text-sm">
          {Number(val).toLocaleString()}
        </span>
      );

    default:
      return (
        <span className="text-slate-800 dark:text-slate-200 font-medium text-sm break-words whitespace-pre-wrap">
          {String(val)}
        </span>
      );
  }
};
