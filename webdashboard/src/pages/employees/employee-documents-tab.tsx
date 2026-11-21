import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import type { FileRecord } from "@/lib/api/types";

import { FileList } from "@/components/files/file-list";
import { FileUpload } from "@/components/files/file-upload";
import Button from "@/components/ui/button";
import { toast } from "@/components/ui/toast-imperative";
import { Typography } from "@/components/ui/typography";
import { client } from "@/lib/api/client";
import { logError } from "@/lib/logger";
import { createQueryKey } from "@/lib/query-keys";
import { maybeNull } from "@/types/api-overrides";


interface EmployeeDocumentsTabProps {
  employeeId: string;
}

export function EmployeeDocumentsTab({ employeeId }: EmployeeDocumentsTabProps) {
  const _t = useTranslation().t;
  const queryClient = useQueryClient();

  // State for Split Pane / File Upload Panel visibility
  const [showUploadPanel, setShowUploadPanel] = useState(false);

  // Fetch Files
  const { data: filesData, isLoading } = useQuery({
    queryKey: createQueryKey("files", "employee", employeeId),
    queryFn: async () => {
      const { data } = await client.GET<FileRecord[]>('/api/v1/files/by-owner/{owner_type}/{owner_id}', {
        params: {
          path: {
            owner_type: 'Employee',
            owner_id: employeeId,
          },
        },
      });
      return data;
    },
    enabled: !!employeeId,
  });

  const files: FileRecord[] = Array.isArray(filesData) ? filesData : [];

  const handleDownload = async (file: FileRecord) => {
    try {
      const { data } = await client.GET<{ url?: string }>('/api/v1/files/{id}/url', {
        params: {
          path: { id: file.id },
        },
      });

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

  const handleDelete = async (fileIds: string[]) => {
    // Optimistic UI update could be added here
    try {
      for (const id of fileIds) {
        await client.DELETE('/api/v1/files/{id}', {
          params: { path: { id } },
        });
      }
      toast.success(`${fileIds.length} file(s) deleted successfully.`);
      void queryClient.invalidateQueries({ queryKey: createQueryKey("files", "employee", employeeId) });
    } catch (e: unknown) {
      logError(e, { component: 'EmployeeDocumentsTab.delete' });
      toast.error("Failed to delete some files.");
    }
  };

  const handleShare = (file: FileRecord) => {
    toast.info(`Sharing feature coming in Phase 5 Distribution for file: ${file.file_name}`);
  };

  const handleUploadSuccess = (file: FileRecord) => {
    toast.success(`File uploaded successfully: ${file.file_name}`);
    setShowUploadPanel(false);
    void queryClient.invalidateQueries({ queryKey: createQueryKey("files", "employee", employeeId) });
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
      {showUploadPanel ? (
        <div className="animate-in slide-in-from-top-4 fade-in duration-300">
          <FileUpload category="EmployeeDoc"
          maxSizeMB={50} // 50MB for Employee Docs
          onUploadSuccess={handleUploadSuccess}
          ownerId={employeeId}
          ownerType="Employee"  />
        </div>
      ) : null}

      {/* Main Grid View */}
      <div className="flex-1 min-h-0">
        <FileList
          files={files}
          isLoading={isLoading}
          onDelete={handleDelete}
          onDownload={handleDownload}
          onShare={handleShare}
        />
      </div>
    </div>
  );
}
