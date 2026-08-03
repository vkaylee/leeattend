import { FileIcon, FileText, Image as ImageIcon, FileArchive, FileCode, FileSpreadsheet, Download, Trash2 } from "lucide-react";
import React from "react";

import type { components } from "@/types/api-generated";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import { useDateTimeFormatter } from "@/hooks/use-datetime";

type WorkspaceFile = components["schemas"]["WorkspaceFile"];

export interface FileGridProps {
  files: WorkspaceFile[];
  onDeleteFile: (file: WorkspaceFile) => void;
  onDownloadFile: (file: WorkspaceFile) => void;
  onViewFile: (file: WorkspaceFile) => void;
}

/**
 * Map MIME type to icon component. The if/else chain is inherently flat —
 * each branch is a single return. Marking complexity as acceptable.
 */
// eslint-disable-next-line complexity
function getFileIcon(mimeType: string) {
  if (mimeType.includes("pdf")) return <FileText className="w-8 h-8 text-red-500" />;
  if (mimeType.includes("image")) return <ImageIcon className="w-8 h-8 text-blue-500" />;
  if (mimeType.includes("zip") || mimeType.includes("tar")) return <FileArchive className="w-8 h-8 text-amber-500" />;
  if (mimeType.includes("json") || mimeType.includes("javascript")) return <FileCode className="w-8 h-8 text-slate-500" />;
  if (mimeType.includes("spreadsheet") || mimeType.includes("excel") || mimeType.includes("csv")) return <FileSpreadsheet className="w-8 h-8 text-green-500" />;
  if (mimeType.includes("word")) return <FileText className="w-8 h-8 text-blue-600" />;
  return <FileIcon className="w-8 h-8 text-slate-400" />;
}

function formatBytes(bytes: number, decimals = 2) {
  // eslint-disable-next-line @typescript-eslint/no-unnecessary-type-conversion
  if (!+bytes) return '0 Bytes';
  const k = 1024;
  const dm = decimals < 0 ? 0 : decimals;
  const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB', 'PB', 'EB', 'ZB', 'YB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(dm))} ${sizes[i]}`;
}

export function FileGrid({ files, onViewFile, onDownloadFile, onDeleteFile }: FileGridProps) {
  const { formatDate } = useDateTimeFormatter();
  if (files.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center py-20 text-slate-400 border-2 border-dashed border-slate-200 dark:border-slate-800 rounded-2xl">
        <FileIcon className="w-12 h-12 mb-4 text-slate-300 dark:text-slate-600" />
        <Typography variant="p">No files in this folder.</Typography>
      </div>
    );
  }

  return (
    <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-4">
      {files.map((file) => (
        <div
          className="group relative flex flex-col items-center p-4 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-xl hover:shadow-lg transition-all hover:border-primary-200 dark:hover:border-primary-800 cursor-pointer focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary-500"
          key={file.id}
          onClick={() => { onViewFile(file); }}
          onKeyDown={(e) => {
            if (e.key === "Enter" || e.key === " ") {
              e.preventDefault();
              onViewFile(file);
            }
          }}
          role="button"
          tabIndex={0}
        >
          <div className="absolute top-2 right-2 flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
            <Button
              className="w-8 h-8 bg-white/80 dark:bg-slate-900/80 backdrop-blur-sm"
              onClick={(e: React.MouseEvent) => {
                e.stopPropagation();
                onDownloadFile(file);
              }}
              size="icon"
              variant="ghost"
            >
              <Download className="w-4 h-4 text-slate-600 dark:text-slate-300" />
            </Button>
            <Button
              className="w-8 h-8 bg-white/80 dark:bg-slate-900/80 backdrop-blur-sm hover:text-red-600"
              onClick={(e: React.MouseEvent) => {
                e.stopPropagation();
                onDeleteFile(file);
              }}
              size="icon"
              variant="ghost"
            >
              <Trash2 className="w-4 h-4 text-red-500" />
            </Button>
          </div>
          
          <div className="w-16 h-16 mb-4 rounded-xl bg-slate-50 dark:bg-slate-800 flex items-center justify-center">
            {getFileIcon(file.mime_type)}
          </div>
          
          <div className="w-full text-center">
            <Typography className="font-medium text-sm text-slate-700 dark:text-slate-300 truncate" title={file.file_name} variant="p">
              {file.file_name}
            </Typography>
            <Typography className="text-xs text-slate-400 mt-1" variant="p">
              {formatBytes(file.file_size)} • {formatDate(file.created_at)}
            </Typography>
          </div>
        </div>
      ))}
    </div>
  );
}
