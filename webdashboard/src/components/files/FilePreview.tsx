/* eslint-disable */
import { Download, File as FileIcon, ExternalLink, Loader2 } from "lucide-react";
import React, { useState, useEffect } from "react";

import Button from "@/components/ui/button";
import { client } from "@/lib/api/client";
import { components } from "@/types/api-generated";

type WorkspaceFile = components["schemas"]["WorkspaceFile"];

interface FilePreviewProps {
  className?: string;
  file: WorkspaceFile;
  workspaceId: string;
}

export function FilePreview({ file, workspaceId, className = "" }: FilePreviewProps) {
  const [previewUrl, setPreviewUrl] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let isMounted = true;
    
    const fetchPreviewUrl = async () => {
      // Only fetch URL for previewable types
      const isPreviewable = file.mime_type.startsWith("image/") || file.mime_type === "application/pdf";
      if (!isPreviewable) return;

      setIsLoading(true);
      setError(null);
      try {
        const { data, error } = await client.GET("/v1/workspaces/{ws_id}/files/{id}/url", {
          params: {
            path: {
              ws_id: workspaceId,
              id: file.id,
            },
          },
        });

        if (error) throw error;
        
        // @ts-ignore
        if (isMounted && data?.url) {
          // @ts-ignore
          setPreviewUrl(data.url);
        }
      } catch (err: any) {
        if (isMounted) setError("Failed to load preview");
      } finally {
        if (isMounted) setIsLoading(false);
      }
    };

    fetchPreviewUrl();

    return () => {
      isMounted = false;
    };
  }, [file.id, file.mime_type, workspaceId]);

  const handleDownload = async () => {
    try {
      const { data, error } = await client.GET("/v1/workspaces/{ws_id}/files/{id}/url", {
        params: { path: { ws_id: workspaceId, id: file.id } },
      });
      if (error) throw error;
      // @ts-ignore
      if (data?.url) window.open(data.url, "_blank");
    } catch (err) {
      console.error(err);
    }
  };

  const isImage = file.mime_type.startsWith("image/");
  const isPdf = file.mime_type === "application/pdf";

  return (
    <div className={`flex flex-col rounded-2xl border border-slate-200 dark:border-slate-800 bg-slate-50 dark:bg-slate-900/50 overflow-hidden ${className}`}>
      {/* Header */}
      <div className="flex items-center justify-between p-3 border-b border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900">
        <div className="flex items-center gap-2 truncate pr-4">
          <FileIcon className="w-4 h-4 text-slate-400 shrink-0" />
          <span className="text-sm font-semibold text-slate-700 dark:text-slate-300 truncate" title={file.file_name}>
            {file.file_name}
          </span>
        </div>
        <Button
          className="h-8 w-8 rounded-lg text-slate-500 hover:text-primary-600 dark:text-slate-400 dark:hover:text-primary-400"
          onClick={handleDownload}
          size="icon"
          title="Download File"
          variant="ghost"
        >
          <Download className="w-4 h-4" />
        </Button>
      </div>

      {/* Preview Area */}
      <div className="relative flex-1 min-h-[200px] flex items-center justify-center bg-slate-100/50 dark:bg-slate-950/30 p-4">
        {!!isLoading && (
          <div className="flex flex-col items-center justify-center text-slate-400 gap-2">
            <Loader2 className="w-6 h-6 animate-spin" />
            <span className="text-xs font-medium tracking-wide uppercase">Loading preview...</span>
          </div>
        )}

        {!!error && (
          <div className="text-center">
            <p className="text-sm text-rose-500 font-medium mb-2">{error}</p>
            <Button className="text-xs font-bold text-slate-600 underline hover:bg-transparent hover:text-slate-800 h-auto p-0" onClick={handleDownload} variant="ghost">
              Download instead
            </Button>
          </div>
        )}

        {!isLoading && !error && !!previewUrl && !!isImage && (
          <img 
            alt={file.file_name} 
            className="max-w-full max-h-[500px] object-contain rounded-lg shadow-sm border border-slate-200/50 dark:border-slate-800/50"
            loading="lazy"
            src={previewUrl}
          />
        )}

        {!isLoading && !error && !!previewUrl && !!isPdf && (
          <iframe 
            className="w-full h-[500px] rounded-lg border border-slate-200/50 dark:border-slate-800/50" 
            src={`${previewUrl}#toolbar=0`}
            title={file.file_name}
          />
        )}

        {!isLoading && !error && !isImage && !isPdf && (
          <div className="flex flex-col items-center justify-center text-center p-6">
            <FileIcon className="w-16 h-16 text-slate-300 dark:text-slate-700 mb-4" />
            <h3 className="text-sm font-bold text-slate-700 dark:text-slate-300 mb-1">No Preview Available</h3>
            <p className="text-xs text-slate-500 dark:text-slate-500 mb-4 max-w-[200px]">
              This file type ({file.mime_type}) cannot be previewed in the browser.
            </p>
            <Button
              className="flex items-center gap-2 px-4 py-2 rounded-xl text-sm font-bold text-primary-600 dark:text-primary-400 hover:bg-primary-50 dark:hover:bg-primary-950/30 transition-colors shadow-sm"
              onClick={handleDownload}
              variant="outline"
            >
              <ExternalLink className="w-4 h-4" /> Download File
            </Button>
          </div>
        )}
      </div>
    </div>
  );
}
