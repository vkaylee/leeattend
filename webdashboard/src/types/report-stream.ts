/**
 * Types for WebSocket-based report streaming
 * Used by use-report-stream hook for real-time report data delivery
 */

export interface ChunkMessage {
  batch_index: number;
  total_batches: number;
  type: "CHUNK";
}

export interface DoneMessage {
  total_batches: number;
  total_records: number;
  type: "DONE";
}

export interface ErrorMessage {
  batch_index: number | null;
  message: string;
  type: "ERROR";
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
  total_leave_days: number;
  total_ot_units: number;
  total_std_units: number;
}

export interface DetailRow {
  actual_in: string | null;
  actual_out: string | null;
  date: string;
  employee_name: string;
  ot_claim_status?: string | null;
  ot_work_units: number;
  scheduled_end: string | null;
  scheduled_start: string | null;
  status: string;
  std_work_units: number;
}

export type AttendanceRow = SummaryRow | DetailRow;
