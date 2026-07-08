import { type ClassValue, clsx } from "clsx";
import { format, parse } from "date-fns";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export function isAgentOnline(lastSeenAt: string | null | undefined): boolean {
  if (!lastSeenAt) return false;
  const lastSeen = new Date(lastSeenAt).getTime();
  const now = new Date().getTime();
  // Threshold is 90 seconds
  return now - lastSeen < 90000;
}

export function formatTime(timeString: string | null | undefined, timeFormat: string | undefined): string {
  if (!timeString) return "--:--";
  // timeStr is expected to be HH:mm or HH:mm:ss
  try {
    const referenceDate = new Date();
    const date = parse(timeString.substring(0, 8), timeString.includes(":") && timeString.split(":").length >= 3 ? "HH:mm:ss" : "HH:mm", referenceDate);
    const pattern = timeFormat === "12h" ? "hh:mm a" : "HH:mm";
    return format(date, pattern);
  } catch {
    return timeString.substring(0, 5);
  }
}

export function formatDateTime(dateTimeString: string | null | undefined, timeFormat: string | undefined): string {
  if (!dateTimeString) return "--:--";
  const date = new Date(dateTimeString);
  const pattern = timeFormat === "12h" ? "dd/MM/yyyy hh:mm a" : "dd/MM/yyyy HH:mm";
  return format(date, pattern);
}

/**
 * Returns attributes for debugging and testing.
 * Stripped in production via build-time rollup configuration.
 */
 
export function debugProps(data: Record<string, unknown>) {
  // We check PROD here for runtime safety, though Rollup will strip the result anyway
  if (import.meta.env.PROD) return {};
  
  return {
    'data-debug-context': JSON.stringify(data)
  };
}

/**
 * Mask an email address for PII protection.
 * "john.doe@example.com" → "j***e@example.com"
 * "ab@example.com" → "a***b@example.com"
 */
export function maskEmail(email: string | null | undefined): string {
  if (!email) return "***";
  const [local, domain] = email.split("@");
  if (!domain) return "***";
  if (local.length === 1) {
    return `${local.charAt(0)}***@${domain}`;
  }
  // 2+ chars: first + asterisks (capped at 4) + last
  const middle = Math.max(3, Math.min(local.length - 2, 4));
  return `${local.charAt(0)}${"*".repeat(middle)}${local.charAt(local.length - 1)}@${domain}`;
}

/**
 * Mask a phone number for PII protection.
 * "+84901234567" → "+84***4567"
 * "0901234567" → "***4567"
 */
export function maskPhone(phone: string | null | undefined): string {
  if (!phone) return "***";
  const cleaned = phone.replace(/[\s\-()]/g, "");
  if (cleaned.length <= 4) return "***";
  const visible = cleaned.slice(-4);
  return `***${visible}`;
}

export function formatBytes(bytes: number, decimals = 2) {
  if (bytes === 0) return "0 Bytes";

  const k = 1024;
  const dm = decimals < 0 ? 0 : decimals;
  const sizes = ["Bytes", "KB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"];

  const i = Math.floor(Math.log(bytes) / Math.log(k));

  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(dm))} ${sizes[i]}`;
}
