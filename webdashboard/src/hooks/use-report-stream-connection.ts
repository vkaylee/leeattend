import { useCallback, useEffect } from "react";
import type { StreamStatus } from "@/types/report-stream";
import { useReportStreamSocket } from "./use-report-stream-socket";

interface ConnectionProps {
  chunksRef: React.RefObject<unknown[]>;
  handleMessage: (event: MessageEvent) => void;
  jobId: string | null;
  maxRetries: number;
  onErrorRef: React.RefObject<((message: string) => void) | undefined>;
  retryCountRef: React.RefObject<number>;
  setChunks: (chunks: unknown[]) => void;
  setError: (error: string | null) => void;
  setProgress: (progress: number) => void;
  setStatus: (status: StreamStatus) => void;
  shouldReconnectRef: React.RefObject<boolean>;
  statusRef: React.RefObject<StreamStatus>;
  totalBatchesRef: React.RefObject<number>;
}

export function useReportStreamConnection({
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
}: ConnectionProps) {
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
    (retryCountRef as any).current = 0;
    (shouldReconnectRef as any).current = true;
    (chunksRef as any).current = [];
    setChunks([]);
    setProgress(0);
    setError(null);
    connect();
  }, [connect, chunksRef, setChunks, setError, setProgress, retryCountRef, shouldReconnectRef]);

  useEffect(() => {
    (chunksRef as any).current = [];
    (retryCountRef as any).current = 0;
    (shouldReconnectRef as any).current = true;
    (totalBatchesRef as any).current = 0;

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

  return { connect, disconnect, reconnect };
}
