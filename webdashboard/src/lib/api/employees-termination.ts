import type { components } from "@/types/api-generated";

import { client } from "./client";
import { downloadBlob } from "./download-helper";
import { ApiError } from "./errors";
import { apiFetch } from "./helpers";

export type TerminationPreviewResponse = components["schemas"]["TerminationPreviewResponse"];
export type ConfirmTerminationRow = components["schemas"]["ConfirmTerminationRow"];

/** Upload Excel file to preview employee bulk terminations.
 *  Uses apiFetch directly because multipart/FormData is not modeled in the
 *  generated OpenAPI types. */
export async function uploadTerminationImportPreview(
  wsId: string,
  file: File
): Promise<TerminationPreviewResponse> {
  const formData = new FormData();
  formData.append("file", file);
  const res = await apiFetch(`/v1/workspaces/${wsId}/employees/import-termination/preview`, {
    method: "POST",
    body: formData,
  });
  if (!res.ok) await ApiError.rejectResponse(res);
  return res.json() as Promise<TerminationPreviewResponse>;
}

/** Confirm employee bulk terminations */
export async function confirmTerminationImport(
  wsId: string,
  rows: ConfirmTerminationRow[]
): Promise<{ success_count: number; job_id?: string }> {
  const { data } = await client.POST("/workspaces/{workspace_id}/employees/import-termination/confirm", {
    params: {
      path: { workspace_id: wsId },
    },
    body: {
      rows,
    },
  });
  return data as { success_count: number; job_id?: string };
}

/** Download bulk termination Excel template via authenticated fetch */
export async function downloadTerminationTemplate(wsId: string, lang = "en"): Promise<void> {
  const response = await apiFetch(`/v1/workspaces/${wsId}/employees/import-termination/template?lang=${lang}`);
  if (!response.ok) throw new Error("Failed to download template");
  const filename = lang === "vi"
    ? "mau_import_nghi_viec.xlsx"
    : lang === "zh"
    ? "employee_termination_template_zh.xlsx"
    : "employee_termination_template.xlsx";
  downloadBlob(await response.blob(), filename);
}
