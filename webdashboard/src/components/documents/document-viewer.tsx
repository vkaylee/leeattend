import React from "react";
import Modal from "@/components/ui/modal";
import { FileIcon, Download, AlertCircle } from "lucide-react";
import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";

export interface DocumentViewerModalProps {
  isOpen: boolean;
  onClose: () => void;
  fileName: string;
  mimeType: string;
  fileUrl?: string | null;
  onDownload: () => void;
}

export function DocumentViewerModal({
  isOpen,
  onClose,
  fileName,
  mimeType,
  fileUrl,
  onDownload,
}: DocumentViewerModalProps) {
  // Determine how to render based on mime type
  const isImage = mimeType.startsWith("image/");
  const isPdf = mimeType === "application/pdf";
  const isSupported = isImage || isPdf;

  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      title={fileName}
      subtitle={mimeType}
      maxWidth="4xl"
      icon={<FileIcon className="w-5 h-5" />}
    >
      <div className="flex flex-col gap-4 w-full h-[60vh]">
        {!fileUrl ? (
          <div className="flex-1 flex flex-col items-center justify-center bg-slate-50 dark:bg-slate-900 rounded-2xl border border-slate-100 dark:border-slate-800">
            <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary-600 mb-4"></div>
            <Typography className="text-slate-500" variant="p">Loading document securely...</Typography>
          </div>
        ) : isSupported ? (
          <div className="flex-1 w-full bg-slate-100 dark:bg-slate-950 rounded-2xl overflow-hidden border border-slate-200 dark:border-slate-800 relative">
            {isImage ? (
              <img
                src={fileUrl}
                alt={fileName}
                className="w-full h-full object-contain"
              />
            ) : null}
            {isPdf ? (
              <iframe
                src={fileUrl}
                title={fileName}
                className="w-full h-full border-none"
              />
            ) : null}
          </div>
        ) : (
          <div className="flex-1 flex flex-col items-center justify-center bg-slate-50 dark:bg-slate-900 rounded-2xl border border-slate-100 dark:border-slate-800">
            <AlertCircle className="w-16 h-16 text-amber-500 mb-4" />
            <Typography className="text-lg font-bold text-slate-800 dark:text-slate-200 mb-2" variant="h3">
              Preview Not Available
            </Typography>
            <Typography className="text-slate-500 text-center max-w-sm mb-6" variant="p">
              This file type ({mimeType}) cannot be previewed directly in the browser. Please download it to view its contents.
            </Typography>
            <Button onClick={onDownload} size="lg">
              <Download className="w-4 h-4 mr-2" />
              Download File
            </Button>
          </div>
        )}

        {isSupported ? (
          <div className="flex justify-end mt-2">
            <Button onClick={onDownload} variant="outline">
              <Download className="w-4 h-4 mr-2" />
              Download Original
            </Button>
          </div>
        ) : null}
      </div>
    </Modal>
  );
}
