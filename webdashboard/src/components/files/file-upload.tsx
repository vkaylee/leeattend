import { UploadCloud, X, AlertCircle } from "lucide-react";
import React, { useCallback, useState } from "react";
import { useTranslation } from "react-i18next";

import type { components } from "@/types/api-generated";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import { useFileUpload } from "@/hooks/use-file-upload";

type FileCategory = components["schemas"]["FileCategory"];
type FileOwnerType = components["schemas"]["FileOwnerType"];

interface FileUploadProps {
  accept?: string;
  category: FileCategory;
  className?: string;
  maxSizeMB?: number;
  onUploadSuccess?: (file: components["schemas"]["WorkspaceFile"]) => void;
  ownerId: string;
  ownerType: FileOwnerType;
}

interface UploadProgressProps {
  progress: number;
}

function UploadProgress({ progress }: UploadProgressProps) {
  const { t } = useTranslation("common");
  return (
    <div className="absolute inset-0 flex flex-col items-center justify-center bg-white/80 dark:bg-slate-900/80 backdrop-blur-sm rounded-2xl z-10 animate-in fade-in">
      <div className="w-64 max-w-[80%]">
        <div className="flex justify-between text-xs font-bold text-slate-700 dark:text-slate-300 mb-2 uppercase tracking-wider">
          <span>{t("common.file.uploading")}</span>
          <span>{Math.round(progress)}%</span>
        </div>
        <div className="w-full bg-slate-200 dark:bg-slate-700 rounded-full h-2 overflow-hidden">
          <div 
            className="bg-primary-500 h-2 rounded-full transition-all duration-300 ease-out" 
            style={{ width: `${progress}%` }}
          ></div>
        </div>
      </div>
    </div>
  );
}

interface ErrorMessageProps {
  message: string;
  onClear: () => void;
}

function ErrorMessage({ message, onClear }: ErrorMessageProps) {
  return (
    <div className="mt-3 flex items-start gap-2 text-rose-600 bg-rose-50 dark:bg-rose-950/30 p-3 rounded-xl text-sm animate-in slide-in-from-top-2">
      <AlertCircle className="w-5 h-5 shrink-0" />
      <span className="font-medium">{message}</span>
      <Button 
        className="ml-auto p-1 h-6 w-6 hover:bg-rose-100 dark:hover:bg-rose-900/50 rounded-lg transition-colors text-rose-600"
        onClick={onClear}
        size="icon"
        variant="ghost"
      >
        <X className="w-4 h-4" />
      </Button>
    </div>
  );
}

function useDragDrop(onDrop: (e: React.DragEvent) => void) {
  const [isDragActive, setIsDragActive] = useState(false);

  const handleDragEnter = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragActive(true);
  }, []);

  const handleDragLeave = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragActive(false);
  }, []);

  const handleDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragActive(false);
    onDrop(e);
  }, [onDrop]);

  return {
    isDragActive,
    handleDragEnter,
    handleDragLeave,
    handleDrop,
  };
}

function getLabelClassName(isDragActive: boolean, isUploading: boolean) {
  return `
    flex flex-col items-center justify-center w-full h-48 
    border-2 border-dashed rounded-2xl cursor-pointer 
    transition-all duration-200 ease-in-out
    ${isDragActive 
      ? "border-primary-500 bg-primary-50 dark:bg-primary-950/20 shadow-inner" 
      : "border-slate-300 dark:border-slate-700 bg-slate-50 dark:bg-slate-900 hover:bg-slate-100 dark:hover:bg-slate-800"
    }
    ${isUploading ? "pointer-events-none opacity-80" : ""}
  `;
}

function getUploadCloudClassName(isDragActive: boolean) {
  return `w-10 h-10 mb-3 transition-colors ${
    isDragActive ? "text-primary-500" : "text-slate-400"
  }`;
}

export function FileUpload({category,
  ownerType,
  ownerId,
  onUploadSuccess,
  maxSizeMB = 100, // Default 100MB
  accept = "*",
  className = "",
}: FileUploadProps) {
  const { t } = useTranslation("common");
  const [errorMsg, setErrorMsg] = useState<string | null>(null);

  const { uploadAsync: uploadFile, isUploading, progress, error: uploadError } = useFileUpload({category,
    ownerType,
    ownerId,
  });

  const processFile = useCallback(
    async (file: File) => {
      setErrorMsg(null);
      if (file.size > maxSizeMB * 1024 * 1024) {
        setErrorMsg(t("common.file.errorSizeExceeded", { size: maxSizeMB }));
        return;
      }
      try {
        const uploadedFile = await uploadFile(file);
        onUploadSuccess?.(uploadedFile);
      } catch (err: unknown) {
        const message = err instanceof Error ? err.message : t("common.file.errorUploadFailed");
        setErrorMsg(message);
      }
    },
    [maxSizeMB, uploadFile, onUploadSuccess, t]
  );

  const handleDropCallback = useCallback(
    (e: React.DragEvent) => {
      if (e.dataTransfer.files.length > 0) {
        void processFile(e.dataTransfer.files[0]);
      }
    },
    [processFile]
  );

  const { isDragActive, handleDragEnter, handleDragLeave, handleDrop } = useDragDrop(handleDropCallback);

  const handleFileChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      if (e.target.files && e.target.files.length > 0) {
        void processFile(e.target.files[0]);
      }
      // Reset input so the same file can be selected again if it failed
      e.target.value = "";
    },
    [processFile]
  );

  const currentError = errorMsg ?? uploadError?.message ?? null;

  return (
    <div className={`relative ${className}`}>
      <label
        className={getLabelClassName(isDragActive, isUploading)}
        onDragEnter={handleDragEnter}
        onDragLeave={handleDragLeave}
        onDragOver={handleDragEnter}
        onDrop={handleDrop}
      >
        <div className="flex flex-col items-center justify-center pt-5 pb-6 text-center px-4">
          <UploadCloud className={getUploadCloudClassName(isDragActive)} />
          <Typography 
            className="mb-2 font-semibold text-slate-700 dark:text-slate-200 mt-0" 
            variant="small"
          >
            <span className="text-primary-600 dark:text-primary-400">{t("common.file.clickToUpload")}</span> {t("common.file.dragAndDrop")}
          </Typography>
          <Typography 
            className="text-xs text-slate-500 dark:text-slate-400 font-medium mt-0" 
            variant="small"
          >
            {t("common.file.maxSize", { size: maxSizeMB })}
          </Typography>
        </div>
        <input 
          accept={accept} 
          className="hidden" 
          disabled={isUploading} 
          onChange={handleFileChange}
          type="file"
        />
      </label>

      {/* Uploading Progress Overlay */}
      {isUploading ? <UploadProgress progress={progress} /> : null}

      {/* Error Message */}
      {!isUploading && currentError ? (
        <ErrorMessage message={currentError} onClear={() => { setErrorMsg(null); }} />
      ) : null}
    </div>
  );
}
