import React from "react";

import { TemplateEditor } from "@/components/documents/template-editor";
import PageHeader from "@/components/ui/page-header";
import { toast } from "@/components/ui/toast";

export function DocumentTemplatesPage() {
  return (
    <div className="flex flex-col h-full gap-6">
      <PageHeader
        subtitle="Create and manage rich-text templates with dynamic variables."
        title="Document Templates"
      />
      <div className="flex-1 min-h-0">
        <TemplateEditor 
          onSave={(_content) => {
            toast.success("Template saved successfully");
          }}
        />
      </div>
    </div>
  );
}
