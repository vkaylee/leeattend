"use client";

import { useEffect, useRef, useState } from "react";

import type {
  StreamStatus,
} from "@/types/report-stream";

import { useReportStreamHandlers } from "./use-report-stream-handlers";
import { useReportStreamSocket } from "./use-report-stream-socket";

export interface UseReportStreamOptions {
  jobId: string | null;
  maxRetries?: number;
  onComplete?: (data: unknown[]) => void;
  onError?: (message: string) => void;
}

export interface UseReportStreamResult {
  chunks: unknown[];
  error: string | null;
  progress: number; // 0-100
  reconnect: () => void;
  status: StreamStatus;
}

/**
 * Internal state capsule for the report stream.
 * Keeps all mutable state (values + refs) in one place so the
 * orchestrating hook stays readable.
 */
function useReportStreamState() {
  const [status, setStatus] = useState<StreamStatus>("idle");
  const [progress, setProgress] = useState(0);
  const [error, setError] = useState<string | null>(null);
  const [chunks, setChunks] = useState<unknown[]>([]);

  const chunksRef = useRef<unknown[]>([]);
  const retryCountRef = useRef(0);
  const shouldReconnectRef = useRef(true);
  const totalBatchesRef = useRef(0);
  const statusRef = useRef<StreamStatus>("idle");

  return {
    status, setStatus,
    progress, setProgress,
    error, setError,
    chunks, setChunks,
    chunksRef,
    retryCountRef,
    shouldReconnectRef,
    totalBatchesRef,
    statusRef,
  };
}

/**
 * Top-level orchestrating hook for WebSocket-based report streaming.
 *
 * Responsibilities:
 * - Owns all stream state (status, progress, chunks, error).
 * - Wires together two single-responsibility sub-hooks:
 *   - {@link useReportStreamHandlers} — parses WS messages into state transitions.
 *   - {@link useReportStreamSocket} — manages the WS lifecycle (connect, reconnect, ping, cleanup).
 * - Resets state on jobId changes.
 */
export function useReportStream({
  jobId,
  onComplete,
  onError,
  maxRetries = 3,
}: UseReportStreamOptions): UseReportStreamResult {
  const {
    status, setStatus,
    progress, setProgress,
    error, setError,
    chunks, setChunks,
    chunksRef,
    retryCountRef,
    shouldReconnectRef,
    totalBatchesRef,
    statusRef,
  } = useReportStreamState();

  const onCompleteRef = useRef(onComplete);
  const onErrorRef = useRef(onError);

  useEffect(() => {
    onCompleteRef.current = onComplete;
    onErrorRef.current = onError;
    statusRef.current = status;
  });

  const { handleMessage } = useReportStreamHandlers({
    setStatus,
    setProgress,
    setError,
    setChunks,
    chunksRef,
    onCompleteRef,
    onErrorRef,
    shouldReconnectRef,
    totalBatchesRef,
  });

  const { reconnect } = useReportStreamSocket({
    jobId,
    maxRetries,
    setStatus,
    setError,
    setChunks,
    setProgress,
    handleMessage,
    onErrorRef,
    shouldReconnectRef,
    statusRef,
    retryCountRef,
    chunksRef,
    totalBatchesRef,
  });

  const [previousJobId, setPreviousJobId] = useState(jobId);
  if (jobId !== previousJobId) {
    setPreviousJobId(jobId);
    if (!jobId) {
      setStatus("idle");
      setProgress(0);
      setError(null);
      setChunks([]);
    } else {
      setChunks([]);
    }
  }

  return {
    status,
    progress,
    chunks,
    error,
    reconnect,
  };
}
