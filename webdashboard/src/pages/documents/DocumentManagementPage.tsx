import React, { useState } from "react";
import PageHeader from "@/components/ui/page-header";
import { FolderTree } from "@/components/documents/folder-tree";
import { FileGrid } from "@/components/documents/file-grid";
import { useWorkspace } from "@/hooks/use-workspace";
import { useFolders } from "@/hooks/use-documents";

export function DocumentManagementPage() {
  const { workspace } = useWorkspace();
  const { data: folders = [] } = useFolders(workspace?.id);
  const [selectedFolderId, setSelectedFolderId] = useState<string | null>(null);

  // In a real implementation we would fetch files for the selected folder
  const files: any[] = [];

  return (
    <div className="flex flex-col h-full gap-6">
      <PageHeader
        title="Document Management"
        subtitle="Manage folders, files, and distribute documents to employees."
      />
      <div className="flex-1 flex gap-6 min-h-0">
        <div className="w-64 bg-white dark:bg-slate-900 rounded-3xl border border-slate-200 dark:border-slate-800 p-4 overflow-y-auto shrink-0 shadow-sm">
          <FolderTree
            folders={folders}
            selectedFolderId={selectedFolderId}
            onSelectFolder={setSelectedFolderId}
          />
        </div>
        <div className="flex-1 bg-white dark:bg-slate-900 rounded-3xl border border-slate-200 dark:border-slate-800 p-6 overflow-y-auto shadow-sm">
          <FileGrid
            files={files}
            onViewFile={() => {}}
            onDownloadFile={() => {}}
            onDeleteFile={() => {}}
          />
        </div>
      </div>
    </div>
  );
}
