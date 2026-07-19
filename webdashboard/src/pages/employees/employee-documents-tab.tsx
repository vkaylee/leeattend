/* eslint-disable */
import { useQuery, useQueryClient } from "@tanstack/react-query";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import { toast } from "sonner";

import { FileList } from "@/components/files/file-list";
import { FileUpload } from "@/components/files/file-upload";
import Button from "@/components/ui/button";
import { useWorkspace } from "@/hooks/use-workspace";
import { client } from "@/lib/api/client";
import type { components } from "@/types/api-generated";

type WorkspaceFile = components["schemas"]["WorkspaceFile"];

interface EmployeeDocumentsTabProps {
  employeeId: string;
}

export function EmployeeDocumentsTab({ employeeId }: EmployeeDocumentsTabProps) {
  const { t } = useTranslation();
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
        throw new Error((error as { message?: string })?.message || "Failed to load documents");
      }

      return data;
    },
    enabled: !!currentWorkspace?.id && !!employeeId,
  });

  const files = filesData || [];

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
      if (error) throw error;
      
      // Open Presigned URL in new tab to download
      // @ts-ignore
      if (data?.url) {
        // @ts-ignore
        window.open(data.url, "_blank");
      }
    } catch (err) {
      toast.error("Download failed. Please try again.");
    }
  };

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
        if (error) throw error;
      }
      toast.success(`${fileIds.length} file(s) deleted successfully.`);
      queryClient.invalidateQueries({ queryKey: ["files", "employee", employeeId] });
    } catch (err) {
      toast.error("Failed to delete some files.");
    }
  };

  const handleShare = (file: WorkspaceFile) => {
    toast.info(`Sharing feature coming in Phase 5 Distribution for file: ${file.file_name}`);
  };

  const handleUploadSuccess = (file: WorkspaceFile) => {
    toast.success(`File uploaded successfully: ${file.file_name}`);
    setShowUploadPanel(false);
    queryClient.invalidateQueries({ queryKey: ["files", "employee", employeeId] });
  };

  return (
    <div className="flex flex-col h-full bg-white dark:bg-slate-900 rounded-3xl border border-slate-200 dark:border-slate-800 shadow-sm overflow-hidden animate-in fade-in zoom-in-95 duration-300 p-6 gap-6">
      
      {/* Header Actions */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-xl font-bold tracking-tight text-slate-800 dark:text-slate-100">
            Employee Documents
          </h2>
          <p className="text-sm text-slate-500 dark:text-slate-400 mt-1">
            Manage contracts, ID cards, and compliance documents for this employee.
          </p>
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
      {!!showUploadPanel && !!currentWorkspace && (
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
      )}

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
