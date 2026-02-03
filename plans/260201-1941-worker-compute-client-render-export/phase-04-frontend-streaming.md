# Phase 4: Frontend Streaming Hook

**Priority**: P0
**Status**: Complete
**Estimated**: Medium complexity

---

## Context Links
- [Plan Overview](./plan.md)
- [Phase 3: WebSocket Endpoint](./phase-03-websocket-endpoint.md)

---

## Overview

Create React hook `useReportStream` that connects to WebSocket endpoint and accumulates report data chunks.

---

## Key Insights

- WebSocket native API sufficient (no socket.io needed)
- Need reconnection logic for network hiccups
- Accumulate rows in state, trigger Excel generation on DONE
- Progress tracking from CHUNK messages

---

## Requirements

### Functional
- Connect to `/ws/reports/{job_id}` on mount
- Parse incoming CHUNK messages, accumulate data
- Track progress percentage from messages
- Trigger callback on DONE with full dataset
- Handle ERROR messages with user notification
- Auto-reconnect on connection drop (max 3 attempts)

### Non-Functional
- Memory efficient (don't duplicate large arrays)
- TypeScript types for all messages
- Cleanup WebSocket on unmount

---

## Related Code Files

### Create
- `webdashboard/src/hooks/use-report-stream.ts` - Main hook
- `webdashboard/src/types/report-stream.ts` - TypeScript types

### Modify
- `webdashboard/src/components/features/reports/ReportExportDialog.tsx` - Use hook

---

## Implementation Steps

### Step 1: Define Types

```typescript
// webdashboard/src/types/report-stream.ts

export interface ChunkMessage {
  type: 'CHUNK';
  data: AttendanceRow[];
  batch_index: number;
  total_batches: number;
  progress: number; // 0-100
}

export interface DoneMessage {
  type: 'DONE';
  total_rows: number;
}

export interface ErrorMessage {
  type: 'ERROR';
  message: string;
  batch_index?: number;
}

export type StreamMessage = ChunkMessage | DoneMessage | ErrorMessage;

export interface AttendanceRow {
  employee_id: string;
  employee_name: string;
  date: string;
  check_in?: string;
  check_out?: string;
  status: string;
  // ... other fields based on report type
}

export type StreamStatus = 'connecting' | 'streaming' | 'completed' | 'error' | 'idle';
```

### Step 2: Create Hook

```typescript
// webdashboard/src/hooks/use-report-stream.ts

import { useCallback, useEffect, useRef, useState } from 'react';
import type { AttendanceRow, StreamMessage, StreamStatus } from '@/types/report-stream';

interface UseReportStreamOptions {
  jobId: string | null;
  token: string;
  onComplete?: (rows: AttendanceRow[]) => void;
  onError?: (message: string) => void;
  maxRetries?: number;
}

interface UseReportStreamResult {
  status: StreamStatus;
  progress: number;
  rows: AttendanceRow[];
  error: string | null;
  reconnect: () => void;
}

export function useReportStream({
  jobId,
  token,
  onComplete,
  onError,
  maxRetries = 3,
}: UseReportStreamOptions): UseReportStreamResult {
  const [status, setStatus] = useState<StreamStatus>('idle');
  const [progress, setProgress] = useState(0);
  const [rows, setRows] = useState<AttendanceRow[]>([]);
  const [error, setError] = useState<string | null>(null);

  const wsRef = useRef<WebSocket | null>(null);
  const retriesRef = useRef(0);
  const rowsRef = useRef<AttendanceRow[]>([]);

  const connect = useCallback(() => {
    if (!jobId) return;

    const wsUrl = `${window.location.protocol === 'https:' ? 'wss:' : 'ws:'}//${window.location.host}/ws/reports/${jobId}?token=${token}`;

    setStatus('connecting');
    setError(null);

    const ws = new WebSocket(wsUrl);
    wsRef.current = ws;

    ws.onopen = () => {
      setStatus('streaming');
      retriesRef.current = 0;
    };

    ws.onmessage = (event) => {
      try {
        const msg: StreamMessage = JSON.parse(event.data);

        switch (msg.type) {
          case 'CHUNK':
            // Accumulate rows efficiently
            rowsRef.current = [...rowsRef.current, ...msg.data];
            setRows(rowsRef.current);
            setProgress(msg.progress);
            break;

          case 'DONE':
            setStatus('completed');
            setProgress(100);
            onComplete?.(rowsRef.current);
            ws.close();
            break;

          case 'ERROR':
            setStatus('error');
            setError(msg.message);
            onError?.(msg.message);
            ws.close();
            break;
        }
      } catch (e) {
        console.error('Failed to parse WebSocket message:', e);
      }
    };

    ws.onerror = () => {
      setError('Connection error');
    };

    ws.onclose = (event) => {
      if (status === 'streaming' && retriesRef.current < maxRetries) {
        // Unexpected close, attempt reconnect
        retriesRef.current++;
        setTimeout(connect, 1000 * retriesRef.current);
      } else if (status !== 'completed') {
        setStatus('error');
        setError('Connection closed unexpectedly');
      }
    };
  }, [jobId, token, onComplete, onError, maxRetries, status]);

  const reconnect = useCallback(() => {
    wsRef.current?.close();
    rowsRef.current = [];
    setRows([]);
    setProgress(0);
    retriesRef.current = 0;
    connect();
  }, [connect]);

  // Connect when jobId is set
  useEffect(() => {
    if (jobId) {
      rowsRef.current = [];
      setRows([]);
      connect();
    }

    return () => {
      wsRef.current?.close();
    };
  }, [jobId, connect]);

  return { status, progress, rows, error, reconnect };
}
```

### Step 3: Create Fallback Polling Hook

```typescript
// webdashboard/src/hooks/use-report-polling.ts

import { useQuery } from '@tanstack/react-query';
import { useState, useEffect } from 'react';

interface UseReportPollingOptions {
  jobId: string | null;
  enabled?: boolean;
  onComplete?: (downloadUrl: string) => void;
}

export function useReportPolling({
  jobId,
  enabled = true,
  onComplete,
}: UseReportPollingOptions) {
  const [isPolling, setIsPolling] = useState(false);

  const { data, error } = useQuery({
    queryKey: ['report-status', jobId],
    queryFn: async () => {
      const res = await fetch(`/api/workspaces/${workspaceId}/reports/jobs/${jobId}`);
      return res.json();
    },
    enabled: enabled && !!jobId && isPolling,
    refetchInterval: 2000,
  });

  useEffect(() => {
    if (data?.status === 'completed' && data?.download_url) {
      setIsPolling(false);
      onComplete?.(data.download_url);
    }
  }, [data, onComplete]);

  const startPolling = () => setIsPolling(true);
  const stopPolling = () => setIsPolling(false);

  return {
    status: data?.status,
    progress: data?.progress ?? 0,
    downloadUrl: data?.download_url,
    error: error?.message,
    startPolling,
    stopPolling,
  };
}
```

### Step 4: Create Combined Hook with Fallback

```typescript
// webdashboard/src/hooks/use-report-export.ts

import { useState, useCallback } from 'react';
import { useReportStream } from './use-report-stream';
import { useReportPolling } from './use-report-polling';
import { generateExcel } from '@/lib/excel-builder';

interface UseReportExportOptions {
  workspaceId: string;
  onSuccess?: () => void;
  onError?: (message: string) => void;
}

export function useReportExport({
  workspaceId,
  onSuccess,
  onError,
}: UseReportExportOptions) {
  const [jobId, setJobId] = useState<string | null>(null);
  const [mode, setMode] = useState<'stream' | 'poll'>('stream');
  const [token, setToken] = useState('');

  // Streaming mode
  const stream = useReportStream({
    jobId: mode === 'stream' ? jobId : null,
    token,
    onComplete: async (rows) => {
      await generateExcel(rows, 'attendance-report');
      onSuccess?.();
    },
    onError: (msg) => {
      // Fallback to polling if WebSocket fails
      console.warn('WebSocket failed, falling back to polling:', msg);
      setMode('poll');
    },
  });

  // Polling mode (fallback)
  const polling = useReportPolling({
    jobId: mode === 'poll' ? jobId : null,
    onComplete: (url) => {
      window.open(url, '_blank');
      onSuccess?.();
    },
  });

  const startExport = useCallback(async (params: ExportParams) => {
    // Request export job
    const res = await fetch(`/api/workspaces/${workspaceId}/reports/export`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(params),
    });

    const { job_id, token } = await res.json();
    setJobId(job_id);
    setToken(token);
    setMode('stream'); // Try WebSocket first
  }, [workspaceId]);

  return {
    startExport,
    status: mode === 'stream' ? stream.status : polling.status,
    progress: mode === 'stream' ? stream.progress : polling.progress,
    error: mode === 'stream' ? stream.error : polling.error,
    mode,
  };
}
```

---

## Todo List

- [ ] Create TypeScript types for stream messages
- [ ] Implement useReportStream hook
- [ ] Add reconnection logic with exponential backoff
- [ ] Create useReportPolling as fallback
- [ ] Create combined useReportExport hook
- [ ] Update ReportExportDialog to use new hook
- [ ] Add progress bar UI component
- [ ] Test WebSocket connection in browser DevTools

---

## Success Criteria

- [ ] Hook connects to WebSocket on jobId set
- [ ] Progress updates in real-time during streaming
- [ ] Rows accumulate without memory leaks
- [ ] onComplete fires with full dataset
- [ ] Fallback to polling works when WS fails
- [ ] Connection cleaned up on unmount

---

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| Memory bloat | High | Use refs, limit max rows |
| WS not supported | Low | Fallback to polling mode |
| Reconnect storm | Medium | Exponential backoff |

---

## Security Considerations

- Token passed via query param (HTTPS required in production)
- No sensitive data logged to console
- WebSocket URL validated to same origin
