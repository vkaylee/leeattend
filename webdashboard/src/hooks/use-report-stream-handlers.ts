import { useCallback } from "react";

import type { StreamMessage, StreamStatus } from "@/types/report-stream";

import { forceRefreshToken } from "@/lib/api/api-fetch-helpers";
import { getAccessToken } from "@/lib/api/helpers";
import { logError } from "@/lib/logger";

// ── Utility functions (merged from use-report-stream-utils) ──────────────

/**
 * Build WebSocket URL for report streaming.
 * Uses wss: for HTTPS, ws: for HTTP.
 * Includes JWT token as query param (required by ws_auth_middleware).
 */
export function buildWebSocketUrl(jobId: string): string {
  const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
  const host = window.location.host;
  const token = getAccessToken();
  const params = token ? `?token=${encodeURIComponent(token)}` : "";
  return `${protocol}//${host}/api/v1/ws/reports/${jobId}${params}`;
}

/**
 * Calculate exponential backoff delay.
 * Returns delays: 1s, 2s, 4s for retries 0, 1, 2.
 */
export function getBackoffDelay(retryCount: number): number {
  return Math.pow(2, retryCount) * 1000;
}

/**
 * Attempt to refresh JWT token (same logic as apiFetch 401 handler).
 * Returns true if refresh succeeded, false otherwise.
 */
export async function tryRefreshToken(): Promise<boolean> {
  return await forceRefreshToken();
}

// ── Message handler hook ─────────────────────────────────────────────────

/**
 * Single-responsibility hook: parses incoming WebSocket messages and
 * translates them into stream state transitions (progress, completion, error).
 * Does NOT manage the WebSocket lifecycle — that belongs to useReportStreamSocket.
 */
export function useReportStreamHandlers({
  setStatus,
  setProgress,
  setError,
  setChunks,
  chunksRef,
  onCompleteRef,
  onErrorRef,
  shouldReconnectRef,
  totalBatchesRef,
}: {
  chunksRef: React.RefObject<unknown[]>;
  onCompleteRef: React.RefObject<((data: unknown[]) => void) | undefined>;
  onErrorRef: React.RefObject<((message: string) => void) | undefined>;
  setChunks: (chunks: unknown[]) => void;
  setError: (error: string | null) => void;
  setProgress: (progress: number) => void;
  setStatus: (status: StreamStatus) => void;
  shouldReconnectRef: React.RefObject<boolean>;
  totalBatchesRef: React.RefObject<number>;
}) {
  const handleMessage = useCallback((event: MessageEvent) => {
    try {
      const message = JSON.parse(event.data as string) as StreamMessage;
      switch (message.type) {
        case "CHUNK": {
          totalBatchesRef.current = message.total_batches;
          const newProgress = ((message.batch_index + 1) / message.total_batches) * 100;
          setProgress(Math.min(newProgress, 100));
          setStatus("streaming");
          break;
        }
        case "DONE": {
          setStatus("completed");
          setProgress(100);
          shouldReconnectRef.current = false;
          const finalChunks = [...chunksRef.current];
          chunksRef.current = [];
          setChunks(finalChunks);
          onCompleteRef.current?.(finalChunks);
          break;
        }
        case "ERROR": {
          setStatus("error");
          setError(message.message);
          shouldReconnectRef.current = false;
          onErrorRef.current?.(message.message);
          break;
        }
      }
    } catch (error_) {
      logError(error_, { component: "ReportStream" });
    }
  }, [setChunks, setError, setProgress, setStatus, chunksRef, onErrorRef, onCompleteRef, shouldReconnectRef, totalBatchesRef]);

  return { handleMessage };
}
