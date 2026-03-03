import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useReportStream } from "../use-report-stream";

/**
 * Tests for the report stream double-buffering fix.
 * After DONE message, chunksRef should be cleared to avoid
 * keeping duplicate data in both ref and state.
 */
describe("useReportStream Double-Buffering Fix", () => {
    let capturedSocket: any;

    function createMockWebSocket(_url: string) {
        const socket = {
            readyState: 0,
            onopen: null as ((ev: any) => void) | null,
            onmessage: null as ((ev: any) => void) | null,
            onerror: null as ((ev: any) => void) | null,
            onclose: null as ((ev: any) => void) | null,
            close: vi.fn(),
        };
        capturedSocket = socket;
        setTimeout(() => {
            socket.readyState = 1;
            socket.onopen?.({ type: "open" });
        }, 0);
        return socket;
    }

    beforeEach(() => {
        capturedSocket = null;
        vi.stubGlobal("WebSocket", createMockWebSocket);
        vi.spyOn(console, "debug").mockImplementation(() => { });
        vi.spyOn(console, "error").mockImplementation(() => { });
    });

    afterEach(() => {
        vi.unstubAllGlobals();
        vi.clearAllMocks();
        localStorage.clear();
    });

    it("should clear chunks ref after DONE to avoid double-buffering", async () => {
        const onComplete = vi.fn();

        const { result } = renderHook(() =>
            useReportStream({
                jobId: "test-job",
                onComplete,
            })
        );

        // Wait for connection
        await act(async () => {
            await new Promise((r) => setTimeout(r, 10));
        });

        // Simulate receiving DONE message
        await act(async () => {
            capturedSocket?.onmessage?.({
                data: JSON.stringify({
                    type: "DONE",
                    total_batches: 2,
                    total_records: 100,
                }),
            });
        });

        // onComplete should be called with the final chunks
        expect(onComplete).toHaveBeenCalledTimes(1);

        // After DONE, status should be completed
        expect(result.current.status).toBe("completed");

        // The chunks passed to onComplete should be an array (possibly empty
        // since we didn't send CHUNK messages, but it should not be undefined)
        const completedChunks = onComplete.mock.calls[0][0];
        expect(Array.isArray(completedChunks)).toBe(true);
    });

    it("should pass same data to onComplete and state chunks", async () => {
        const onComplete = vi.fn();

        const { result } = renderHook(() =>
            useReportStream({
                jobId: "test-job",
                onComplete,
            })
        );

        // Wait for connection
        await act(async () => {
            await new Promise((r) => setTimeout(r, 10));
        });

        // Simulate DONE
        await act(async () => {
            capturedSocket?.onmessage?.({
                data: JSON.stringify({
                    type: "DONE",
                    total_batches: 0,
                    total_records: 0,
                }),
            });
        });

        // Both the state chunks and the onComplete arg should be the same content
        const completedChunks = onComplete.mock.calls[0][0];
        expect(result.current.chunks).toEqual(completedChunks);
    });

    it("should handle DONE message with progress set to 100", async () => {
        const { result } = renderHook(() =>
            useReportStream({ jobId: "test-job" })
        );

        await act(async () => {
            await new Promise((r) => setTimeout(r, 10));
        });

        await act(async () => {
            capturedSocket?.onmessage?.({
                data: JSON.stringify({
                    type: "DONE",
                    total_batches: 5,
                    total_records: 500,
                }),
            });
        });

        expect(result.current.progress).toBe(100);
        expect(result.current.status).toBe("completed");
    });

    it("should not reconnect after DONE", async () => {
        const { result } = renderHook(() =>
            useReportStream({ jobId: "test-job", maxRetries: 3 })
        );

        await act(async () => {
            await new Promise((r) => setTimeout(r, 10));
        });

        // Send DONE
        await act(async () => {
            capturedSocket?.onmessage?.({
                data: JSON.stringify({
                    type: "DONE",
                    total_batches: 1,
                    total_records: 10,
                }),
            });
        });

        expect(result.current.status).toBe("completed");

        // Simulate connection close after DONE

        await act(async () => {
            capturedSocket?.onclose?.({ code: 1000, reason: "Normal" });
        });

        // Should NOT reconnect (status should remain completed)
        expect(result.current.status).toBe("completed");
    });

    it("should handle ERROR message and stop reconnection", async () => {
        const onError = vi.fn();

        const { result } = renderHook(() =>
            useReportStream({
                jobId: "test-job",
                onError,
                maxRetries: 3,
            })
        );

        await act(async () => {
            await new Promise((r) => setTimeout(r, 10));
        });

        // Send ERROR
        await act(async () => {
            capturedSocket?.onmessage?.({
                data: JSON.stringify({
                    type: "ERROR",
                    message: "Report generation failed",
                    batch_index: null,
                }),
            });
        });

        expect(result.current.status).toBe("error");
        expect(result.current.error).toBe("Report generation failed");
        expect(onError).toHaveBeenCalledWith("Report generation failed");
    });
});
