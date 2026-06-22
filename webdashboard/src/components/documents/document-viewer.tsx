import { FileIcon, Download, AlertCircle } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import Modal from "@/components/ui/modal";
import { Typography } from "@/components/ui/typography";

export interface DocumentViewerModalProps {
  fileName: string;
  fileUrl?: string | null;
  isOpen: boolean;
  mimeType: string;
  onClose: () => void;
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
  const { t } = useTranslation();
  // Determine how to render based on mime type
  const isImage = mimeType.startsWith("image/");
  const isPdf = mimeType === "application/pdf";
  const isSupported = isImage || isPdf;

  return (
    <Modal
      icon={<FileIcon className="w-5 h-5" />}
      isOpen={isOpen}
      maxWidth="4xl"
      onClose={onClose}
      subtitle={mimeType}
      title={fileName}
    >
      <div className="flex flex-col gap-4 w-full h-[60vh]">
        {!fileUrl ? (
          <div className="flex-1 flex flex-col items-center justify-center bg-slate-50 dark:bg-slate-900 rounded-2xl border border-slate-100 dark:border-slate-800">
            <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary-600 mb-4"></div>
            <Typography className="text-slate-500" variant="p">{t("documents.loading_securely")}</Typography>
          </div>
        ) : isSupported ? (
          <div className="flex-1 w-full bg-slate-100 dark:bg-slate-950 rounded-2xl overflow-hidden border border-slate-200 dark:border-slate-800 relative">
            {isImage ? (
              <img
                alt={fileName}
                className="w-full h-full object-contain"
                src={fileUrl}
              />
            ) : null}
            {isPdf ? (
              <iframe
                className="w-full h-full border-none"
                src={fileUrl}
                title={fileName}
              />
            ) : null}
          </div>
        ) : (
          <div className="flex-1 flex flex-col items-center justify-center bg-slate-50 dark:bg-slate-900 rounded-2xl border border-slate-100 dark:border-slate-800">
            <AlertCircle className="w-16 h-16 text-amber-500 mb-4" />
            <Typography className="text-lg font-bold text-slate-800 dark:text-slate-200 mb-2" variant="h3">
              {t("documents.preview_not_available")}
            </Typography>
            <Typography className="text-slate-500 text-center max-w-sm mb-6" variant="p">
              {t("documents.preview_not_supported_desc", { mimeType })}
            </Typography>
            <Button onClick={onDownload} size="lg">
              <Download className="w-4 h-4 mr-2" />
              {t("documents.download_file")}
            </Button>
          </div>
        )}

        {isSupported ? (
          <div className="flex justify-end mt-2">
            <Button onClick={onDownload} variant="outline">
              <Download className="w-4 h-4 mr-2" />
              {t("documents.download_original")}
            </Button>
          </div>
        ) : null}
      </div>
    </Modal>
  );
}
