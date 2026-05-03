"use client";

import { useEffect, useRef, useState, useCallback } from "react";

import type {
  StreamMessage,
  StreamStatus,
} from "@/types/report-stream";

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

function useReportStreamMessageHandler({
  setStatus,
  setProgress,
  setError,
  setChunks,
  chunksRef,
  onErrorRef,
  onCompleteRef,
  shouldReconnectRef,
  totalBatchesRef
}: {
  setStatus: React.Dispatch<React.SetStateAction<StreamStatus>>;
  setProgress: React.Dispatch<React.SetStateAction<number>>;
  setError: React.Dispatch<React.SetStateAction<string | null>>;
  setChunks: React.Dispatch<React.SetStateAction<unknown[]>>;
  chunksRef: React.RefObject<unknown[]>;
  onErrorRef: React.RefObject<((message: string) => void) | undefined>;
  onCompleteRef: React.RefObject<((data: unknown[]) => void) | undefined>;
  shouldReconnectRef: React.RefObject<boolean>;
  totalBatchesRef: React.RefObject<number>;
}) {
  return useCallback((event: MessageEvent) => {
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
      console.error("[ReportStream] Failed to parse message:", error_);
    }
  }, [setChunks, setError, setProgress, setStatus, chunksRef, onErrorRef, onCompleteRef, shouldReconnectRef, totalBatchesRef]);
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

  const handleMessage = useReportStreamMessageHandler({
    setStatus, setProgress, setError, setChunks,
    chunksRef, onErrorRef, onCompleteRef,
    shouldReconnectRef, totalBatchesRef
  });

  const { connect, disconnect } = useReportStreamSocket({
    jobId,
    maxRetries,
    setStatus,
    setError,
    handleMessage,
    onErrorRef,
    shouldReconnectRef,
    statusRef,
    retryCountRef,
  });

  const reconnect = useCallback(() => {
    retryCountRef.current = 0;
    shouldReconnectRef.current = true;
    chunksRef.current = [];
    setChunks([]);
    setProgress(0);
    setError(null);
    connect();
  }, [connect, chunksRef, setChunks, setError, setProgress, retryCountRef, shouldReconnectRef]);

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

  useEffect(() => {
    chunksRef.current = [];
    retryCountRef.current = 0;
    shouldReconnectRef.current = true;
    totalBatchesRef.current = 0;

    if (!jobId) return;

    void Promise.resolve().then(() => {
      if (shouldReconnectRef.current) {
        connect();
      }
    });

    return () => {
      disconnect();
    };
  }, [jobId, connect, disconnect, chunksRef, retryCountRef, shouldReconnectRef, totalBatchesRef]);

  return {
    status,
    progress,
    chunks,
    error,
    reconnect,
  };
}
