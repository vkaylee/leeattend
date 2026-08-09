/* eslint-disable @typescript-eslint/no-unnecessary-type-conversion */
import { FileText, Loader2 } from "lucide-react";
import React, { useState } from "react";

import Modal from "@/components/ui/modal";

interface PDFPreviewModalProps {
  fileUrl: string | null;
  isOpen: boolean;
  onClose: () => void;
  title?: string;
}

export function PDFPreviewModal({
  isOpen,
  onClose,
  fileUrl,
  title = "Document Preview",
}: PDFPreviewModalProps) {
  const [isLoading, setIsLoading] = useState(true);
  const [prevFileUrl, setPrevFileUrl] = useState<string | null>(null);
  const [prevIsOpen, setPrevIsOpen] = useState<boolean>(false);

  if (fileUrl !== prevFileUrl || isOpen !== prevIsOpen) {
    setPrevFileUrl(fileUrl);
    setPrevIsOpen(isOpen);
    if (isOpen) {
      setIsLoading(true);
    }
  }

  if (!isOpen || !fileUrl) return null;

  return (
    <Modal
      className="h-[85dvh] flex flex-col"
      icon={<FileText size={20} />}
      isOpen={isOpen}
      maxWidth="4xl"
      onClose={onClose}
      title={title}
    >
      <div className="relative w-full h-full min-h-[500px] flex-1 rounded-2xl overflow-hidden border border-slate-200 dark:border-slate-800 bg-slate-100 dark:bg-slate-950">
        {!!isLoading && (
          <div className="absolute inset-0 flex flex-col items-center justify-center bg-slate-50 dark:bg-slate-900/50 backdrop-blur-sm z-10">
            <Loader2 className="w-10 h-10 animate-spin text-primary-500 mb-2" />
            <span className="text-xs text-slate-500 dark:text-slate-400 font-semibold uppercase tracking-wider">
              Loading PDF Document...
            </span>
          </div>
        )}
        <iframe
          className="w-full h-full border-none rounded-2xl"
          onLoad={() => { setIsLoading(false); }}
          src={`${fileUrl}#toolbar=0&navpanes=0`}
          title="PDF Document Preview"
        />
      </div>
    </Modal>
  );
}
