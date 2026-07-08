import { Link2, Mail, Phone, Check, X } from "lucide-react";
import React from "react";

import type { CustomFieldValueResponse } from "@/lib/api/types";

interface CustomFieldRendererProps {
  className?: string;
  value: CustomFieldValueResponse;
}

function renderBoolean(val: unknown) {
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

function renderLink(val: unknown, icon: React.ReactNode, href: string) {
  return (
    <a
      className="inline-flex items-center gap-1.5 text-primary-600 dark:text-primary-400 hover:text-primary-800 dark:hover:text-primary-300 font-medium hover:underline text-sm break-all"
      href={href}
      rel="noopener noreferrer"
      target="_blank"
    >
      {icon}
      {String(val)}
    </a>
  );
}

function parseMultiselectList(val: unknown) {
  try {
    const parsed: unknown = Array.isArray(val)
      ? val
      : typeof val === "string"
        ? JSON.parse(val)
        : [];
    return { list: Array.isArray(parsed) ? parsed : [], isParseError: !Array.isArray(parsed) };
  } catch {
    return { list: [] as unknown[], isParseError: true };
  }
}

function renderMultiselect(val: unknown) {
  const { list, isParseError } = parseMultiselectList(val);

  if (isParseError) {
    return <span className="text-slate-700 dark:text-slate-300 text-sm">{String(val)}</span>;
  }
  if (list.length === 0) {
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
}

const FIELD_TYPE_RENDERERS: Record<string, (val: unknown) => React.ReactNode> = {
  boolean: renderBoolean,
  url: (val) => {
    const href = String(val).startsWith("http") ? String(val) : `https://${val}`;
    return renderLink(val, <Link2 className="w-3.5 h-3.5 shrink-0" />, href);
  },
  email: (val) => renderLink(val, <Mail className="w-3.5 h-3.5 shrink-0" />, `mailto:${val}`),
  phone: (val) => renderLink(val, <Phone className="w-3.5 h-3.5 shrink-0" />, `tel:${val}`),
  multiselect: renderMultiselect,
  number: (val) => (
    <span className="text-slate-800 dark:text-slate-200 font-bold text-sm">
      {Number(val).toLocaleString()}
    </span>
  ),
};

export const CustomFieldRenderer: React.FC<CustomFieldRendererProps> = ({
  value,
}) => {
  const val = value.value;

  if (val === null || val === undefined || val === "") {
    return <span className="text-slate-400 dark:text-slate-600 italic text-xs">—</span>;
  }

  const renderer = FIELD_TYPE_RENDERERS[value.field_type];
  if (renderer) {
    return <>{renderer(val)}</>;
  }

  return (
    <span className="text-slate-800 dark:text-slate-200 font-medium text-sm break-words whitespace-pre-wrap">
      {String(val)}
    </span>
  );
};
