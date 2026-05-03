"use client";

import { useEffect, useRef, useState } from "react";

import type {
  StreamStatus,
} from "@/types/report-stream";

import { useReportStreamConnection } from "./use-report-stream-connection";
import { useReportStreamHandlers } from "./use-report-stream-handlers";

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
 * Hook for WebSocket-based report streaming
 * Connects to the report WebSocket endpoint and accumulates chunks
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

  const { reconnect } = useReportStreamConnection({
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
