import React from "react";
import PageHeader from "@/components/ui/page-header";
import { TemplateEditor } from "@/components/documents/template-editor";
import { toast } from "sonner";

export function DocumentTemplatesPage() {
  return (
    <div className="flex flex-col h-full gap-6">
      <PageHeader
        title="Document Templates"
        subtitle="Create and manage rich-text templates with dynamic variables."
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
