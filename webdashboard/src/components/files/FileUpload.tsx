/* eslint-disable */
import { UploadCloud, X, AlertCircle } from "lucide-react";
import React, { useCallback, useState } from "react";

import Button from "@/components/ui/button";
import { useFileUpload } from "@/hooks/useFileUpload";
import { components } from "@/types/api-generated";

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
  workspaceId: string;
}

export function FileUpload({
  workspaceId,
  category,
  ownerType,
  ownerId,
  onUploadSuccess,
  maxSizeMB = 100, // Default 100MB
  accept = "*",
  className = "",
}: FileUploadProps) {
  const [isDragActive, setIsDragActive] = useState(false);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);

  const { uploadAsync: uploadFile, isUploading, progress, error: uploadError } = useFileUpload({
    workspaceId,
    category,
    ownerType,
    ownerId,
  });

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

  const processFile = async (file: File) => {
    setErrorMsg(null);
    if (file.size > maxSizeMB * 1024 * 1024) {
      setErrorMsg(`File size exceeds the limit of ${maxSizeMB}MB.`);
      return;
    }
    try {
      const uploadedFile = await uploadFile(file);
      if (uploadedFile && onUploadSuccess) {
        onUploadSuccess(uploadedFile);
      }
    } catch (err: any) {
      setErrorMsg(err.message || "Failed to upload file.");
    }
  };

  const handleDrop = useCallback(
    (e: React.DragEvent) => {
      e.preventDefault();
      e.stopPropagation();
      setIsDragActive(false);
      
      if (e.dataTransfer.files && e.dataTransfer.files.length > 0) {
        processFile(e.dataTransfer.files[0]);
      }
    },
    [maxSizeMB, uploadFile, onUploadSuccess]
  );

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (e.target.files && e.target.files.length > 0) {
      processFile(e.target.files[0]);
    }
    // Reset input so the same file can be selected again if it failed
    e.target.value = "";
  };

  const currentError = errorMsg || uploadError?.message;

  return (
    <div className={`relative ${className}`}>
      <label
        className={`
          flex flex-col items-center justify-center w-full h-48 
          border-2 border-dashed rounded-2xl cursor-pointer 
          transition-all duration-200 ease-in-out
          ${isDragActive 
            ? "border-primary-500 bg-primary-50 dark:bg-primary-950/20 shadow-inner" 
            : "border-slate-300 dark:border-slate-700 bg-slate-50 dark:bg-slate-900 hover:bg-slate-100 dark:hover:bg-slate-800"
          }
          ${isUploading ? "pointer-events-none opacity-80" : ""}
        `}
        onDragEnter={handleDragEnter}
        onDragLeave={handleDragLeave}
        onDragOver={handleDragEnter}
        onDrop={handleDrop}
      >
        <div className="flex flex-col items-center justify-center pt-5 pb-6 text-center px-4">
          <UploadCloud 
            className={`w-10 h-10 mb-3 transition-colors ${
              isDragActive ? "text-primary-500" : "text-slate-400"
            }`} 
          />
          <p className="mb-2 text-sm font-semibold text-slate-700 dark:text-slate-200">
            <span className="text-primary-600 dark:text-primary-400">Click to upload</span> or drag and drop
          </p>
          <p className="text-xs text-slate-500 dark:text-slate-400 font-medium">
            Maximum file size: {maxSizeMB}MB
          </p>
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
      {!!isUploading && (
        <div className="absolute inset-0 flex flex-col items-center justify-center bg-white/80 dark:bg-slate-900/80 backdrop-blur-sm rounded-2xl z-10 animate-in fade-in">
          <div className="w-64 max-w-[80%]">
            <div className="flex justify-between text-xs font-bold text-slate-700 dark:text-slate-300 mb-2 uppercase tracking-wider">
              <span>Uploading...</span>
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
      )}

      {/* Error Message */}
      {!!currentError && !isUploading && (
        <div className="mt-3 flex items-start gap-2 text-rose-600 bg-rose-50 dark:bg-rose-950/30 p-3 rounded-xl text-sm animate-in slide-in-from-top-2">
          <AlertCircle className="w-5 h-5 shrink-0" />
          <span className="font-medium">{currentError}</span>
          <Button 
            className="ml-auto p-1 h-6 w-6 hover:bg-rose-100 dark:hover:bg-rose-900/50 rounded-lg transition-colors text-rose-600"
            onClick={() => { setErrorMsg(null); }}
            size="icon"
            variant="ghost"
          >
            <X className="w-4 h-4" />
          </Button>
        </div>
      )}
    </div>
  );
}
