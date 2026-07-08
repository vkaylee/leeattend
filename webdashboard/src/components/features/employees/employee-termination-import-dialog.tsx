import { useQueryClient } from "@tanstack/react-query";
import { Upload, FileText, Loader2, CheckCircle2, AlertCircle, RefreshCw, Download } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import Modal from "@/components/ui/modal";
import { Table, TableHeader, TableBody, TableRow, TableHead, TableCell } from "@/components/ui/table";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";
import { Typography } from "@/components/ui/typography";
import {
  uploadTerminationImportPreview,
  confirmTerminationImport,
  downloadTerminationTemplate,
  TerminationPreviewResponse,
  ConfirmTerminationRow
} from "@/lib/api/employees";
import { fetchJobStatus } from "@/lib/api/leaves";
import { cn } from "@/lib/utils";

interface EmployeeTerminationImportDialogProps {
  isOpen: boolean;
  onClose: () => void;
  wsId: string;
}

export default function EmployeeTerminationImportDialog({
  isOpen,
  onClose,
  wsId
}: EmployeeTerminationImportDialogProps) {
  const { t, i18n } = useTranslation("employees");
  const queryClient = useQueryClient();

  const [file, setFile] = useState<File | null>(null);
  const [loading, setLoading] = useState(false);
  const [confirming, setConfirming] = useState(false);
  const [step, setStep] = useState<1 | 2>(1);
  const [previewData, setPreviewData] = useState<TerminationPreviewResponse | null>(null);
  const [activeTab, setActiveTab] = useState<"valid" | "invalid">("valid");
  const [importSuccess, setImportSuccess] = useState(false);
  const [successCount, setSuccessCount] = useState(0);
  const [jobId, setJobId] = useState<string | null>(null);
  const [, setJobStatus] = useState<string | null>(null);
  const [jobError, setJobError] = useState<string | null>(null);

  const handleDownloadTemplate = async () => {
    try {
      await downloadTerminationTemplate(wsId, i18n.language);
    } catch (err) {
      console.error("Failed to download template:", err);
    }
  };

  const handleUpload = async () => {
    if (!file) return;
    setLoading(true);
    try {
      const data = await uploadTerminationImportPreview(wsId, file);
      setPreviewData(data);
      setStep(2);
      setActiveTab(data.valid_rows.length > 0 ? "valid" : "invalid");
    } catch (err) {
      console.error("Failed to upload file for preview:", err);
    } finally {
      setLoading(false);
    }
  };

  const pollJob = async (id: string, initialCount: number) => {
    const POLL_INTERVAL = 2000;
    const MAX_POLLS = 60;
    
    for (let i = 0; i < MAX_POLLS; i++) {
      try {
        const statusRes = await fetchJobStatus(wsId, id);
        setJobStatus(statusRes.status);
        if (statusRes.status === "completed") {
          setSuccessCount(initialCount);
          setImportSuccess(true);
          void queryClient.invalidateQueries();
          setConfirming(false);
          setJobId(null);
          return;
        }
        if (statusRes.status === "failed") {
          setJobError(statusRes.error_message || "Đã xảy ra lỗi khi xử lý nghỉ việc.");
          setConfirming(false);
          setJobId(null);
          return;
        }
      } catch (err) {
        console.error("Error polling job status:", err);
      }
      await new Promise((resolve) => setTimeout(resolve, POLL_INTERVAL));
    }
    setJobError("Hết thời gian chờ xử lý tác vụ nghỉ việc. Vui lòng tải lại trang để kiểm tra.");
    setConfirming(false);
    setJobId(null);
  };

  const handleConfirm = async () => {
    if (!previewData || previewData.valid_rows.length === 0) return;
    setConfirming(true);
    setJobError(null);
    try {
      const rows: ConfirmTerminationRow[] = previewData.valid_rows.map((row) => ({
        employment_id: row.employment_id,
        termination_date: row.termination_date,
        termination_reason: row.termination_reason,
        termination_notes: row.termination_notes,
      }));
      const res = await confirmTerminationImport(wsId, rows);
      if (res.job_id) {
        setJobId(res.job_id);
        setJobStatus("queued");
        void pollJob(res.job_id, res.success_count);
      } else {
        setSuccessCount(res.success_count);
        setImportSuccess(true);
        void queryClient.invalidateQueries();
        setConfirming(false);
      }
    } catch (err) {
      console.error("Failed to confirm termination import:", err);
      setJobError("Lỗi kết nối hoặc yêu cầu không hợp lệ.");
      setConfirming(false);
    }
  };

  const exportErrorsToExcel = async () => {
    if (!previewData || previewData.invalid_rows.length === 0) return;
    try {
      const ExcelJS = await import("exceljs").then((m) =>
        "default" in m ? m.default : m
      );
      const { saveAs } = await import("file-saver");

      const workbook = new ExcelJS.Workbook();
      workbook.creator = "LeeAttend";
      workbook.created = new Date();

      const worksheet = workbook.addWorksheet("Lỗi Import");

      worksheet.columns = [
        { header: "Dòng", key: "row_number", width: 10 },
        { header: "Mã nhân viên", key: "employee_code", width: 20 },
        { header: "Ngày nghỉ việc", key: "termination_date", width: 20 },
        { header: "Lý do lỗi", key: "reason", width: 50 },
      ];

      const headerRow = worksheet.getRow(1);
      headerRow.eachCell((cell) => {
        cell.font = { bold: true, color: { argb: "FFFFFFFF" } };
        cell.fill = {
          type: "pattern",
          pattern: "solid",
          fgColor: { argb: "FFC00000" },
        };
        cell.alignment = { horizontal: "center", vertical: "middle" };
      });

      previewData.invalid_rows.forEach((r) => {
        worksheet.addRow({
          row_number: r.row_number,
          employee_code: r.employee_code ?? "",
          termination_date: r.raw_termination_date ?? "",
          reason: r.reason,
        });
      });

      worksheet.eachRow((row, rowNum) => {
        if (rowNum > 1) {
          const cell1 = row.getCell(1);
          cell1.alignment = { horizontal: "center" };
          cell1.font = { bold: true, color: { argb: "FFC00000" } };
        }
      });

      const buffer = await workbook.xlsx.writeBuffer();
      const blob = new Blob([buffer], {
        type: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
      });
      saveAs(blob, `loi_import_nghi_viec_${Date.now()}.xlsx`);
    } catch (err) {
      console.error("Failed to export errors to Excel:", err);
    }
  };

  const handleClose = () => {
    setStep(1);
    setFile(null);
    setPreviewData(null);
    setImportSuccess(false);
    setSuccessCount(0);
    setJobId(null);
    setJobStatus(null);
    setJobError(null);
    onClose();
  };

  return (
    <Modal
      icon={<Upload size={24} />}
      isOpen={isOpen}
      maxWidth={step === 1 ? "lg" : "xl"}
      onClose={handleClose}
      subtitle={t("employees.bulk_termination.dialog_subtitle")}
      title={t("employees.bulk_termination.dialog_title")}
    >
      {importSuccess ? (
        <div className="flex flex-col items-center justify-center py-10 space-y-6 text-center animate-in zoom-in-95 duration-500">
          <div className="w-20 h-20 bg-emerald-50 dark:bg-emerald-950/30 rounded-3xl flex items-center justify-center text-emerald-500 shadow-lg border border-emerald-100 dark:border-emerald-900/50">
            <CheckCircle2 size={40} />
          </div>
          <div>
            <Typography className="font-extrabold text-slate-800 dark:text-slate-200 uppercase tracking-tight text-xl mb-2" variant="h3">
              {t("employees.bulk_termination.import_success_title")}
            </Typography>
            <Typography className="text-sm text-slate-500 dark:text-slate-400" variant="p">
              {t("employees.bulk_termination.import_success_desc", { count: successCount })}
            </Typography>
          </div>
          <Button className="px-10 mt-4 shadow-sm" onClick={handleClose} variant="secondary">
            {t("employees.bulk_termination.close_btn")}
          </Button>
        </div>
      ) : jobError ? (
        <div className="flex flex-col items-center justify-center py-10 space-y-6 text-center animate-in zoom-in-95 duration-500">
          <div className="w-20 h-20 bg-rose-50 dark:bg-rose-950/30 rounded-3xl flex items-center justify-center text-rose-500 shadow-lg border border-rose-100 dark:border-rose-900/50">
            <AlertCircle size={40} />
          </div>
          <div>
            <Typography className="font-extrabold text-slate-800 dark:text-slate-200 uppercase tracking-tight text-xl mb-2 text-rose-600" variant="h3">
              {t("employees.bulk_termination.processing_error_title")}
            </Typography>
            <Typography className="text-sm text-slate-500 dark:text-slate-400 max-w-md" variant="p">
              {jobError}
            </Typography>
          </div>
          <Button
            className="px-10 mt-4 shadow-sm"
            onClick={() => {
              setJobError(null);
            }}
            variant="primary"
          >
            {t("employees.bulk_termination.back_btn")}
          </Button>
        </div>
      ) : confirming ? (
        <div className="flex flex-col items-center justify-center py-12 space-y-6 text-center animate-in fade-in duration-500">
          <div className="relative flex items-center justify-center">
            <Loader2 className="animate-spin text-primary-500" size={48} />
            <RefreshCw className="absolute text-primary-300 animate-pulse" size={24} />
          </div>
          <div>
            <Typography className="font-extrabold text-slate-800 dark:text-slate-200 uppercase tracking-tight text-xl mb-2" variant="h3">
              {t("employees.bulk_termination.processing_data_title")}
            </Typography>
            <Typography className="text-sm text-slate-500 dark:text-slate-400 max-w-md" variant="p">
              {t("employees.bulk_termination.processing_data_desc")}
            </Typography>
            {!!jobId && (
              <span className="inline-block mt-3 text-[10px] font-mono font-bold text-slate-400 bg-slate-100 dark:bg-slate-800 px-2.5 py-1 rounded-full">
                {t("employees.bulk_termination.task_id_label", { id: jobId })}
              </span>
            )}
          </div>
        </div>
      ) : step === 1 ? (
        <div className="space-y-6 pt-2 animate-in fade-in duration-500">
          {/* File Requirements & Guidelines */}
          <div className="bg-primary-50 dark:bg-primary-950/50 rounded-2xl p-5 border border-primary-200 dark:border-primary-800/50 flex gap-4">
            <div className="w-9 h-9 rounded-xl bg-white dark:bg-slate-900 flex items-center justify-center text-primary-600 dark:text-primary-400 shadow-sm border border-primary-200 dark:border-primary-800 shrink-0">
              <AlertCircle size={18} />
            </div>
            <div className="text-xs text-primary-800 dark:text-primary-200 leading-relaxed font-medium pt-0.5 flex-1">
              <Typography className="font-bold uppercase tracking-[0.2em] mb-2 text-slate-700 dark:text-slate-300 block" variant="p">
                {t("employees.bulk_termination.format_req_title")}
              </Typography>
              <Typography className="text-slate-500 dark:text-slate-400 text-[11px] italic mb-3 block" variant="p">
                {t("employees.bulk_termination.format_req_desc")}
              </Typography>
              <ul className="space-y-3 mt-2 text-slate-700 dark:text-slate-300 font-medium text-xs">
                <li className="flex flex-col space-y-1">
                  <div className="flex items-center gap-2">
                    <div className="w-1.5 h-1.5 bg-primary-500 dark:bg-primary-400 rounded-full shrink-0" />
                    <span className="font-mono text-[11px] font-bold bg-white dark:bg-slate-900 px-2 py-0.5 rounded border border-slate-200 dark:border-slate-800 text-primary-700 dark:text-primary-400 shadow-sm">
                      employee_code
                    </span>
                    <span className="text-xs font-bold text-slate-800 dark:text-slate-200">— {t("employees.bulk_termination.employee_code_title")}</span>
                    <span className="text-[9px] font-bold px-2 py-0.5 rounded-full uppercase tracking-wider bg-rose-50 dark:bg-rose-950/30 text-rose-600 dark:text-rose-400 border border-rose-100 dark:border-rose-900/50">
                      {t("employees.bulk_termination.required_badge")}
                    </span>
                  </div>
                  <Typography className="pl-5 text-[11px] text-slate-500 dark:text-slate-400" variant="p">
                    {t("employees.bulk_termination.employee_code_desc")}
                  </Typography>
                </li>
                <li className="flex flex-col space-y-1">
                  <div className="flex items-center gap-2">
                    <div className="w-1.5 h-1.5 bg-primary-500 dark:bg-primary-400 rounded-full shrink-0" />
                    <span className="font-mono text-[11px] font-bold bg-white dark:bg-slate-900 px-2 py-0.5 rounded border border-slate-200 dark:border-slate-800 text-primary-700 dark:text-primary-400 shadow-sm">
                      termination_date
                    </span>
                    <span className="text-xs font-bold text-slate-800 dark:text-slate-200">— {t("employees.bulk_termination.termination_date_title")}</span>
                    <span className="text-[9px] font-bold px-2 py-0.5 rounded-full uppercase tracking-wider bg-rose-50 dark:bg-rose-950/30 text-rose-600 dark:text-rose-400 border border-rose-100 dark:border-rose-900/50">
                      {t("employees.bulk_termination.required_badge")}
                    </span>
                  </div>
                  <div className="pl-5 space-y-1">
                    <Typography className="text-[11px] text-slate-500 dark:text-slate-400" variant="p">
                      {t("employees.bulk_termination.termination_date_desc")}
                    </Typography>
                    <span className="inline-block text-[9px] font-mono font-bold text-amber-600 dark:text-amber-400 bg-amber-50 dark:bg-amber-950/20 px-1.5 py-0.5 rounded">
                      {t("employees.bulk_termination.date_format_tip")}
                    </span>
                  </div>
                </li>
                <li className="flex flex-col space-y-1">
                  <div className="flex items-center gap-2">
                    <div className="w-1.5 h-1.5 bg-primary-500 dark:bg-primary-400 rounded-full shrink-0" />
                    <span className="font-mono text-[11px] font-bold bg-white dark:bg-slate-900 px-2 py-0.5 rounded border border-slate-200 dark:border-slate-800 text-primary-700 dark:text-primary-400 shadow-sm">
                      termination_reason
                    </span>
                    <span className="text-xs font-bold text-slate-800 dark:text-slate-200">— {t("employees.bulk_termination.termination_reason_title")}</span>
                    <span className="text-[9px] font-bold px-2 py-0.5 rounded-full uppercase tracking-wider bg-rose-50 dark:bg-rose-950/30 text-rose-600 dark:text-rose-400 border border-rose-100 dark:border-rose-900/50">
                      {t("employees.bulk_termination.required_badge")}
                    </span>
                  </div>
                  <div className="pl-5 space-y-1">
                    <Typography className="text-[11px] text-slate-500 dark:text-slate-400" variant="p">
                      {t("employees.bulk_termination.termination_reason_desc")}
                    </Typography>
                    <span className="inline-block text-[9px] font-mono font-bold text-amber-600 dark:text-amber-400 bg-amber-50 dark:bg-amber-950/20 px-1.5 py-0.5 rounded">
                      {t("employees.bulk_termination.termination_reason_valid_values")}
                    </span>
                  </div>
                </li>
                <li className="flex flex-col space-y-1">
                  <div className="flex items-center gap-2">
                    <div className="w-1.5 h-1.5 bg-primary-500 dark:bg-primary-400 rounded-full shrink-0" />
                    <span className="font-mono text-[11px] font-bold bg-white dark:bg-slate-900 px-2 py-0.5 rounded border border-slate-200 dark:border-slate-800 text-primary-700 dark:text-primary-400 shadow-sm">
                      termination_notes
                    </span>
                    <span className="text-xs font-bold text-slate-800 dark:text-slate-200">— {t("employees.bulk_termination.termination_notes_title")}</span>
                    <span className="text-[9px] font-bold px-2 py-0.5 rounded-full uppercase tracking-wider bg-slate-50 dark:bg-slate-800 text-slate-500 border border-slate-200 dark:border-slate-700">
                      {t("employees.bulk_termination.optional_badge")}
                    </span>
                  </div>
                  <div className="pl-5 space-y-1">
                    <Typography className="text-[11px] text-slate-500 dark:text-slate-400" variant="p">
                      {t("employees.bulk_termination.termination_notes_desc")}
                    </Typography>
                  </div>
                </li>
              </ul>
              <div className="mt-4 flex items-center">
                <Button
                  className="text-[10px] font-bold text-primary-600 dark:text-primary-400 uppercase tracking-widest hover:text-primary-700 hover:underline flex items-center gap-1.5 cursor-pointer bg-transparent border-0 p-0 h-auto min-w-0"
                  onClick={handleDownloadTemplate}
                  type="button"
                  variant="ghost"
                >
                  <FileText size={14} /> {t("employees.bulk_termination.download_template_btn")}
                </Button>
              </div>
            </div>
          </div>

          {/* Drag & Drop File upload */}
          <label
            className={cn(
              "border-3 border-dashed rounded-2xl p-10 flex flex-col items-center justify-center transition-all cursor-pointer group active:scale-[0.98] relative overflow-hidden w-full",
              file
                ? "border-emerald-400 bg-emerald-50 dark:bg-emerald-950/20"
                : "border-slate-200 dark:border-slate-700 bg-slate-50 dark:bg-slate-800/30 hover:border-primary-400 hover:bg-primary-50 dark:hover:bg-primary-950/30"
            )}
            htmlFor="file-upload"
          >
            <input
              accept=".xlsx,.xls"
              aria-label={t("employees.bulk_termination.uploading_btn")}
              className="hidden"
              id="file-upload"
              onChange={(e) => {
                setFile(e.target.files?.[0] ?? null);
              }}
              type="file"
            />
            {file ? (
              <div className="text-center animate-in zoom-in-95 duration-500">
                <div className="w-16 h-16 bg-emerald-100 dark:bg-emerald-900 rounded-2xl flex items-center justify-center mx-auto mb-5 shadow-lg shadow-emerald-200/50 dark:shadow-emerald-900/30 rotate-3 group-hover:rotate-0 transition-transform">
                  <FileText className="text-emerald-600 dark:text-emerald-300" size={28} />
                </div>
                <Typography className="font-bold text-slate-800 dark:text-slate-200 mb-1 text-lg tracking-tight uppercase" variant="p">
                  {file.name}
                </Typography>
                <Typography className="text-[11px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest" variant="p">
                  {(file.size / 1024).toFixed(1)} KB — {t("employees.bulk_termination.file_ready_label")}
                </Typography>
                <Button
                  className="mt-5 border-rose-200 dark:border-rose-800 text-rose-600 dark:text-rose-400 hover:text-rose-700 dark:hover:text-rose-300 hover:bg-rose-50 dark:hover:bg-rose-950 shadow-sm hover:shadow-md"
                  onClick={(e) => {
                    e.stopPropagation();
                    setFile(null);
                  }}
                  size="sm"
                  variant="outline"
                >
                  {t("employees.bulk_termination.choose_other_file")}
                </Button>
              </div>
            ) : (
              <div className="text-center group-hover:scale-105 transition-transform duration-500">
                <div className="w-16 h-16 bg-white dark:bg-slate-900 rounded-2xl flex items-center justify-center mx-auto mb-5 shadow-lg border border-slate-200 dark:border-slate-700 -rotate-3 group-hover:rotate-0 transition-all">
                  <Upload className="text-slate-400 dark:text-slate-500" size={28} />
                </div>
                <Typography className="font-bold text-slate-800 dark:text-slate-200 mb-1 text-lg tracking-tight uppercase" variant="p">
                  {t("employees.bulk_termination.drag_drop_placeholder")}
                </Typography>
                <Typography className="text-[11px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest" variant="p">
                  {t("employees.bulk_termination.file_support_tip")}
                </Typography>
              </div>
            )}
          </label>

          {/* Actions */}
          <div className="flex gap-4 pt-3 border-t border-slate-200 dark:border-slate-800">
            <Button className="flex-1" onClick={handleClose} size="lg" variant="outline">
              {t("employees.bulk_termination.cancel_btn")}
            </Button>
            <Button
              className="flex-[1.5]"
              disabled={!file || loading}
              loading={loading}
              onClick={handleUpload}
              size="lg"
              variant="primary"
            >
              {loading ? t("employees.bulk_termination.uploading_btn") : t("employees.bulk_termination.continue_btn")}
            </Button>
          </div>
        </div>
      ) : (
        <div className="space-y-6 pt-2 animate-in fade-in duration-500">
          {/* Summary status */}
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div className="bg-slate-50 dark:bg-slate-800/40 p-4 rounded-xl border border-slate-100 dark:border-slate-800">
              <span className="text-[10px] font-bold text-slate-400 uppercase tracking-wider block mb-1">{t("employees.bulk_termination.total_rows")}</span>
              <span className="text-2xl font-extrabold text-slate-700 dark:text-slate-200">
                {(previewData?.valid_rows.length ?? 0) + (previewData?.invalid_rows.length ?? 0)}
              </span>
            </div>
            <div className="bg-emerald-50/50 dark:bg-emerald-950/20 p-4 rounded-xl border border-emerald-100 dark:border-emerald-900/30">
              <span className="text-[10px] font-bold text-emerald-500 uppercase tracking-wider block mb-1">{t("employees.bulk_termination.valid_rows_label")}</span>
              <span className="text-2xl font-extrabold text-emerald-600 dark:text-emerald-400">
                {previewData?.valid_rows.length ?? 0}
              </span>
            </div>
            <div className="bg-rose-50/50 dark:bg-rose-950/20 p-4 rounded-xl border border-rose-100 dark:border-rose-900/30">
              <span className="text-[10px] font-bold text-rose-500 uppercase tracking-wider block mb-1">{t("employees.bulk_termination.invalid_rows_label")}</span>
              <span className="text-2xl font-extrabold text-rose-600 dark:text-rose-400">
                {previewData?.invalid_rows.length ?? 0}
              </span>
            </div>
          </div>

          {/* Preview Tabs */}
          <Tabs onValueChange={(val) => { setActiveTab(val as "valid" | "invalid"); }} value={activeTab}>
            <TabsList>
              <TabsTrigger value="valid">
                {t("employees.bulk_termination.tab_valid", { count: previewData?.valid_rows.length ?? 0 })}
              </TabsTrigger>
              <TabsTrigger value="invalid">
                {t("employees.bulk_termination.tab_invalid", { count: previewData?.invalid_rows.length ?? 0 })}
              </TabsTrigger>
            </TabsList>

            <TabsContent value="valid">
              {previewData && previewData.valid_rows.length > 0 ? (
                <div className="max-h-[350px] overflow-y-auto custom-scrollbar border border-slate-100 dark:border-slate-800 rounded-3xl">
                  <Table>
                    <TableHeader>
                      <TableRow>
                        <TableHead className="w-16 text-center">{t("employees.bulk_termination.table_row")}</TableHead>
                        <TableHead>{t("employees.bulk_termination.table_code")}</TableHead>
                        <TableHead>{t("employees.bulk_termination.table_name")}</TableHead>
                        <TableHead>{t("employees.bulk_termination.table_hire_date")}</TableHead>
                        <TableHead>{t("employees.bulk_termination.table_termination_date")}</TableHead>
                        <TableHead>{t("employees.bulk_termination.table_termination_reason")}</TableHead>
                        <TableHead>{t("employees.bulk_termination.table_prorated_leaves")}</TableHead>
                      </TableRow>
                    </TableHeader>
                    <TableBody>
                      {previewData.valid_rows.map((row: any) => (
                        <TableRow key={row.row_number}>
                          <TableCell className="text-center font-bold text-slate-400">{row.row_number}</TableCell>
                          <TableCell className="font-mono font-bold text-slate-700 dark:text-slate-300">
                            {row.employee_code}
                          </TableCell>
                          <TableCell className="font-semibold text-slate-800 dark:text-slate-200">
                            {row.full_name}
                          </TableCell>
                          <TableCell>{new Date(row.start_date).toLocaleDateString(i18n.language === "vi" ? "vi-VN" : i18n.language === "zh" ? "zh-CN" : "en-US")}</TableCell>
                          <TableCell className="font-bold text-slate-700 dark:text-slate-300">
                            {new Date(row.termination_date).toLocaleDateString(i18n.language === "vi" ? "vi-VN" : i18n.language === "zh" ? "zh-CN" : "en-US")}
                          </TableCell>
                          <TableCell>
                            {row.termination_reason ? (
                              <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-bold uppercase border bg-slate-50 dark:bg-slate-800 border-slate-200 dark:border-slate-700 text-slate-600 dark:text-slate-400">
                                {t(`employees.terminate.reason.${row.termination_reason}`, { defaultValue: row.termination_reason })}
                              </span>
                            ) : (
                              <span className="text-slate-400 italic">—</span>
                            )}
                          </TableCell>
                          <TableCell>
                            {row.prorated_leaves.length > 0 ? (
                              <div className="flex flex-wrap gap-1.5">
                                {row.prorated_leaves.map((p: any) => (
                                  <span
                                    className="inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full text-[10px] font-bold uppercase border bg-slate-50 dark:bg-slate-800 border-slate-200 dark:border-slate-700 text-slate-600 dark:text-slate-400"
                                    key={p.leave_type_id}
                                  >
                                    <span className="font-mono text-primary-600 dark:text-primary-400">
                                      {p.leave_type_code}
                                    </span>
                                    <span>
                                      {p.old_entitlement} &rarr; {p.new_entitlement} {t("employees.bulk_termination.table_prorated_days_suffix")}
                                    </span>
                                  </span>
                                ))}
                              </div>
                            ) : (
                              <span className="text-[10px] text-slate-400 italic">{t("employees.bulk_termination.no_leave_changes")}</span>
                            )}
                          </TableCell>
                        </TableRow>
                      ))}
                    </TableBody>
                  </Table>
                </div>
              ) : (
                <div className="flex flex-col items-center justify-center py-10 border border-dashed border-slate-200 dark:border-slate-800 rounded-3xl">
                  <Typography className="text-slate-400 dark:text-slate-600 text-sm font-semibold italic" variant="p">
                    {t("employees.bulk_termination.no_valid_rows")}
                  </Typography>
                </div>
              )}
            </TabsContent>

            <TabsContent value="invalid">
              {previewData && previewData.invalid_rows.length > 0 ? (
                <div className="space-y-4">
                  <div className="flex justify-between items-center">
                    <Typography className="text-xs text-slate-500 dark:text-slate-400 font-medium" variant="p">
                      {t("employees.bulk_termination.invalid_rows_desc")}
                    </Typography>
                    <Button
                      className="border-rose-200 dark:border-rose-800 text-rose-600 dark:text-rose-400 hover:text-rose-700 dark:hover:text-rose-300 hover:bg-rose-50 dark:hover:bg-rose-950 font-bold"
                      onClick={exportErrorsToExcel}
                      size="sm"
                      variant="outline"
                    >
                      <Download className="mr-1.5" size={14} /> {t("employees.bulk_termination.export_errors_btn")}
                    </Button>
                  </div>
                  <div className="max-h-[350px] overflow-y-auto custom-scrollbar border border-slate-100 dark:border-slate-800 rounded-3xl">
                    <Table>
                      <TableHeader>
                        <TableRow>
                          <TableHead className="w-16 text-center">{t("employees.bulk_termination.table_row")}</TableHead>
                          <TableHead>{t("employees.bulk_termination.table_invalid_code")}</TableHead>
                          <TableHead>{t("employees.bulk_termination.table_invalid_date")}</TableHead>
                          <TableHead>{t("employees.bulk_termination.table_invalid_reason")}</TableHead>
                        </TableRow>
                      </TableHeader>
                      <TableBody>
                        {previewData.invalid_rows.map((row: any) => (
                          <TableRow className="hover:bg-rose-50/20 dark:hover:bg-rose-950/5" key={row.row_number}>
                            <TableCell className="text-center font-bold text-rose-400">{row.row_number}</TableCell>
                            <TableCell className="font-mono text-slate-600 dark:text-slate-400">
                              {row.employee_code || <span className="italic text-slate-400">{t("employees.bulk_termination.empty_label")}</span>}
                            </TableCell>
                            <TableCell className="text-slate-600 dark:text-slate-400">
                              {row.raw_termination_date || <span className="italic text-slate-400">{t("employees.bulk_termination.empty_label")}</span>}
                            </TableCell>
                            <TableCell className="font-bold text-rose-600 dark:text-rose-400 text-xs">
                              {row.reason}
                            </TableCell>
                          </TableRow>
                        ))}
                      </TableBody>
                    </Table>
                  </div>
                </div>
              ) : (
                <div className="flex flex-col items-center justify-center py-10 border border-dashed border-slate-200 dark:border-slate-800 rounded-3xl">
                  <Typography className="text-slate-400 dark:text-slate-600 text-sm font-semibold italic" variant="p">
                    {t("employees.bulk_termination.no_invalid_rows")}
                  </Typography>
                </div>
              )}
            </TabsContent>
          </Tabs>

          {/* Step 2 Actions */}
          <div className="flex gap-4 pt-3 border-t border-slate-200 dark:border-slate-800">
            <Button
              className="flex-1"
              onClick={() => {
                setStep(1);
                setFile(null);
                setPreviewData(null);
              }}
              variant="outline"
            >
              {t("employees.bulk_termination.cancel_back_btn")}
            </Button>
            <Button
              className="flex-[1.5]"
              disabled={confirming || !previewData || previewData.valid_rows.length === 0}
              loading={confirming}
              onClick={handleConfirm}
              variant="primary"
            >
              {confirming
                ? t("employees.bulk_termination.processing_btn")
                : t("employees.bulk_termination.confirm_import_btn", { count: previewData?.valid_rows.length ?? 0 })}
            </Button>
          </div>
        </div>
      )}
    </Modal>
  );
}
