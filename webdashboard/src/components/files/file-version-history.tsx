import { useQuery } from "@tanstack/react-query";
import { History, ArrowDownToLine, CheckCircle2 } from "lucide-react";
import React from "react";

import { format } from "date-fns";
import { toast } from "sonner";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import { fetch } from "@/lib/api/client";
import { formatBytes } from "@/lib/utils";
import { components } from "@/types/api-generated";

type WorkspaceFile = components["schemas"]["WorkspaceFile"];

interface FileVersionHistoryProps {
  className?: string;
  fileId: string;
  workspaceId: string;
}

interface VersionTimelineItemProps {
  onDownload: (id: string) => void;
  version: WorkspaceFile;
}

function LoadingSkeleton({ className = "" }: { className?: string }) {
  return (
    <div className={`flex flex-col gap-4 animate-pulse p-4 ${className}`}>
      {[1, 2, 3].map(i => (
        <div className="flex gap-4" key={i}>
          <div className="w-2 h-20 bg-slate-200 dark:bg-slate-800 rounded-full" />
          <div className="flex-1 space-y-2 py-1">
            <div className="h-4 bg-slate-200 dark:bg-slate-800 rounded w-1/4" />
            <div className="h-3 bg-slate-100 dark:bg-slate-900 rounded w-1/2" />
          </div>
        </div>
      ))}
    </div>
  );
}

function VersionTimelineItem({ version, onDownload }: VersionTimelineItemProps) {
  const isLatest = version.is_latest;
  
  return (
    <div className="relative flex gap-5 group">
      {/* Timeline Dot */}
      <div className={`
        relative z-10 w-3 h-3 mt-1.5 rounded-full shrink-0 border-2 transition-colors
        ${isLatest 
          ? "bg-primary-500 border-white dark:border-slate-900 shadow-[0_0_0_4px_rgba(59,130,246,0.15)]" 
          : "bg-slate-300 dark:bg-slate-600 border-white dark:border-slate-900 group-hover:bg-primary-300"
        }
      `} />

      {/* Content Card */}
      <div className={`
        flex-1 flex items-start justify-between gap-4 p-4 rounded-2xl border transition-all duration-200
        ${isLatest 
          ? "bg-white dark:bg-slate-900 border-primary-100 dark:border-primary-900/50 shadow-sm shadow-primary-500/5" 
          : "bg-slate-50 dark:bg-slate-900/50 border-slate-100 dark:border-slate-800 hover:bg-white dark:hover:bg-slate-900"
        }
      `}>
        <div>
          <div className="flex items-center gap-2 mb-1">
            <span className="font-bold text-slate-700 dark:text-slate-200 text-sm">
              Version {version.version}
            </span>
            {isLatest ? (
              <span className="flex items-center gap-1 px-1.5 py-0.5 rounded-md bg-primary-50 dark:bg-primary-950/30 text-primary-600 dark:text-primary-400 text-[9px] font-black tracking-widest uppercase">
                <CheckCircle2 className="w-3 h-3" /> Current
              </span>
            ) : null}
          </div>
          
          <Typography className="text-xs text-slate-500 dark:text-slate-400 font-medium mb-2 mt-0" variant="small">
            {format(new Date(version.created_at), "MMM dd, yyyy 'at' HH:mm a")}
          </Typography>

          <div className="flex items-center gap-3">
            <span className="text-xs font-mono text-slate-400 dark:text-slate-500 bg-slate-100 dark:bg-slate-800 px-2 py-0.5 rounded-md">
              {formatBytes(version.file_size)}
            </span>
            {version.scan_status === "Clean" ? (
              <span className="text-[10px] font-bold text-emerald-600 uppercase tracking-widest">
                Scan Passed
              </span>
            ) : null}
          </div>
        </div>

        <Button
          className="h-8 w-8 rounded-xl"
          onClick={() => { onDownload(version.id); }}
          size="icon"
          title="Download this version"
          variant={isLatest ? "secondary" : "outline"}
        >
          <ArrowDownToLine className="w-4 h-4" />
        </Button>
      </div>
    </div>
  );
}

export function FileVersionHistory({ fileId, workspaceId, className = "" }: FileVersionHistoryProps) {
  const { data: versions, isLoading, error } = useQuery({
    queryKey: ["file-versions", workspaceId, fileId],
    queryFn: async () => {
      const { data } = await fetch<WorkspaceFile[] | undefined>("GET", "/v1/workspaces/{ws_id}/files/{id}/versions", {
        params: { path: { ws_id: workspaceId, id: fileId } },
      });
      return data;
    },
    enabled: Boolean(workspaceId && fileId),
  });

  const handleDownloadVersion = async (versionId: string) => {
    try {
      const { data } = await fetch<{ url?: string } | undefined>("GET", "/v1/workspaces/{ws_id}/files/{id}/url", {
        params: { path: { ws_id: workspaceId, id: versionId } },
      });

      if (data?.url) window.open(data.url, "_blank");
    } catch {
      toast.error("Download failed.");
    }
  };

  if (isLoading) {
    return <LoadingSkeleton className={className} />;
  }

  const hasNoVersions = Boolean(error) || !versions || versions.length === 0;
  if (hasNoVersions) {
    return (
      <div className={`p-6 text-center text-slate-500 text-sm ${className}`}>
        No version history available for this file.
      </div>
    );
  }

  return (
    <div className={`flex flex-col ${className}`}>
      <div className="flex items-center gap-2 mb-6 px-1">
        <History className="w-5 h-5 text-primary-500" />
        <Typography as="h3" className="font-bold text-slate-800 dark:text-slate-200 text-lg mt-0" variant="small">Version History</Typography>
      </div>

      <div className="relative pl-3">
        {/* Vertical Timeline Line */}
        <div className="absolute left-[15px] top-2 bottom-2 w-0.5 bg-slate-200 dark:bg-slate-800 rounded-full" />

        <div className="space-y-6">
          {versions.map((version) => (
            <VersionTimelineItem
              key={version.id}
              onDownload={handleDownloadVersion}
              version={version}
            />
          ))}
        </div>
      </div>
    </div>
  );
}
