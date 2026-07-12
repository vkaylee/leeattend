import { Download } from "lucide-react";
import { useTranslation } from "react-i18next";

import type { TerminationPreviewResponse } from "@/lib/api/employees";

import Button from "@/components/ui/button";
import { Table, TableHeader, TableBody, TableRow, TableHead, TableCell } from "@/components/ui/table";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";

interface Props {
  previewData: TerminationPreviewResponse | null;
  activeTab: "valid" | "invalid";
  confirming: boolean;
  onTabChange: (tab: "valid" | "invalid") => void;
  onExportErrors: () => void;
  onConfirm: () => void;
  onClose: () => void;
}

export default function ImportPreviewStep({
  previewData,
  activeTab,
  confirming,
  onTabChange,
  onExportErrors,
  onConfirm,
  onClose,
}: Props) {
  const { t } = useTranslation("employees");
  const total = (previewData?.valid_rows.length ?? 0) + (previewData?.invalid_rows.length ?? 0);
  const validCount = previewData?.valid_rows.length ?? 0;
  const invalidCount = previewData?.invalid_rows.length ?? 0;
  const hasPreview = previewData !== null && invalidCount > 0;

  return (
    <div className="space-y-6 pt-2 animate-in fade-in duration-500">
      {/* Summary cards */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <SummaryCard label={t("employees.bulk_termination.total_rows")} value={total} className="bg-slate-50 dark:bg-slate-800/40 border-slate-100 dark:border-slate-800 text-slate-700 dark:text-slate-200" />
        <SummaryCard label={t("employees.bulk_termination.valid_rows_label")} value={validCount} className="bg-emerald-50/50 dark:bg-emerald-950/20 border-emerald-100 dark:border-emerald-900/30 text-emerald-600 dark:text-emerald-400" />
        <SummaryCard label={t("employees.bulk_termination.invalid_rows_label")} value={invalidCount} className="bg-rose-50/50 dark:bg-rose-950/20 border-rose-100 dark:border-rose-900/30 text-rose-600 dark:text-rose-400" />
      </div>

      {/* Preview Tabs */}
      <Tabs onValueChange={(val) => onTabChange(val as "valid" | "invalid")} value={activeTab}>
        <TabsList>
          <TabsTrigger value="valid">
            {t("employees.bulk_termination.tab_valid", { count: validCount })}
          </TabsTrigger>
          <TabsTrigger value="invalid">
            {t("employees.bulk_termination.tab_invalid", { count: invalidCount })}
          </TabsTrigger>
        </TabsList>
        <TabsContent value="valid">
          <div className="border border-slate-200 dark:border-slate-800 rounded-xl overflow-hidden shadow-sm">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>{t("employees.bulk_termination.col_employee_code")}</TableHead>
                  <TableHead>{t("employees.bulk_termination.col_employee_name")}</TableHead>
                  <TableHead>{t("employees.bulk_termination.col_termination_date")}</TableHead>
                  <TableHead>{t("employees.bulk_termination.col_reason")}</TableHead>
                  <TableHead>{t("employees.bulk_termination.col_prorated_leave")}</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {previewData?.valid_rows.map((row, index) => (
                  <TableRow key={row.employment_id ?? index}>
                    <TableCell className="font-mono text-[13px] font-bold tracking-tight">{row.employee_code}</TableCell>
                    <TableCell className="font-medium">{(row as unknown as { full_name?: string }).full_name}</TableCell>
                    <TableCell>{row.termination_date}</TableCell>
                    <TableCell className="max-w-[200px] truncate">{row.termination_reason}</TableCell>
                    <TableCell>
                      {(row as unknown as Record<string, unknown>).prorated_leave_preview
                        ? t("employees.bulk_termination.yes")
                        : t("employees.bulk_termination.no")}
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </div>
        </TabsContent>
        <TabsContent value="invalid">
          <div className="border border-rose-200 dark:border-rose-800/50 rounded-xl overflow-hidden shadow-sm">
            {previewData && invalidCount > 0 && (
              <div className="flex justify-end px-4 pt-3">
                <Button
                  className="text-[10px] font-bold text-rose-600 dark:text-rose-400 uppercase tracking-widest hover:underline flex items-center gap-1.5 bg-transparent border-0 p-0 h-auto min-w-0 cursor-pointer"
                  onClick={onExportErrors}
                  type="button"
                  variant="ghost"
                >
                  <Download className="mr-1.5" size={14} /> {t("employees.bulk_termination.export_errors_btn")}
                </Button>
              </div>
            )}
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>{t("employees.bulk_termination.col_row")}</TableHead>
                  <TableHead>{t("employees.bulk_termination.col_employee_code")}</TableHead>
                  <TableHead>{t("employees.bulk_termination.col_error")}</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {previewData?.invalid_rows.map((row, index) => (
                  <TableRow key={index}>
                    <TableCell className="font-mono text-[13px]">{row.row_number}</TableCell>
                    <TableCell className="font-mono text-[13px] font-bold">{row.employee_code ?? "-"}</TableCell>
                    <TableCell className="text-rose-600 dark:text-rose-400 text-xs font-medium max-w-[300px]">{row.reason}</TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </div>
        </TabsContent>
      </Tabs>

      {/* Actions */}
      <div className="flex gap-4 pt-3 border-t border-slate-200 dark:border-slate-800">
        <Button className="flex-1" disabled={confirming} onClick={onClose} size="lg" variant="outline">
          {t("employees.bulk_termination.cancel_btn")}
        </Button>
        <Button
          className="flex-[1.5]"
          disabled={validCount === 0 || confirming}
          loading={confirming}
          onClick={onConfirm}
          size="lg"
          variant="primary"
        >
          {confirming
            ? t("employees.bulk_termination.processing_btn")
            : t("employees.bulk_termination.confirm_btn", { count: validCount })}
        </Button>
      </div>
    </div>
  );
}

function SummaryCard({
  label,
  value,
  className,
}: {
  label: string;
  value: number;
  className: string;
}) {
  return (
    <div className={`p-4 rounded-xl border ${className}`}>
      <span className="text-[10px] font-bold uppercase tracking-wider block mb-1 opacity-60">{label}</span>
      <span className="text-2xl font-extrabold">{value}</span>
    </div>
  );
}
