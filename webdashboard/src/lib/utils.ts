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

export function formatTime(timeStr: string | null | undefined, timeFormat: string | undefined): string {
  if (!timeStr) return "--:--";
  // timeStr is expected to be HH:mm or HH:mm:ss
  try {
    const referenceDate = new Date();
    const date = parse(timeStr.substring(0, 8), timeStr.includes(":") && timeStr.split(":").length >= 3 ? "HH:mm:ss" : "HH:mm", referenceDate);
    const pattern = timeFormat === "12h" ? "hh:mm a" : "HH:mm";
    return format(date, pattern);
  } catch (__e) {
    return timeStr.substring(0, 5);
  }
}

export function formatDateTime(dateTimeStr: string | null | undefined, timeFormat: string | undefined): string {
  if (!dateTimeStr) return "--:--";
  const date = new Date(dateTimeStr);
  const pattern = timeFormat === "12h" ? "dd/MM/yyyy hh:mm a" : "dd/MM/yyyy HH:mm";
  return format(date, pattern);
}

/**
 * Returns attributes for debugging and testing.
 * Stripped in production via build-time rollup configuration.
 */
export function debugProps(data: Record<string, any>) {
  // We check PROD here for runtime safety, though Rollup will strip the result anyway
  if (import.meta.env.PROD) return {};
  
  return {
    'data-debug-context': JSON.stringify(data)
  };
}
