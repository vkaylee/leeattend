/** CSV utility for bulk upload — download template & error report */

export function downloadCsvTemplate(): void {
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
}

interface CsvError {
  line_number: number;
  employment_id: string;
  error_message: string;
}

export function downloadCsvErrorReport(errors: CsvError[]): void {
  if (errors.length === 0) return;

  const headers = "line_number,employment_id,error_message";
  const rows = errors
    .map(
      (e) =>
        `${e.line_number.toString()},${e.employment_id},"${e.error_message}"`,
    )
    .join("\n");

  const csvContent = `${headers}\n${rows}`;
  const blob = new Blob([csvContent], { type: "text/csv" });
  const url = window.URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = `ot_upload_errors_${Date.now().toString()}.csv`;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  window.URL.revokeObjectURL(url);
}
