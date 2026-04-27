import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";

const mockInit = vi.fn();
const mockBrowserTracingIntegration = vi.fn(() => ({}));
const mockReplayIntegration = vi.fn(() => ({}));

vi.mock("@sentry/react", () => ({
  init: mockInit,
  browserTracingIntegration: mockBrowserTracingIntegration,
  replayIntegration: mockReplayIntegration,
  ErrorBoundary: ({ children }: any) => children,
}));

describe("initSentry", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.unstubAllEnvs();
  });

  afterEach(() => {
    vi.unstubAllEnvs();
  });

  it("returns false when VITE_SENTRY_DSN is not set", async () => {
    vi.stubEnv("VITE_SENTRY_DSN", undefined as any);
    const { initSentry } = await import("../sentry");

    const result = initSentry();

    expect(result).toBe(false);
    expect(mockInit).not.toHaveBeenCalled();
  });

  it("returns false when VITE_SENTRY_DSN is empty", async () => {
    vi.stubEnv("VITE_SENTRY_DSN", "");
    const { initSentry } = await import("../sentry");

    const result = initSentry();

    expect(result).toBe(false);
    expect(mockInit).not.toHaveBeenCalled();
  });

  it("initializes Sentry when VITE_SENTRY_DSN is set", async () => {
    vi.stubEnv("VITE_SENTRY_DSN", "https://key@sentry.io/project");
    vi.stubEnv("MODE", "production");
    const { initSentry } = await import("../sentry");

    const result = initSentry();

    expect(result).toBe(true);
    expect(mockInit).toHaveBeenCalledWith(
      expect.objectContaining({
        dsn: "https://key@sentry.io/project",
        environment: "production",
      }),
    );
    expect(mockBrowserTracingIntegration).toHaveBeenCalled();
    expect(mockReplayIntegration).toHaveBeenCalled();
  });
});
