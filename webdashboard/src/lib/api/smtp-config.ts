import { apiFetch, TIMEOUT } from "./helpers";

export interface SmtpConfigData {
  host?: string;
  port?: number;
  username?: string;
  password?: string;
  from?: string;
  base_url?: string;
}

export interface SmtpTestResponse {
  success: boolean;
  message: string;
}

export async function fetchSmtpConfig(): Promise<SmtpConfigData> {
  const res = await apiFetch("/v1/system/smtp/config", {
    timeout: TIMEOUT.DEFAULT,
  });
  if (!res.ok) throw new Error("Failed to load SMTP config");
  return res.json() as Promise<SmtpConfigData>;
}

export async function saveSmtpConfig(config: SmtpConfigData): Promise<void> {
  const res = await apiFetch("/v1/system/smtp/config", {
    method: "PUT",
    body: JSON.stringify(config),
    timeout: TIMEOUT.DEFAULT,
  });
  if (!res.ok) throw new Error("Failed to save SMTP config");
}

export async function testSmtp(toEmail: string): Promise<SmtpTestResponse> {
  const res = await apiFetch("/v1/system/smtp/test", {
    method: "POST",
    body: JSON.stringify({ to_email: toEmail }),
    timeout: TIMEOUT.REPORT_OPERATION,
  });
  return res.json() as Promise<SmtpTestResponse>;
}
