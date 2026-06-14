import React from "react";
import PageHeader from "@/components/ui/page-header";

export function ESSDocumentsPage() {
  return (
    <div className="flex flex-col h-full gap-6">
      <PageHeader
        title="My Documents"
        subtitle="View and acknowledge documents requiring your signature."
      />
      <div className="flex-1 bg-white dark:bg-slate-900 rounded-3xl border border-slate-200 dark:border-slate-800 p-6">
        {/* We will build Document Acknowledgement View here */}
        <p className="text-slate-500">Employee Documents (ESS) Interface</p>
      </div>
    </div>
  );
}
