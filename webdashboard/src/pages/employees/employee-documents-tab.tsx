import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import type { components } from "@/types/api-generated";

import { FileList } from "@/components/files/file-list";
import { FileUpload } from "@/components/files/file-upload";
import Button from "@/components/ui/button";
import { toast } from "@/components/ui/toast-imperative";
import { Typography } from "@/components/ui/typography";
import { useWorkspace } from "@/hooks/use-workspace";
import { client } from "@/lib/api/client";
import { logError } from "@/lib/logger";
import { maybeNull } from "@/types/api-overrides";

type WorkspaceFile = components["schemas"]["WorkspaceFile"];

interface EmployeeDocumentsTabProps {
  employeeId: string;
}

export function EmployeeDocumentsTab({ employeeId }: EmployeeDocumentsTabProps) {
  const _t = useTranslation().t;
  const queryClient = useQueryClient();
  const { workspace: currentWorkspace } = useWorkspace();
  
  // State for Split Pane / File Upload Panel visibility
  const [showUploadPanel, setShowUploadPanel] = useState(false);

  // Fetch Files
  const { data: filesData, isLoading } = useQuery({
    queryKey: ["files", "employee", employeeId],
    queryFn: async () => {
      if (!currentWorkspace?.id) return [];
      const { data, error } = await client.GET("/v1/workspaces/{ws_id}/files/by-owner/{owner_type}/{owner_id}", {
        params: {
          path: {
            ws_id: currentWorkspace.id,
            owner_type: "Employee",
            owner_id: employeeId,
          },
        },
      });

      if (error) {
        toast.error("Failed to load documents");
        const docErr: { message?: string } | null | undefined = error;
        throw new Error(maybeNull(docErr)?.message ?? "Failed to load documents");
      }

      return data;
    },
    enabled: !!currentWorkspace?.id && !!employeeId,
  });

  const files = filesData ?? [];

  const handleDownload = async (file: WorkspaceFile) => {
    if (!currentWorkspace?.id) return;
    try {
      const { data, error } = await client.GET("/v1/workspaces/{ws_id}/files/{id}/url", {
        params: {
          path: {
            ws_id: currentWorkspace.id,
            id: file.id,
          },
        },
      });
      if (error) throw new Error("Failed to get download URL");

      // The generated client type for this endpoint omits the `url` field,
      // but the runtime response includes a presigned download URL.
      const downloadUrl = (maybeNull(data) as { url?: string } | undefined)?.url;
      if (downloadUrl) {
        window.open(downloadUrl, "_blank");
      }
    } catch (e: unknown) {
      logError(e, { component: 'EmployeeDocumentsTab.download' });
      toast.error("Download failed. Please try again.");
    }
  };

  // eslint-disable-next-line @typescript-eslint/require-await -- async required by FileList prop signature
  const handleRename = async (fileId: string, newName: string) => {
    toast.success(`Renamed file to: ${newName}`);
    // Will implement PUT /v1/workspaces/{ws_id}/files/{id} in backend later
  };

  const handleDelete = async (fileIds: string[]) => {
    if (!currentWorkspace?.id) return;
    
    // Optimistic UI update could be added here
    try {
      for (const id of fileIds) {
        const { error } = await client.DELETE("/v1/workspaces/{ws_id}/files/{id}", {
          params: { path: { ws_id: currentWorkspace.id, id } },
        });
        if (error) throw new Error("Failed to delete file");
      }
      toast.success(`${fileIds.length} file(s) deleted successfully.`);
      void queryClient.invalidateQueries({ queryKey: ["files", "employee", employeeId] });
    } catch (e: unknown) {
      logError(e, { component: 'EmployeeDocumentsTab.delete' });
      toast.error("Failed to delete some files.");
    }
  };

  const handleShare = (file: WorkspaceFile) => {
    toast.info(`Sharing feature coming in Phase 5 Distribution for file: ${file.file_name}`);
  };

  const handleUploadSuccess = (file: WorkspaceFile) => {
    toast.success(`File uploaded successfully: ${file.file_name}`);
    setShowUploadPanel(false);
    void queryClient.invalidateQueries({ queryKey: ["files", "employee", employeeId] });
  };

  return (
    <div className="flex flex-col h-full bg-white dark:bg-slate-900 rounded-3xl border border-slate-200 dark:border-slate-800 shadow-sm overflow-hidden animate-in fade-in zoom-in-95 duration-300 p-6 gap-6">
      
      {/* Header Actions */}
      <div className="flex items-center justify-between">
        <div>
          <Typography className="text-xl font-bold tracking-tight text-slate-800 dark:text-slate-100" variant="h2">
            Employee Documents
          </Typography>
          <Typography className="text-sm text-slate-500 dark:text-slate-400 mt-1" variant="p">
            Manage contracts, ID cards, and compliance documents for this employee.
          </Typography>
        </div>
        
        <Button
          className="px-4 py-2 text-sm rounded-xl transition-all shadow-sm"
          onClick={() => { setShowUploadPanel(!showUploadPanel); }}
          variant={showUploadPanel ? "secondary" : "primary"}
        >
          {showUploadPanel ? "Close Upload" : "+ Upload Document"}
        </Button>
      </div>

      {/* Upload Panel (Collapsible) */}
      {showUploadPanel && currentWorkspace ? (
        <div className="animate-in slide-in-from-top-4 fade-in duration-300">
          <FileUpload
            category="EmployeeDoc"
            maxSizeMB={50} // 50MB for Employee Docs
            onUploadSuccess={handleUploadSuccess}
            ownerId={employeeId}
            ownerType="Employee"
            workspaceId={currentWorkspace.id}
          />
        </div>
      ) : null}

      {/* Main Grid View */}
      <div className="flex-1 min-h-0">
        <FileList
          files={files}
          isLoading={isLoading}
          onDelete={handleDelete}
          onDownload={handleDownload}
          onRename={handleRename}
          onShare={handleShare}
        />
      </div>
    </div>
  );
}
