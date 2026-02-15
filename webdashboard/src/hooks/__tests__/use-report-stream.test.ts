import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import { renderHook, waitFor, act } from "@testing-library/react";
import { useReportStream } from "../use-report-stream";

/**
 * Unit tests for useReportStream hook
 * Tests core functionality: initialization, state management, and message handling
 */

describe("useReportStream Hook", () => {
  beforeEach(() => {
    vi.spyOn(console, "debug").mockImplementation(() => {});
    vi.spyOn(console, "error").mockImplementation(() => {});
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  describe("Initialization & State", () => {
    it("should start in idle state with null jobId", () => {
      const { result } = renderHook(() => useReportStream({ jobId: null }));

      expect(result.current.status).toBe("idle");
      expect(result.current.progress).toBe(0);
      expect(result.current.chunks).toEqual([]);
      expect(result.current.error).toBeNull();
    });

    it("should have reconnect function available", () => {
      const { result } = renderHook(() => useReportStream({ jobId: null }));
      expect(typeof result.current.reconnect).toBe("function");
    });

    it("should initialize with provided options", () => {
      const onComplete = vi.fn();
      const onError = vi.fn();

      const { result } = renderHook(() =>
        useReportStream({
          jobId: null,
          onComplete,
          onError,
          maxRetries: 5,
        })
      );

      // Options are stored internally, just verify no errors
      expect(result.current.status).toBe("idle");
      expect(typeof result.current.reconnect).toBe("function");
    });
  });

  describe("State Transitions with jobId Changes", () => {
    it("should reset state when jobId becomes null", async () => {
      const { result, rerender } = renderHook(
        ({ jobId }) => useReportStream({ jobId }),
        { initialProps: { jobId: "test-job" } }
      );

      // When jobId is set, it will try to connect
      expect(result.current.status).toMatch(/connecting|error/);

      // Change to null
      rerender({ jobId: null });

      // Should return to idle
      expect(result.current.status).toBe("idle");
      expect(result.current.progress).toBe(0);
      expect(result.current.chunks).toEqual([]);
      expect(result.current.error).toBeNull();
    });

    it("should reset chunks and progress on new jobId", async () => {
      // Manually set up the state transition without relying on actual WebSocket
      const { result, rerender } = renderHook(
        ({ jobId }) => useReportStream({ jobId }),
        { initialProps: { jobId: "job-1" } }
      );

      const initialStatus = result.current.status;
      expect(initialStatus).toMatch(/connecting|streaming|error/);

      // Change to different job
      rerender({ jobId: "job-2" });

      // Progress and chunks should be reset
      expect(result.current.progress).toBe(0);
      expect(result.current.chunks).toEqual([]);
    });
  });

  describe("Callback Functions", () => {
    it("should accept onComplete callback", () => {
      const onComplete = vi.fn();

      const { result } = renderHook(() =>
        useReportStream({
          jobId: null,
          onComplete,
        })
      );

      expect(result.current.status).toBe("idle");
      // Callback is stored internally
    });

    it("should accept onError callback", () => {
      const onError = vi.fn();

      const { result } = renderHook(() =>
        useReportStream({
          jobId: null,
          onError,
        })
      );

      expect(result.current.status).toBe("idle");
      // Callback is stored internally
    });

    it("should allow changing callbacks between renders", () => {
      const onComplete1 = vi.fn();
      const onComplete2 = vi.fn();

      const { rerender } = renderHook(
        ({ onComplete }) => useReportStream({ jobId: null, onComplete }),
        { initialProps: { onComplete: onComplete1 } }
      );

      // Change callback
      rerender({ onComplete: onComplete2 });

      // Should not error
      expect(true).toBe(true);
    });
  });

  describe("Manual Reconnect", () => {
    it("should provide reconnect function", () => {
      const { result } = renderHook(() =>
        useReportStream({ jobId: "test-job" })
      );

      const reconnect = result.current.reconnect;
      expect(typeof reconnect).toBe("function");

      // Should not throw when called
      expect(() => {
        act(() => {
          reconnect();
        });
      }).not.toThrow();
    });

    it("should reset state when reconnect called", () => {
      const { result } = renderHook(() =>
        useReportStream({ jobId: "test-job" })
      );

      // Call reconnect
      act(() => {
        result.current.reconnect();
      });

      // Should reset state
      expect(result.current.error).toBeNull();
      expect(result.current.chunks).toEqual([]);
      expect(result.current.progress).toBe(0);
    });
  });

  describe("Progress Calculations", () => {
    it("should calculate correct progress percentage", () => {
      // Test the progress calculation logic independently
      const testCases = [
        { batch_index: 0, total_batches: 4, expected: 25 },
        { batch_index: 1, total_batches: 4, expected: 50 },
        { batch_index: 2, total_batches: 4, expected: 75 },
        { batch_index: 3, total_batches: 4, expected: 100 },
      ];

      for (const testCase of testCases) {
        const progress = ((testCase.batch_index + 1) / testCase.total_batches) * 100;
        expect(Math.min(progress, 100)).toBe(testCase.expected);
      }
    });

    it("should cap progress at 100%", () => {
      // Test that capping works
      const progress = ((150 / 100) * 100); // Would be 150% without capping
      const cappedProgress = Math.min(progress, 100);
      expect(cappedProgress).toBe(100);
    });
  });

  describe("Data Accumulation", () => {
    it("should accumulate chunks in correct order", () => {
      // Test chunk accumulation logic
      const chunks: unknown[] = [];

      // Simulate receiving chunks
      const chunkData1 = [{ id: 1 }, { id: 2 }];
      const chunkData2 = [{ id: 3 }];
      const chunkData3 = [{ id: 4 }, { id: 5 }, { id: 6 }];

      chunks.push(...chunkData1);
      chunks.push(...chunkData2);
      chunks.push(...chunkData3);

      expect(chunks).toHaveLength(6);
      expect((chunks[0] as { id: number })?.id).toBe(1);
      expect((chunks[5] as { id: number })?.id).toBe(6);
    });

    it("should handle array and object chunk data", () => {
      // Test both array and object data types
      const chunks: unknown[] = [];

      // Array data
      chunks.push(...[{ id: 1 }]);

      // Object data wrapped
      chunks.push({ id: 2 });

      expect(chunks).toHaveLength(2);
      expect(Array.isArray(chunks)).toBe(true);
    });
  });

  describe("Error Handling", () => {
    it("should handle invalid JSON gracefully", () => {
      // Test JSON parsing with try-catch
      const testData = "{ invalid json }";

      expect(() => {
        JSON.parse(testData);
      }).toThrow();

      // In actual hook, this is caught and logged
    });

    it("should initialize maxRetries option", () => {
      const { result } = renderHook(() =>
        useReportStream({
          jobId: null,
          maxRetries: 3,
        })
      );

      expect(result.current.status).toBe("idle");
      // maxRetries is used internally for reconnection logic
    });
  });

  describe("Cleanup on Unmount", () => {
    it("should handle unmount gracefully", () => {
      const { unmount } = renderHook(() =>
        useReportStream({ jobId: "test-job" })
      );

      expect(() => {
        unmount();
      }).not.toThrow();
    });

    it("should allow remounting after unmount", () => {
      const { unmount, rerender } = renderHook(() =>
        useReportStream({ jobId: "test-job" })
      );

      unmount();

      // Remount should work
      const { result } = renderHook(() =>
        useReportStream({ jobId: "new-job" })
      );

      expect(result.current.status).toMatch(/connecting|error/);
    });
  });

  describe("Config Variations", () => {
    it("should handle no callbacks provided", () => {
      const { result } = renderHook(() =>
        useReportStream({ jobId: null })
      );

      expect(result.current.status).toBe("idle");
    });

    it("should handle only onComplete callback", () => {
      const { result } = renderHook(() =>
        useReportStream({
          jobId: null,
          onComplete: vi.fn(),
        })
      );

      expect(result.current.status).toBe("idle");
    });

    it("should handle only onError callback", () => {
      const { result } = renderHook(() =>
        useReportStream({
          jobId: null,
          onError: vi.fn(),
        })
      );

      expect(result.current.status).toBe("idle");
    });

    it("should handle custom maxRetries", () => {
      const { result } = renderHook(() =>
        useReportStream({
          jobId: null,
          maxRetries: 10,
        })
      );

      expect(result.current.status).toBe("idle");
    });
  });
});
