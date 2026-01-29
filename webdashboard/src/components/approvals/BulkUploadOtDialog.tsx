import { useState } from "react";
import * as Dialog from "@radix-ui/react-dialog";
import { X, Upload, Download, FileText, AlertCircle, CheckCircle2, XCircle } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useBulkUploadPaperOt } from "@/hooks/use-attendance";

interface Props {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSuccess?: () => void;
}

interface UploadResult {
  total_rows: number;
  success_count: number;
  failed_count: number;
  errors: Array<{
    line_number: number;
    employee_id: string;
    error_message: string;
  }>;
}

export function BulkUploadOtDialog({ open, onOpenChange, onSuccess }: Props) {
  const { t } = useTranslation();
  const { bulkUploadPaperOt, isUploading } = useBulkUploadPaperOt();

  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  const [uploadResult, setUploadResult] = useState<UploadResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isDragOver, setIsDragOver] = useState(false);

  const resetForm = () => {
    setSelectedFile(null);
    setUploadResult(null);
    setError(null);
    setIsDragOver(false);
  };

  const handleFileChange = (file: File | null) => {
    if (!file) return;

    if (!file.name.endsWith(".csv")) {
      setError(t("approvals.bulk_upload.error_csv_only") || "Only CSV files are allowed");
      return;
    }

    setSelectedFile(file);
    setError(null);
    setUploadResult(null);
  };

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragOver(false);
    const file = e.dataTransfer.files[0];
    handleFileChange(file);
  };

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragOver(true);
  };

  const handleDragLeave = () => {
    setIsDragOver(false);
  };

  const downloadTemplate = () => {
    const csvContent = `employee_id,date,minutes,reason
E001,2026-01-28,120,Production line overtime - paper form #123
E002,2026-01-28,90,Urgent order completion`;

    const blob = new Blob([csvContent], { type: "text/csv" });
    const url = window.URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = "ot_upload_template.csv";
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    window.URL.revokeObjectURL(url);
  };

  const downloadErrorReport = () => {
    if (!uploadResult || uploadResult.errors.length === 0) return;

    const headers = "line_number,employee_id,error_message";
    const rows = uploadResult.errors
      .map(e => `${e.line_number},${e.employee_id},"${e.error_message}"`)
      .join("\n");

    const csvContent = `${headers}\n${rows}`;
    const blob = new Blob([csvContent], { type: "text/csv" });
    const url = window.URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `ot_upload_errors_${new Date().getTime()}.csv`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    window.URL.revokeObjectURL(url);
  };

  const handleUpload = async () => {
    if (!selectedFile) return;

    setError(null);
    try {
      const result = await bulkUploadPaperOt(selectedFile);
      setUploadResult(result);

      if (result.success_count > 0) {
        onSuccess?.();
      }
    } catch (e: any) {
      setError(e.message || "Failed to upload file");
    }
  };

  const handleClose = () => {
    if (uploadResult && uploadResult.failed_count > 0) {
      const confirmed = confirm(
        t("approvals.bulk_upload.confirm_close_with_errors") ||
        "Some rows failed. Are you sure you want to close?"
      );
      if (!confirmed) return;
    }
    resetForm();
    onOpenChange(false);
  };

  return (
    <Dialog.Root open={open} onOpenChange={(o) => { if (!o) handleClose(); }}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 bg-black/40 z-50 backdrop-blur-sm" />
        <Dialog.Content className="fixed left-1/2 top-1/2 z-50 w-full max-w-2xl -translate-x-1/2 -translate-y-1/2 border border-slate-100 bg-white p-6 shadow-xl rounded-[2rem] max-h-[90vh] overflow-y-auto">
          <Dialog.Title className="text-xl font-bold text-slate-800 uppercase tracking-tight flex items-center gap-2">
            <Upload className="w-5 h-5" />
            {t("approvals.bulk_upload.title") || "Bulk Upload OT"}
          </Dialog.Title>
          <Dialog.Description className="text-sm text-slate-500 mt-1">
            {t("approvals.bulk_upload.description") || "Upload CSV file to enter multiple OT claims at once"}
          </Dialog.Description>

          <button
            onClick={handleClose}
            className="absolute right-4 top-4 p-2 rounded-full hover:bg-slate-100"
          >
            <X className="h-4 w-4" />
          </button>

          <div className="mt-6 space-y-5">
            {/* Download Template Button */}
            <div className="flex justify-center">
              <button
                onClick={downloadTemplate}
                className="px-6 py-3 rounded-xl bg-slate-100 hover:bg-slate-200 text-slate-700 text-xs font-bold uppercase tracking-wider transition-all flex items-center gap-2"
              >
                <Download className="w-4 h-4" />
                {t("approvals.bulk_upload.download_template") || "Download CSV Template"}
              </button>
            </div>

            {/* File Upload Zone */}
            {!uploadResult && (
              <div
                onDrop={handleDrop}
                onDragOver={handleDragOver}
                onDragLeave={handleDragLeave}
                className={`border-2 border-dashed rounded-xl p-8 text-center transition-all cursor-pointer ${
                  isDragOver
                    ? "border-primary-500 bg-primary-50"
                    : "border-slate-200 hover:border-primary-400"
                }`}
              >
                {selectedFile ? (
                  <div className="space-y-3">
                    <FileText className="w-12 h-12 text-primary-600 mx-auto" />
                    <p className="text-sm font-bold text-slate-700">{selectedFile.name}</p>
                    <p className="text-xs text-slate-500">
                      {(selectedFile.size / 1024).toFixed(2)} KB
                    </p>
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        setSelectedFile(null);
                      }}
                      className="text-xs text-rose-600 hover:underline"
                    >
                      {t("common.remove") || "Remove"}
                    </button>
                  </div>
                ) : (
                  <>
                    <Upload className="w-12 h-12 text-slate-400 mx-auto mb-3" />
                    <p className="text-sm font-bold text-slate-700 mb-1">
                      {t("approvals.bulk_upload.drag_drop") || "Drag and drop CSV file here"}
                    </p>
                    <p className="text-xs text-slate-500 mb-4">
                      {t("approvals.bulk_upload.or_click") || "or click to browse"}
                    </p>
                    <input
                      type="file"
                      accept=".csv"
                      onChange={(e) => handleFileChange(e.target.files?.[0] || null)}
                      className="hidden"
                      id="csv-upload"
                    />
                    <label
                      htmlFor="csv-upload"
                      className="inline-block px-6 py-2 rounded-xl bg-primary-600 text-white text-xs font-bold uppercase tracking-wider cursor-pointer hover:bg-primary-700"
                    >
                      {t("approvals.bulk_upload.select_file") || "Select File"}
                    </label>
                  </>
                )}
              </div>
            )}

            {/* Upload Button */}
            {selectedFile && !uploadResult && (
              <div className="flex justify-center">
                <button
                  onClick={handleUpload}
                  disabled={isUploading}
                  className="px-8 py-3 rounded-xl bg-primary-600 text-white text-xs font-bold uppercase tracking-wider shadow-lg hover:bg-primary-700 disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
                >
                  {isUploading ? (
                    <>
                      <div className="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin" />
                      {t("common.uploading") || "Uploading..."}
                    </>
                  ) : (
                    <>
                      <Upload className="w-4 h-4" />
                      {t("approvals.bulk_upload.upload") || "Upload & Process"}
                    </>
                  )}
                </button>
              </div>
            )}

            {/* Error Message */}
            {error && (
              <div className="flex items-center gap-3 bg-rose-50 border border-rose-200 rounded-xl p-4">
                <AlertCircle className="w-5 h-5 text-rose-600 flex-shrink-0" />
                <p className="text-xs font-bold text-rose-600 uppercase tracking-wide">{error}</p>
              </div>
            )}

            {/* Upload Result */}
            {uploadResult && (
              <div className="space-y-4">
                {/* Summary */}
                <div className="bg-slate-50 rounded-xl p-4 space-y-2">
                  <h3 className="text-sm font-bold text-slate-700 uppercase tracking-wide">
                    {t("approvals.bulk_upload.result_summary") || "Upload Summary"}
                  </h3>
                  <div className="grid grid-cols-3 gap-3 text-center">
                    <div className="bg-white rounded-lg p-3 border border-slate-100">
                      <p className="text-xs text-slate-500 uppercase tracking-wider mb-1">
                        {t("approvals.bulk_upload.total") || "Total"}
                      </p>
                      <p className="text-2xl font-bold text-slate-700">{uploadResult.total_rows}</p>
                    </div>
                    <div className="bg-emerald-50 rounded-lg p-3 border border-emerald-200">
                      <p className="text-xs text-emerald-600 uppercase tracking-wider mb-1 flex items-center justify-center gap-1">
                        <CheckCircle2 className="w-3 h-3" />
                        {t("approvals.bulk_upload.success") || "Success"}
                      </p>
                      <p className="text-2xl font-bold text-emerald-700">{uploadResult.success_count}</p>
                    </div>
                    <div className="bg-rose-50 rounded-lg p-3 border border-rose-200">
                      <p className="text-xs text-rose-600 uppercase tracking-wider mb-1 flex items-center justify-center gap-1">
                        <XCircle className="w-3 h-3" />
                        {t("approvals.bulk_upload.failed") || "Failed"}
                      </p>
                      <p className="text-2xl font-bold text-rose-700">{uploadResult.failed_count}</p>
                    </div>
                  </div>
                </div>

                {/* Error Details */}
                {uploadResult.errors.length > 0 && (
                  <div className="space-y-3">
                    <div className="flex items-center justify-between">
                      <h3 className="text-sm font-bold text-slate-700 uppercase tracking-wide">
                        {t("approvals.bulk_upload.error_details") || "Error Details"}
                      </h3>
                      <button
                        onClick={downloadErrorReport}
                        className="px-4 py-2 rounded-xl bg-rose-100 hover:bg-rose-200 text-rose-700 text-xs font-bold uppercase tracking-wider transition-all flex items-center gap-2"
                      >
                        <Download className="w-3 h-3" />
                        {t("approvals.bulk_upload.download_errors") || "Download Errors"}
                      </button>
                    </div>

                    <div className="bg-white border border-slate-200 rounded-xl overflow-hidden max-h-64 overflow-y-auto">
                      <table className="w-full text-left">
                        <thead className="bg-slate-50 sticky top-0">
                          <tr>
                            <th className="px-4 py-2 text-[10px] font-bold text-slate-500 uppercase tracking-wider">
                              {t("approvals.bulk_upload.line") || "Line"}
                            </th>
                            <th className="px-4 py-2 text-[10px] font-bold text-slate-500 uppercase tracking-wider">
                              {t("common.employee") || "Employee ID"}
                            </th>
                            <th className="px-4 py-2 text-[10px] font-bold text-slate-500 uppercase tracking-wider">
                              {t("common.error") || "Error"}
                            </th>
                          </tr>
                        </thead>
                        <tbody className="divide-y divide-slate-100">
                          {uploadResult.errors.map((err, idx) => (
                            <tr key={idx} className="hover:bg-slate-50">
                              <td className="px-4 py-2 text-xs font-mono text-slate-600">
                                {err.line_number}
                              </td>
                              <td className="px-4 py-2 text-xs font-mono text-slate-600">
                                {err.employee_id}
                              </td>
                              <td className="px-4 py-2 text-xs text-rose-600">
                                {err.error_message}
                              </td>
                            </tr>
                          ))}
                        </tbody>
                      </table>
                    </div>
                  </div>
                )}

                {/* Success Message */}
                {uploadResult.success_count > 0 && (
                  <div className="flex items-center gap-3 bg-emerald-50 border border-emerald-200 rounded-xl p-4">
                    <CheckCircle2 className="w-5 h-5 text-emerald-600 flex-shrink-0" />
                    <p className="text-xs font-bold text-emerald-700 uppercase tracking-wide">
                      {t("approvals.bulk_upload.success_message", { count: uploadResult.success_count }) ||
                        `${uploadResult.success_count} OT claims entered successfully`}
                    </p>
                  </div>
                )}
              </div>
            )}

            {/* Action Buttons */}
            <div className="flex justify-end gap-3 pt-2">
              {uploadResult ? (
                <button
                  onClick={() => {
                    resetForm();
                    onOpenChange(false);
                  }}
                  className="px-6 py-2 rounded-xl bg-primary-600 text-white text-xs font-bold uppercase tracking-wider shadow-lg hover:bg-primary-700"
                >
                  {t("common.close") || "Close"}
                </button>
              ) : (
                <button
                  onClick={handleClose}
                  className="px-4 py-2 rounded-xl text-xs font-bold uppercase tracking-wider text-slate-600 hover:bg-slate-100"
                >
                  {t("common.cancel") || "Cancel"}
                </button>
              )}
            </div>
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
