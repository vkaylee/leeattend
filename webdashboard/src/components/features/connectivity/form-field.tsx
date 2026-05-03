import React from "react";

interface FormFieldProps {
  children: React.ReactNode;
  label: string;
}

export function FormField({ label, children }: FormFieldProps) {
  return (
    <div className="space-y-3 px-1">
      <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{label}</label>
      {children}
    </div>
  );
}
