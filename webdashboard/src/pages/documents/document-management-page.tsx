import React, { useState } from "react";

import { FileGrid } from "@/components/documents/file-grid";
import { FolderTree } from "@/components/documents/folder-tree";
import PageHeader from "@/components/ui/page-header";
import { useFolders } from "@/hooks/use-documents";
import { useWorkspace } from "@/hooks/use-workspace";
import { components } from "@/types/api-generated";

type WorkspaceFile = components["schemas"]["WorkspaceFile"];

export function DocumentManagementPage() {
  const { workspace } = useWorkspace();
  const { data: folders = [] } = useFolders(workspace?.id);
  const [selectedFolderId, setSelectedFolderId] = useState<string | null>(null);

  // Use correct type definition instead of any[]
  const files: WorkspaceFile[] = [];

  return (
    <div className="flex flex-col h-full gap-6">
      <PageHeader
        subtitle="Manage folders, files, and distribute documents to employees."
        title="Document Management"
      />
      <div className="flex-1 flex gap-6 min-h-0">
        <div className="w-64 bg-white dark:bg-slate-900 rounded-3xl border border-slate-200 dark:border-slate-800 p-4 overflow-y-auto shrink-0 shadow-sm">
          <FolderTree
            folders={folders}
            onSelectFolder={setSelectedFolderId}
            selectedFolderId={selectedFolderId}
          />
        </div>
        <div className="flex-1 bg-white dark:bg-slate-900 rounded-3xl border border-slate-200 dark:border-slate-800 p-6 overflow-y-auto shadow-sm">
          <FileGrid
            files={files}
            onDeleteFile={() => {}}
            onDownloadFile={() => {}}
            onViewFile={() => {}}
          />
        </div>
      </div>
    </div>
  );
}
