import React, { useState } from "react";
import Modal from "@/components/ui/modal";
import { FileText, Loader2 } from "lucide-react";

interface PDFPreviewModalProps {
  isOpen: boolean;
  onClose: () => void;
  fileUrl: string | null;
  title?: string;
}

export function PDFPreviewModal({
  isOpen,
  onClose,
  fileUrl,
  title = "Document Preview",
}: PDFPreviewModalProps) {
  const [isLoading, setIsLoading] = useState(true);

  React.useEffect(() => {
    if (isOpen) {
      setIsLoading(true);
    }
  }, [isOpen, fileUrl]);

  if (!isOpen || !fileUrl) return null;

  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      title={title}
      maxWidth="4xl"
      icon={<FileText size={20} />}
      className="h-[85vh] flex flex-col"
    >
      <div className="relative w-full h-full min-h-[500px] flex-1 rounded-2xl overflow-hidden border border-slate-200 dark:border-slate-800 bg-slate-100 dark:bg-slate-950">
        {isLoading && (
          <div className="absolute inset-0 flex flex-col items-center justify-center bg-slate-50 dark:bg-slate-900/50 backdrop-blur-sm z-10">
            <Loader2 className="w-10 h-10 animate-spin text-primary-500 mb-2" />
            <span className="text-xs text-slate-500 dark:text-slate-400 font-semibold uppercase tracking-wider">
              Loading PDF Document...
            </span>
          </div>
        )}
        <iframe
          src={`${fileUrl}#toolbar=0&navpanes=0`}
          className="w-full h-full border-none rounded-2xl"
          onLoad={() => setIsLoading(false)}
          title="PDF Document Preview"
        />
      </div>
    </Modal>
  );
}
