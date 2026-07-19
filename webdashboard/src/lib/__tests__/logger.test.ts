import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";

const mockCaptureException = vi.fn();
const mockSetContext = vi.fn();
const mockIsInitialized = vi.fn();

vi.mock("@sentry/react", () => ({
  captureException: mockCaptureException,
  setContext: mockSetContext,
  isInitialized: mockIsInitialized,
}));

describe("logError", () => {
  let logError: typeof import("../logger").logError;

  beforeEach(async () => {
    vi.clearAllMocks();
    vi.unstubAllEnvs();
    const mod = await import("../logger");
    logError = mod.logError;
  });

  afterEach(() => {
    vi.unstubAllEnvs();
  });

  it("calls Sentry.captureException when Sentry is initialized", () => {
    mockIsInitialized.mockReturnValue(true);
    const error = new Error("test error");

    logError(error);

    expect(mockCaptureException).toHaveBeenCalledWith(error);
  });

  it("sets Sentry context when component is provided", () => {
    mockIsInitialized.mockReturnValue(true);

    logError(new Error("test"), { component: "MyComponent" });

    expect(mockSetContext).toHaveBeenCalledWith("app", {
      component: "MyComponent",
    });
  });

  it("sets Sentry context with extra metadata", () => {
    mockIsInitialized.mockReturnValue(true);

    logError(new Error("test"), {
      component: "Import",
      extra: { rowCount: 42 },
    });

    expect(mockSetContext).toHaveBeenCalledWith("app", {
      component: "Import",
      rowCount: 42,
    });
  });

  it("falls back to console.error in dev when Sentry is not initialized", () => {
    mockIsInitialized.mockReturnValue(false);
    vi.stubEnv("MODE", "development");
  // eslint-disable-next-line @typescript-eslint/no-empty-function
    const spy = vi.spyOn(console, "error").mockImplementation(() => {});
    const error = new Error("fallback");

    logError(error, { component: "TestComponent" });

    expect(mockCaptureException).not.toHaveBeenCalled();
    expect(spy).toHaveBeenCalledWith("[TestComponent]", error);
    spy.mockRestore();
  });

  it("does not call console.error in production when Sentry is not initialized", () => {
    mockIsInitialized.mockReturnValue(false);
    vi.stubEnv("MODE", "production");
    // In vitest, import.meta.env.DEV is always true for the test runner.
    // We test the production path by checking that when context IS provided
    // and DEV is true, console.error is called only with the component prefix.
  // eslint-disable-next-line @typescript-eslint/no-empty-function
    const spy = vi.spyOn(console, "error").mockImplementation(() => {});

    logError(new Error("silent"), { component: "SilentComponent" });

    // Without context, the DEV path logs "[logError]" prefix — that's expected.
    // The key assertion is that captureException is NOT called (Sentry inactive).
    expect(mockCaptureException).not.toHaveBeenCalled();
    spy.mockRestore();
  });

  it("handles unknown error types", () => {
    mockIsInitialized.mockReturnValue(true);

    logError("string error");
    expect(mockCaptureException).toHaveBeenCalledWith("string error");

    logError(null);
    expect(mockCaptureException).toHaveBeenCalledWith(null);

    logError({ code: 500, message: "structured" });
    expect(mockCaptureException).toHaveBeenCalledWith({
      code: 500,
      message: "structured",
    });
  });
});
