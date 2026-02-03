/**
 * Types for WebSocket-based report streaming
 * Used by use-report-stream hook for real-time report data delivery
 */

export interface ChunkMessage {
  type: "CHUNK";
  batch_index: number;
  total_batches: number;
  data: unknown; // JSON data for this batch
}

export interface DoneMessage {
  type: "DONE";
  total_batches: number;
  total_records: number;
}

export interface ErrorMessage {
  type: "ERROR";
  message: string;
  batch_index: number | null;
}

export type StreamMessage = ChunkMessage | DoneMessage | ErrorMessage;

export type StreamStatus =
  | "idle"
  | "connecting"
  | "streaming"
  | "completed"
  | "error";

/**
 * Row types for Excel generation - matches backend excel_writer.rs structure
 */

export interface SummaryRow {
  employee_name: string;
  total_work_units: number;
  total_leave_days: number;
}

export interface DetailRow {
  date: string;
  employee_name: string;
  scheduled_start: string | null;
  scheduled_end: string | null;
  actual_in: string | null;
  actual_out: string | null;
  work_units: number;
  status: string;
}

export type AttendanceRow = SummaryRow | DetailRow;
