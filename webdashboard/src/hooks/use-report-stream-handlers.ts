import { useCallback } from "react";

import type { StreamMessage, StreamStatus } from "@/types/report-stream";

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
      console.error("[ReportStream] Failed to parse message:", error_);
    }
  }, [setChunks, setError, setProgress, setStatus, chunksRef, onErrorRef, onCompleteRef, shouldReconnectRef, totalBatchesRef]);

  return { handleMessage };
}
