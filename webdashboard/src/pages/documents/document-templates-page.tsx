import React from "react";

import { toast } from "sonner";

import { TemplateEditor } from "@/components/documents/template-editor";
import PageHeader from "@/components/ui/page-header";

export function DocumentTemplatesPage() {
  return (
    <div className="flex flex-col h-full gap-6">
      <PageHeader
        subtitle="Create and manage rich-text templates with dynamic variables."
        title="Document Templates"
      />
      <div className="flex-1 min-h-0">
        <TemplateEditor 
          onSave={(content) => {
            toast.success("Template saved successfully");
            console.warn("Saving template:", content);
          }}
        />
      </div>
    </div>
  );
}
