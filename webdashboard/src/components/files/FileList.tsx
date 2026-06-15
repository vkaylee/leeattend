/* eslint-disable */
import { type ColumnDef } from "@tanstack/react-table";
import {
  FileText,
  FileImage,
  FileArchive,
  FileCode,
  FileSpreadsheet,
  FileQuestion,
  Download,
  Trash2,
  Share2,
  PenSquare
} from "lucide-react";
import React, { useMemo, useState } from "react";

import { format } from "date-fns";

import Button from "@/components/ui/button";
import SmartTable from "@/components/ui/smart-table";
import { formatBytes } from "@/lib/utils";
import { components } from "@/types/api-generated";

type WorkspaceFile = components["schemas"]["WorkspaceFile"];

// Utility to get nice icons for different mime types
const getFileIcon = (mimeType: string) => {
  if (mimeType.startsWith("image/")) return <FileImage className="h-4 w-4 text-blue-500" />;
  if (mimeType.includes("pdf")) return <FileText className="h-4 w-4 text-red-500" />;
  if (mimeType.includes("zip") || mimeType.includes("tar")) return <FileArchive className="h-4 w-4 text-yellow-500" />;
  if (mimeType.includes("spreadsheet") || mimeType.includes("excel") || mimeType.includes("csv")) return <FileSpreadsheet className="h-4 w-4 text-emerald-500" />;
  if (mimeType.includes("json") || mimeType.includes("xml")) return <FileCode className="h-4 w-4 text-indigo-500" />;
  return <FileQuestion className="h-4 w-4 text-slate-400" />;
};

interface FileListProps {
  files: WorkspaceFile[];
  isLoading?: boolean;
  onDelete: (fileIds: string[]) => void;
  onDownload: (file: WorkspaceFile) => void;
  onRename: (fileId: string, newName: string) => Promise<void>;
  onShare: (file: WorkspaceFile) => void;
}

export function FileList({
  files,
  isLoading,
  onDownload,
  onRename,
  onDelete,
  onShare,
}: FileListProps) {
  const [selectedRowIds, setSelectedRowIds] = useState<Set<string>>(new Set());

  const columns = useMemo<ColumnDef<WorkspaceFile>[]>(
    () => [
      {
        accessorKey: "file_name",
        header: "Name",
        size: 350,
        meta: {
          editable: true,
          editType: "text",
          sticky: "left",
        },
        cell: ({ row, getValue }) => {
          const mimeType = row.original.mime_type;
          return (
            <div className="flex items-center gap-3">
              {getFileIcon(mimeType)}
              <span className="font-semibold truncate max-w-[300px]" title={getValue() as string}>
                {getValue() as string}
              </span>
            </div>
          );
        },
      },
      {
        accessorKey: "file_size",
        header: "Size",
        size: 100,
        cell: ({ getValue }) => (
          <span className="text-slate-500 font-mono text-xs">
            {formatBytes(getValue() as number)}
          </span>
        ),
      },
      {
        accessorKey: "category",
        header: "Category",
        size: 150,
        cell: ({ getValue }) => {
          const category = getValue() as string;
          return (
            <div className="inline-flex items-center px-2 py-0.5 rounded-full bg-slate-100 dark:bg-slate-800 text-slate-600 dark:text-slate-300 text-[10px] uppercase font-bold tracking-wider">
              {category.replace("_", " ")}
            </div>
          );
        },
      },
      {
        accessorKey: "scan_status",
        header: "Security",
        size: 120,
        cell: ({ getValue }) => {
          const status = getValue() as string;
          const statusStyles: Record<string, string> = {
            "clean": "text-emerald-600 bg-emerald-50 dark:bg-emerald-950/30",
            "pending": "text-amber-600 bg-amber-50 dark:bg-amber-950/30",
            "infected": "text-rose-600 bg-rose-50 dark:bg-rose-950/30",
            "skipped": "text-slate-500 bg-slate-50 dark:bg-slate-800",
          };
          
          return (
            <div className={`inline-flex items-center px-2.5 py-1 rounded-md text-[11px] font-bold ${statusStyles[status] || statusStyles.skipped}`}>
              {status}
            </div>
          );
        },
      },
      {
        accessorKey: "created_at",
        header: "Uploaded At",
        size: 150,
        cell: ({ getValue }) => (
          <span className="text-slate-500 text-xs">
            {format(new Date(getValue() as string), "MMM dd, yyyy HH:mm")}
          </span>
        ),
      },
    ],
    []
  );

  const handleCellSave = async (rowId: string, columnId: string, value: any) => {
    if (columnId === "file_name" && value.trim()) {
      await onRename(rowId, value.trim());
    }
  };

  const getContextMenuItems = (row: WorkspaceFile) => [
    {
      label: "Download",
      icon: <Download className="h-4 w-4" />,
      onClick: () => { onDownload(row); },
    },
    {
      label: "Share & Request Signature",
      icon: <Share2 className="h-4 w-4" />,
      onClick: () => { onShare(row); },
    },
    {
      label: "Rename",
      icon: <PenSquare className="h-4 w-4" />,
      onClick: () => {
        // We rely on F2/Enter for direct editing, but providing a menu is good UX
        // You would typically trigger the startCellEdit logic here via a ref to SmartTable,
        // but for now, we leave it as an instructional item.
      },
    },
    {
      label: "Delete",
      icon: <Trash2 className="h-4 w-4 text-rose-500" />,
      onClick: () => { onDelete([row.id]); },
    },
  ];

  return (
    <div className="h-full w-full flex flex-col gap-4">
      {/* Table Area */}
      <div className="flex-1 min-h-[400px]">
        <SmartTable<WorkspaceFile>
          columns={columns}
          contextMenuItems={getContextMenuItems}
          data={files}
          emptyMessage="No files found. Drag and drop to upload."
          enableDragSelect={true}
          getRowId={(row) => row.id}
          isLoading={isLoading}
          onCellSave={handleCellSave}
          onSelectionChange={setSelectedRowIds}
          selectedRowIds={selectedRowIds}
        />
      </div>

      {/* Bulk Actions Panel (Only visible when items are selected) */}
      {selectedRowIds.size > 0 && (
        <div className="flex items-center justify-between p-4 bg-primary-50 dark:bg-primary-950/30 border border-primary-200 dark:border-primary-800 rounded-2xl animate-in slide-in-from-bottom-2 fade-in">
          <span className="text-sm font-semibold text-primary-700 dark:text-primary-300">
            {selectedRowIds.size} file(s) selected
          </span>
          <div className="flex items-center gap-3">
            <Button
              className="px-4 py-2 text-sm font-bold rounded-xl transition-colors h-9"
              onClick={() => { onDelete(Array.from(selectedRowIds)); }}
              variant="destructive"
            >
              Delete Selected
            </Button>
            <Button
              className="px-4 py-2 text-sm font-bold rounded-xl transition-colors h-9"
              onClick={() => { setSelectedRowIds(new Set()); }}
              variant="outline"
            >
              Cancel
            </Button>
          </div>
        </div>
      )}
    </div>
  );
}
