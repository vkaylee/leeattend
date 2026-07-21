import { ApiError } from "./errors";
import { apiFetch, TIMEOUT } from "./helpers";

export interface HttpApiConfig {
  url: string;
  auth_header?: string;
  auth_value?: string;
  success_field?: string;
  success_value?: unknown;
  body_template: string;
}

export interface SmtpConfigData {
  base_url?: string;
  from?: string;
  host?: string;
  http_api?: HttpApiConfig | null;
  password?: string;
  port?: number;
  username?: string;
}

export interface SmtpTestResponse {
  message: string;
  success: boolean;
}

export async function fetchSmtpConfig(): Promise<SmtpConfigData> {
  const res = await apiFetch("/v1/system/smtp/config", {
    timeout: TIMEOUT.DEFAULT,
  });
  if (!res.ok) await ApiError.rejectResponse(res);
  return res.json() as Promise<SmtpConfigData>;
}

export async function saveSmtpConfig(config: SmtpConfigData): Promise<void> {
  const res = await apiFetch("/v1/system/smtp/config", {
    method: "PUT",
    body: JSON.stringify(config),
    timeout: TIMEOUT.DEFAULT,
  });
  if (!res.ok) await ApiError.rejectResponse(res);
}

export async function testSmtp(toEmail: string): Promise<SmtpTestResponse> {
  const res = await apiFetch("/v1/system/smtp/test", {
    method: "POST",
    body: JSON.stringify({ to_email: toEmail }),
    timeout: TIMEOUT.REPORT_OPERATION,
  });
  return res.json() as Promise<SmtpTestResponse>;
}
