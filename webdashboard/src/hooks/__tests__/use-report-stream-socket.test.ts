import { renderHook, act } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";

// ── Mocks for dependencies ─────────────────────────────────────────────────

const mockBuildWebSocketUrl = vi.fn();
const mockGetBackoffDelay = vi.fn();
const mockTryRefreshToken = vi.fn();
const mockClearAuthSession = vi.fn();

vi.mock("../use-report-stream-handlers", () => ({
  buildWebSocketUrl: (...args: unknown[]) => mockBuildWebSocketUrl(...args),
  getBackoffDelay: (...args: unknown[]) => mockGetBackoffDelay(...args),
  tryRefreshToken: () => mockTryRefreshToken(),
}));

vi.mock("@/lib/api/helpers", () => ({
  clearAuthSession: () => mockClearAuthSession(),
}));

vi.mock("@/lib/routes", () => ({
  ROUTES: { LOGIN: "/login" },
}));

// Need to mock maybeNull or use the real one
vi.mock("@/types/api-overrides", () => ({
  maybeNull: <T,>(value: T): T | null | undefined => value,
}));

import { useReportStreamSocket } from "../use-report-stream-socket";

// ── WebSocket constructor mock ─────────────────────────────────────────────

// Use a regular function so it's constructable with `new`
let mockSocketInstance: {
  readyState: number;
  onopen: ((ev: Event) => void) | null;
  onmessage: ((ev: MessageEvent) => void) | null;
  onclose: ((ev: CloseEvent) => void) | null;
  onerror: ((ev: Event) => void) | null;
  send: ReturnType<typeof vi.fn>;
  close: ReturnType<typeof vi.fn>;
} | null = null;

const WS_CLOSED = 3;
const WS_CONNECTING = 0;
const WS_OPEN = 1;

type SocketEventMap = Record<string, EventListenerOrEventListenerObject | null>;

function createMockWebSocketConstructor() {
  // Use a class-like function for `new` compatibility
  const WsCtor = function (this: Record<string, unknown>, _url: string) {
    const listeners: Record<string, ((e: Event | CloseEvent | MessageEvent) => void) | null> = {
      open: null,
      message: null,
      close: null,
      error: null,
    };

    const instance = {
      readyState: WS_CONNECTING,
      CLOSED: WS_CLOSED,
      CONNECTING: WS_CONNECTING,
      OPEN: WS_OPEN,
      send: vi.fn(),
      close: vi.fn(),

      get onopen() {
        return (listeners.open as ((e: Event) => void) | null) ?? null;
      },
      set onopen(fn: ((e: Event) => void) | null) {
        listeners.open = fn;
      },

      get onmessage() {
        return (listeners.message as ((e: MessageEvent) => void) | null) ?? null;
      },
      set onmessage(fn: ((e: MessageEvent) => void) | null) {
        listeners.message = fn;
      },

      get onclose() {
        return (listeners.close as ((e: CloseEvent) => void) | null) ?? null;
      },
      set onclose(fn: ((e: CloseEvent) => void) | null) {
        listeners.close = fn;
      },

      get onerror() {
        return (listeners.error as ((e: Event) => void) | null) ?? null;
      },
      set onerror(fn: ((e: Event) => void) | null) {
        listeners.error = fn;
      },
    };

    mockSocketInstance = instance;
    // Store instance on the constructor for lookup
    (WsCtor as unknown as Record<string, unknown>)._lastInstance = instance;
    return instance;
  } as unknown as { new (url: string): unknown; _lastInstance?: typeof mockSocketInstance };

  // Add static constants
  (WsCtor as unknown as Record<string, unknown>).CONNECTING = WS_CONNECTING;
  (WsCtor as unknown as Record<string, unknown>).OPEN = WS_OPEN;
  (WsCtor as unknown as Record<string, unknown>).CLOSED = WS_CLOSED;

  return WsCtor;
}

// ── Helpers ────────────────────────────────────────────────────────────────

interface SocketTestProps {
  jobId: string | null;
  maxRetries?: number;
}

function createProps(overrides: Partial<SocketTestProps> = {}) {
  const jobId = "jobId" in overrides ? overrides.jobId! : "job-123";

  return {
    jobId,
    maxRetries: overrides.maxRetries ?? 3,
    setStatus: vi.fn(),
    setError: vi.fn(),
    setChunks: vi.fn(),
    setProgress: vi.fn(),
    handleMessage: vi.fn(),
    onErrorRef: { current: undefined as ((msg: string) => void) | undefined },
    shouldReconnectRef: { current: true },
    statusRef: { current: "idle" as const },
    retryCountRef: { current: 0 },
    chunksRef: { current: [] as unknown[] },
    totalBatchesRef: { current: 0 },
  };
}

describe("useReportStreamSocket", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.useFakeTimers();
    mockSocketInstance = null;
    mockBuildWebSocketUrl.mockReturnValue("ws://localhost/api/v1/ws/reports/job-123");
    mockGetBackoffDelay.mockReturnValue(2000);
    mockTryRefreshToken.mockResolvedValue(true);
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    (globalThis as any).WebSocket = createMockWebSocketConstructor();
  });

  afterEach(() => {
    vi.useRealTimers();
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    delete (globalThis as any).WebSocket;
  });

  // ── No jobId ──────────────────────────────────────────────────────────

  it("does not attempt connection when jobId is null", () => {
    const props = createProps({ jobId: null });
    renderHook(() => useReportStreamSocket(props));

    // Should not have built a URL or created a WebSocket
    expect(mockBuildWebSocketUrl).not.toHaveBeenCalled();
  });

  // ── Connection lifecycle ──────────────────────────────────────────────

  it("builds URL and creates WebSocket on mount", () => {
    const props = createProps();
    renderHook(() => useReportStreamSocket(props));

    expect(mockBuildWebSocketUrl).toHaveBeenCalledWith("job-123");
  });

  it("transitions to streaming when socket opens", () => {
    const props = createProps();
    renderHook(() => useReportStreamSocket(props));

    expect(props.setStatus).toHaveBeenCalledWith("connecting");

    act(() => {
      mockSocketInstance!.onopen!(new Event("open"));
    });

    expect(props.setStatus).toHaveBeenCalledWith("streaming");
  });

  it("resets retry count on successful open", () => {
    const props = createProps();
    props.retryCountRef.current = 3;

    renderHook(() => useReportStreamSocket(props));

    act(() => {
      mockSocketInstance!.onopen!(new Event("open"));
    });

    expect(props.retryCountRef.current).toBe(0);
  });

  // ── Connection timeout ────────────────────────────────────────────────

  it("closes socket after 3s connection timeout", () => {
    const props = createProps();
    renderHook(() => useReportStreamSocket(props));

    // Don't trigger onopen — advance past the 3s timeout
    act(() => {
      vi.advanceTimersByTime(3000);
    });

    expect(mockSocketInstance!.close).toHaveBeenCalled();
  });

  // ── Ping keep-alive ───────────────────────────────────────────────────

  it("sends ping on 30s interval after connection", () => {
    const props = createProps();
    renderHook(() => useReportStreamSocket(props));

    act(() => {
      mockSocketInstance!.onopen!(new Event("open"));
    });

    act(() => {
      vi.advanceTimersByTime(30000);
    });

    // Should send ping while socket is in connecting→open state
    // The interval uses socket.readyState check; after onopen, state is still CONNECTING
    // unless we also update readyState
    mockSocketInstance!.readyState = WS_OPEN;

    act(() => {
      vi.advanceTimersByTime(30000);
    });

    expect(mockSocketInstance!.send).toHaveBeenCalledWith(
      JSON.stringify({ type: "ping" }),
    );
  });

  it("does not send ping when socket is not open", () => {
    const props = createProps();
    renderHook(() => useReportStreamSocket(props));

    act(() => {
      mockSocketInstance!.onopen!(new Event("open"));
    });

    // Keep readyState as CONNECTING or CLOSED (it defaults to CONNECTING)
    mockSocketInstance!.readyState = WS_CLOSED;

    act(() => {
      vi.advanceTimersByTime(30000);
    });

    expect(mockSocketInstance!.send).not.toHaveBeenCalled();
  });

  // ── Message handling ──────────────────────────────────────────────────

  it("wires onmessage to handleMessage prop", () => {
    const props = createProps();
    renderHook(() => useReportStreamSocket(props));

    expect(mockSocketInstance!.onmessage).toBe(props.handleMessage);
  });

  // ── Auth error recovery ───────────────────────────────────────────────

  it("recovers token on auth error (code 1006 at retry 0 while connecting)", () => {
    const props = createProps();
    props.statusRef.current = "connecting";
    props.retryCountRef.current = 0;
    props.shouldReconnectRef.current = true;

    renderHook(() => useReportStreamSocket(props));

    act(() => {
      mockSocketInstance!.onclose!(
        new CloseEvent("close", { code: 1006, reason: "", wasClean: false }),
      );
    });

    expect(mockTryRefreshToken).toHaveBeenCalled();
  });

  it("does not attempt token refresh on non-auth close (code 1000)", () => {
    const props = createProps();
    props.statusRef.current = "connecting";
    props.retryCountRef.current = 0;

    renderHook(() => useReportStreamSocket(props));

    act(() => {
      mockSocketInstance!.onclose!(
        new CloseEvent("close", { code: 1000, reason: "", wasClean: true }),
      );
    });

    expect(mockTryRefreshToken).not.toHaveBeenCalled();
  });

  // ── Terminal states → no reconnect ────────────────────────────────────

  it("does not reconnect when status is completed", () => {
    const props = createProps();
    props.statusRef.current = "completed";

    renderHook(() => useReportStreamSocket(props));

    act(() => {
      mockSocketInstance!.onclose!(
        new CloseEvent("close", { code: 1000, reason: "", wasClean: true }),
      );
    });

    expect(mockGetBackoffDelay).not.toHaveBeenCalled();
  });

  it("does not reconnect when status is error", () => {
    const props = createProps();
    props.statusRef.current = "error";

    renderHook(() => useReportStreamSocket(props));

    act(() => {
      mockSocketInstance!.onclose!(
        new CloseEvent("close", { code: 1000, reason: "", wasClean: true }),
      );
    });

    expect(mockGetBackoffDelay).not.toHaveBeenCalled();
  });

  // ── Reconnect with backoff ────────────────────────────────────────────

  it("schedules reconnect on unexpected close", () => {
    const props = createProps();
    props.statusRef.current = "streaming";
    props.shouldReconnectRef.current = true;

    renderHook(() => useReportStreamSocket(props));

    // Set retryCount AFTER render since the effect resets it to 0
    props.retryCountRef.current = 1;

    act(() => {
      mockSocketInstance!.onclose!(
        new CloseEvent("close", { code: 1006, reason: "", wasClean: false }),
      );
    });

    expect(mockGetBackoffDelay).toHaveBeenCalledWith(1);
  });

  // ── Max retries exceeded ──────────────────────────────────────────────

  it("sets error state when retries >= maxRetries", () => {
    const props = createProps({ maxRetries: 2 });
    props.statusRef.current = "streaming";
    props.shouldReconnectRef.current = true;

    renderHook(() => useReportStreamSocket(props));

    // Set retryCount AFTER render since the effect resets it to 0
    props.retryCountRef.current = 3;

    act(() => {
      mockSocketInstance!.onclose!(
        new CloseEvent("close", { code: 1000, reason: "", wasClean: true }),
      );
    });

    expect(props.setStatus).toHaveBeenCalledWith("error");
    expect(props.setError).toHaveBeenCalledWith("Connection lost. Max retries exceeded.");
  });

  // ── Manual reconnect ─────────────────────────────────────────────────

  it("reconnect resets state and reconnects", () => {
    const props = createProps();
    props.retryCountRef.current = 5;

    const { result } = renderHook(() => useReportStreamSocket(props));
    const callCountBeforeReconnect = mockBuildWebSocketUrl.mock.calls.length;

    act(() => {
      result.current.reconnect();
    });

    expect(props.retryCountRef.current).toBe(0);
    expect(props.setChunks).toHaveBeenCalledWith([]);
    expect(props.setProgress).toHaveBeenCalledWith(0);
    expect(props.setError).toHaveBeenCalledWith(null);
    expect(mockBuildWebSocketUrl.mock.calls.length).toBeGreaterThan(callCountBeforeReconnect);
  });

  // ── Disconnect ────────────────────────────────────────────────────────

  it("disconnect stops reconnect and closes socket", () => {
    const props = createProps();
    const { result } = renderHook(() => useReportStreamSocket(props));

    act(() => {
      result.current.disconnect();
    });

    expect(props.shouldReconnectRef.current).toBe(false);
    expect(mockSocketInstance!.close).toHaveBeenCalled();
  });

  // ── Cleanup on unmount ────────────────────────────────────────────────

  it("cleans up on unmount", () => {
    const props = createProps();
    const { unmount } = renderHook(() => useReportStreamSocket(props));

    unmount();

    expect(props.shouldReconnectRef.current).toBe(false);
    expect(mockSocketInstance!.close).toHaveBeenCalled();
  });

  // ── Error handling ────────────────────────────────────────────────────

  it("handles WebSocket constructor throwing", () => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    (globalThis as any).WebSocket = vi.fn(() => {
      throw new Error("Connection failed");
    });

    const props = createProps();
    renderHook(() => useReportStreamSocket(props));

    expect(props.setStatus).toHaveBeenCalledWith("error");
    expect(props.setError).toHaveBeenCalledWith("Failed to establish connection");
  });
});
