import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import { useQueryClient } from "@tanstack/react-query";
import { Upload, FileText, Loader2, CheckCircle2, AlertCircle, RefreshCw, X, Download } from "lucide-react";
import Button from "@/components/ui/button";
import Modal from "@/components/ui/modal";
import { Typography } from "@/components/ui/typography";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";
import { Table, TableHeader, TableBody, TableRow, TableHead, TableCell } from "@/components/ui/table";
import { cn } from "@/lib/utils";
import {
  uploadTerminationImportPreview,
  confirmTerminationImport,
  downloadTerminationTemplate,
  TerminationPreviewResponse,
  ConfirmTerminationRow
} from "@/lib/api/employees";

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
  const { t } = useTranslation();
  const queryClient = useQueryClient();

  const [file, setFile] = useState<File | null>(null);
  const [loading, setLoading] = useState(false);
  const [confirming, setConfirming] = useState(false);
  const [step, setStep] = useState<1 | 2>(1);
  const [previewData, setPreviewData] = useState<TerminationPreviewResponse | null>(null);
  const [activeTab, setActiveTab] = useState<"valid" | "invalid">("valid");
  const [importSuccess, setImportSuccess] = useState(false);
  const [successCount, setSuccessCount] = useState(0);

  const handleDownloadTemplate = async () => {
    try {
      await downloadTerminationTemplate(wsId);
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

  const handleConfirm = async () => {
    if (!previewData || previewData.valid_rows.length === 0) return;
    setConfirming(true);
    try {
      const rows: ConfirmTerminationRow[] = previewData.valid_rows.map((row) => ({
        employment_id: row.employment_id,
        termination_date: row.termination_date,
      }));
      const res = await confirmTerminationImport(wsId, rows);
      setSuccessCount(res.success_count);
      setImportSuccess(true);
      void queryClient.invalidateQueries();
    } catch (err) {
      console.error("Failed to confirm termination import:", err);
    } finally {
      setConfirming(false);
    }
  };

  const exportErrorsToCSV = () => {
    if (!previewData || previewData.invalid_rows.length === 0) return;
    const headers = ["Dòng", "Mã nhân viên", "Ngày nghỉ việc", "Lý do lỗi"];
    const rows = previewData.invalid_rows.map((r) => [
      r.row_number,
      r.employee_code ?? "",
      r.raw_termination_date ?? "",
      `"${r.reason.replace(/"/g, '""')}"`
    ]);
    const csvContent = "\uFEFF" + [headers.join(","), ...rows.map((e) => e.join(","))].join("\n");
    const blob = new Blob([csvContent], { type: "text/csv;charset=utf-8;" });
    const url = URL.createObjectURL(blob);
    const link = document.createElement("a");
    link.setAttribute("href", url);
    link.setAttribute("download", `loi_import_nghi_viec_${Date.now()}.csv`);
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
  };

  const handleClose = () => {
    setStep(1);
    setFile(null);
    setPreviewData(null);
    setImportSuccess(false);
    setSuccessCount(0);
    onClose();
  };

  return (
    <Modal
      isOpen={isOpen}
      onClose={handleClose}
      title="IMPORT DANH SÁCH NGHỈ VIỆC"
      subtitle="Chấm dứt hợp đồng lao động hàng loạt từ tệp Excel"
      icon={<Upload size={24} />}
      maxWidth={step === 1 ? "lg" : "xl"}
    >
      {importSuccess ? (
        <div className="flex flex-col items-center justify-center py-10 space-y-6 text-center animate-in zoom-in-95 duration-500">
          <div className="w-20 h-20 bg-emerald-50 dark:bg-emerald-950/30 rounded-3xl flex items-center justify-center text-emerald-500 shadow-lg border border-emerald-100 dark:border-emerald-900/50">
            <CheckCircle2 size={40} />
          </div>
          <div>
            <Typography variant="h3" className="font-extrabold text-slate-800 dark:text-slate-200 uppercase tracking-tight text-xl mb-2">
              Import Thành Công
            </Typography>
            <Typography variant="p" className="text-sm text-slate-500 dark:text-slate-400">
              Đã xử lý nghỉ việc thành công cho <span className="font-bold text-emerald-600 dark:text-emerald-400">{successCount}</span> nhân viên.
            </Typography>
          </div>
          <Button className="px-10 mt-4 shadow-sm" variant="secondary" onClick={handleClose}>
            ĐÓNG
          </Button>
        </div>
      ) : step === 1 ? (
        <div className="space-y-6 pt-2 animate-in fade-in duration-500">
          {/* File Requirements & Guidelines */}
          <div className="bg-primary-50 dark:bg-primary-950/50 rounded-2xl p-5 border border-primary-200 dark:border-primary-800/50 flex gap-4">
            <div className="w-9 h-9 rounded-xl bg-white dark:bg-slate-900 flex items-center justify-center text-primary-600 dark:text-primary-400 shadow-sm border border-primary-200 dark:border-primary-800 shrink-0">
              <AlertCircle size={18} />
            </div>
            <div className="text-xs text-primary-800 dark:text-primary-200 leading-relaxed font-medium pt-0.5 flex-1">
              <Typography variant="p" className="font-bold uppercase tracking-[0.2em] mb-2 text-slate-700 dark:text-slate-300 block">
                Yêu cầu định dạng tệp tin
              </Typography>
              <Typography variant="p" className="text-slate-500 dark:text-slate-400 text-[11px] italic mb-3 block">
                Hệ thống chỉ chấp nhận định dạng Excel (.xlsx, .xls). Các cột trong tệp tin phải bao gồm:
              </Typography>
              <ul className="space-y-3 mt-2 text-slate-700 dark:text-slate-300 font-medium text-xs">
                <li className="flex flex-col space-y-1">
                  <div className="flex items-center gap-2">
                    <div className="w-1.5 h-1.5 bg-primary-500 dark:bg-primary-400 rounded-full shrink-0" />
                    <span className="font-mono text-[11px] font-bold bg-white dark:bg-slate-900 px-2 py-0.5 rounded border border-slate-200 dark:border-slate-800 text-primary-700 dark:text-primary-400 shadow-sm">
                      employee_code
                    </span>
                    <span className="text-xs font-bold text-slate-800 dark:text-slate-200">— Mã nhân viên</span>
                    <span className="text-[9px] font-bold px-2 py-0.5 rounded-full uppercase tracking-wider bg-rose-50 dark:bg-rose-950/30 text-rose-600 dark:text-rose-400 border border-rose-100 dark:border-rose-900/50">
                      Bắt buộc
                    </span>
                  </div>
                  <Typography variant="p" className="pl-5 text-[11px] text-slate-500 dark:text-slate-400">
                    Mã số định danh của nhân viên cần chấm dứt hợp đồng.
                  </Typography>
                </li>
                <li className="flex flex-col space-y-1">
                  <div className="flex items-center gap-2">
                    <div className="w-1.5 h-1.5 bg-primary-500 dark:bg-primary-400 rounded-full shrink-0" />
                    <span className="font-mono text-[11px] font-bold bg-white dark:bg-slate-900 px-2 py-0.5 rounded border border-slate-200 dark:border-slate-800 text-primary-700 dark:text-primary-400 shadow-sm">
                      termination_date
                    </span>
                    <span className="text-xs font-bold text-slate-800 dark:text-slate-200">— Ngày nghỉ việc</span>
                    <span className="text-[9px] font-bold px-2 py-0.5 rounded-full uppercase tracking-wider bg-rose-50 dark:bg-rose-950/30 text-rose-600 dark:text-rose-400 border border-rose-100 dark:border-rose-900/50">
                      Bắt buộc
                    </span>
                  </div>
                  <div className="pl-5 space-y-1">
                    <Typography variant="p" className="text-[11px] text-slate-500 dark:text-slate-400">
                      Ngày cuối cùng làm việc của nhân viên tại công ty.
                    </Typography>
                    <span className="inline-block text-[9px] font-mono font-bold text-amber-600 dark:text-amber-400 bg-amber-50 dark:bg-amber-950/20 px-1.5 py-0.5 rounded">
                      Định dạng: YYYY-MM-DD hoặc DD/MM/YYYY
                    </span>
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
                  <FileText size={14} /> Tải tệp tin mẫu Excel
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
              aria-label="Tải lên tệp"
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
                <Typography variant="p" className="font-bold text-slate-800 dark:text-slate-200 mb-1 text-lg tracking-tight uppercase">
                  {file.name}
                </Typography>
                <Typography variant="p" className="text-[11px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">
                  {(file.size / 1024).toFixed(1)} KB — TỆP SẴN SÀNG
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
                  Chọn tệp khác
                </Button>
              </div>
            ) : (
              <div className="text-center group-hover:scale-105 transition-transform duration-500">
                <div className="w-16 h-16 bg-white dark:bg-slate-900 rounded-2xl flex items-center justify-center mx-auto mb-5 shadow-lg border border-slate-200 dark:border-slate-700 -rotate-3 group-hover:rotate-0 transition-all">
                  <Upload className="text-slate-400 dark:text-slate-500" size={28} />
                </div>
                <Typography variant="p" className="font-bold text-slate-800 dark:text-slate-200 mb-1 text-lg tracking-tight uppercase">
                  Kéo thả tệp tin hoặc Nhấn để tìm kiếm
                </Typography>
                <Typography variant="p" className="text-[11px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">
                  Hỗ trợ định dạng .xlsx, .xls tối đa 10MB
                </Typography>
              </div>
            )}
          </label>

          {/* Actions */}
          <div className="flex gap-4 pt-3 border-t border-slate-200 dark:border-slate-800">
            <Button className="flex-1" onClick={handleClose} size="lg" variant="outline">
              HỦY
            </Button>
            <Button
              className="flex-[1.5]"
              disabled={!file || loading}
              loading={loading}
              onClick={handleUpload}
              size="lg"
              variant="primary"
            >
              {loading ? "ĐANG TẢI LÊN..." : "TIẾP TỤC"}
            </Button>
          </div>
        </div>
      ) : (
        <div className="space-y-6 pt-2 animate-in fade-in duration-500">
          {/* Summary status */}
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div className="bg-slate-50 dark:bg-slate-800/40 p-4 rounded-xl border border-slate-100 dark:border-slate-800">
              <span className="text-[10px] font-bold text-slate-400 uppercase tracking-wider block mb-1">Tổng Số Dòng</span>
              <span className="text-2xl font-extrabold text-slate-700 dark:text-slate-200">
                {(previewData?.valid_rows.length ?? 0) + (previewData?.invalid_rows.length ?? 0)}
              </span>
            </div>
            <div className="bg-emerald-50/50 dark:bg-emerald-950/20 p-4 rounded-xl border border-emerald-100 dark:border-emerald-900/30">
              <span className="text-[10px] font-bold text-emerald-500 uppercase tracking-wider block mb-1">Hợp Lệ (Sẽ Import)</span>
              <span className="text-2xl font-extrabold text-emerald-600 dark:text-emerald-400">
                {previewData?.valid_rows.length ?? 0}
              </span>
            </div>
            <div className="bg-rose-50/50 dark:bg-rose-950/20 p-4 rounded-xl border border-rose-100 dark:border-rose-900/30">
              <span className="text-[10px] font-bold text-rose-500 uppercase tracking-wider block mb-1">Không Hợp Lệ (Lỗi)</span>
              <span className="text-2xl font-extrabold text-rose-600 dark:text-rose-400">
                {previewData?.invalid_rows.length ?? 0}
              </span>
            </div>
          </div>

          {/* Preview Tabs */}
          <Tabs value={activeTab} onValueChange={(val) => setActiveTab(val as "valid" | "invalid")}>
            <TabsList>
              <TabsTrigger value="valid">
                HỢP LỆ ({previewData?.valid_rows.length ?? 0})
              </TabsTrigger>
              <TabsTrigger value="invalid">
                LỖI ({previewData?.invalid_rows.length ?? 0})
              </TabsTrigger>
            </TabsList>

            <TabsContent value="valid">
              {previewData && previewData.valid_rows.length > 0 ? (
                <div className="max-h-[350px] overflow-y-auto custom-scrollbar border border-slate-100 dark:border-slate-800 rounded-3xl">
                  <Table>
                    <TableHeader>
                      <TableRow>
                        <TableHead className="w-16 text-center">Dòng</TableHead>
                        <TableHead>Mã NV</TableHead>
                        <TableHead>Họ và Tên</TableHead>
                        <TableHead>Ngày Vào Làm</TableHead>
                        <TableHead>Ngày Nghỉ Việc</TableHead>
                        <TableHead>Phép Năm Tạm Tính (Cũ &rarr; Mới)</TableHead>
                      </TableRow>
                    </TableHeader>
                    <TableBody>
                      {previewData.valid_rows.map((row) => (
                        <TableRow key={row.row_number}>
                          <TableCell className="text-center font-bold text-slate-400">{row.row_number}</TableCell>
                          <TableCell className="font-mono font-bold text-slate-700 dark:text-slate-300">
                            {row.employee_code}
                          </TableCell>
                          <TableCell className="font-semibold text-slate-800 dark:text-slate-200">
                            {row.full_name}
                          </TableCell>
                          <TableCell>{new Date(row.start_date).toLocaleDateString("vi-VN")}</TableCell>
                          <TableCell className="font-bold text-slate-700 dark:text-slate-300">
                            {new Date(row.termination_date).toLocaleDateString("vi-VN")}
                          </TableCell>
                          <TableCell>
                            {row.prorated_leaves.length > 0 ? (
                              <div className="flex flex-wrap gap-1.5">
                                {row.prorated_leaves.map((p) => (
                                  <span
                                    key={p.leave_type_id}
                                    className="inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full text-[10px] font-bold uppercase border bg-slate-50 dark:bg-slate-800 border-slate-200 dark:border-slate-700 text-slate-600 dark:text-slate-400"
                                  >
                                    <span className="font-mono text-primary-600 dark:text-primary-400">
                                      {p.leave_type_code}
                                    </span>
                                    <span>
                                      {p.old_entitlement} &rarr; {p.new_entitlement} ngày
                                    </span>
                                  </span>
                                ))}
                              </div>
                            ) : (
                              <span className="text-[10px] text-slate-400 italic">Không có thay đổi phép</span>
                            )}
                          </TableCell>
                        </TableRow>
                      ))}
                    </TableBody>
                  </Table>
                </div>
              ) : (
                <div className="flex flex-col items-center justify-center py-10 border border-dashed border-slate-200 dark:border-slate-800 rounded-3xl">
                  <Typography variant="p" className="text-slate-400 dark:text-slate-600 text-sm font-semibold italic">
                    Không có dòng dữ liệu hợp lệ nào
                  </Typography>
                </div>
              )}
            </TabsContent>

            <TabsContent value="invalid">
              {previewData && previewData.invalid_rows.length > 0 ? (
                <div className="space-y-4">
                  <div className="flex justify-between items-center">
                    <Typography variant="p" className="text-xs text-slate-500 dark:text-slate-400 font-medium">
                      Danh sách các hàng có dữ liệu không hợp lệ. Vui lòng tải file lỗi về để sửa và import lại.
                    </Typography>
                    <Button
                      size="sm"
                      variant="outline"
                      className="border-rose-200 dark:border-rose-800 text-rose-600 dark:text-rose-400 hover:text-rose-700 dark:hover:text-rose-300 hover:bg-rose-50 dark:hover:bg-rose-950 font-bold"
                      onClick={exportErrorsToCSV}
                    >
                      <Download size={14} className="mr-1.5" /> Xuất file lỗi (.csv)
                    </Button>
                  </div>
                  <div className="max-h-[350px] overflow-y-auto custom-scrollbar border border-slate-100 dark:border-slate-800 rounded-3xl">
                    <Table>
                      <TableHeader>
                        <TableRow>
                          <TableHead className="w-16 text-center">Dòng</TableHead>
                          <TableHead>Mã NV Tạm Tính</TableHead>
                          <TableHead>Ngày Tạm Tính</TableHead>
                          <TableHead>Lý Do Lỗi</TableHead>
                        </TableRow>
                      </TableHeader>
                      <TableBody>
                        {previewData.invalid_rows.map((row) => (
                          <TableRow key={row.row_number} className="hover:bg-rose-50/20 dark:hover:bg-rose-950/5">
                            <TableCell className="text-center font-bold text-rose-400">{row.row_number}</TableCell>
                            <TableCell className="font-mono text-slate-600 dark:text-slate-400">
                              {row.employee_code || <span className="italic text-slate-400">Trống</span>}
                            </TableCell>
                            <TableCell className="text-slate-600 dark:text-slate-400">
                              {row.raw_termination_date || <span className="italic text-slate-400">Trống</span>}
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
                  <Typography variant="p" className="text-slate-400 dark:text-slate-600 text-sm font-semibold italic">
                    Không có dòng lỗi nào
                  </Typography>
                </div>
              )}
            </TabsContent>
          </Tabs>

          {/* Step 2 Actions */}
          <div className="flex gap-4 pt-3 border-t border-slate-200 dark:border-slate-800">
            <Button
              className="flex-1"
              variant="outline"
              onClick={() => {
                setStep(1);
                setFile(null);
                setPreviewData(null);
              }}
            >
              HỦY VÀ QUAY LẠI
            </Button>
            <Button
              className="flex-[1.5]"
              variant="primary"
              disabled={confirming || !previewData || previewData.valid_rows.length === 0}
              loading={confirming}
              onClick={handleConfirm}
            >
              {confirming
                ? "ĐANG XỬ LÝ..."
                : `XÁC NHẬN IMPORT (${previewData?.valid_rows.length ?? 0} DÒNG)`}
            </Button>
          </div>
        </div>
      )}
    </Modal>
  );
}
