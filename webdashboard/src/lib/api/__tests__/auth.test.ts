import { describe, it, expect, beforeEach, afterEach, vi, Mock } from "vitest";

// Mock apiFetch which is used inside auth.ts for forgotPassword/resetPassword
vi.mock("../helpers", () => ({
  apiFetch: vi.fn(),
  TIMEOUT: { DEFAULT: 10000, REPORT_OPERATION: 180000, QUICK: 5000 },
}));

import { forgotPassword, resetPassword } from "../auth";
import { apiFetch } from "../helpers";

describe("forgotPassword", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("calls POST /v1/auth/forgot-password with email in body", async () => {
    const mockApiFetch = apiFetch as Mock;
    mockApiFetch.mockResolvedValue(
      new Response(JSON.stringify({ message: "ok" }), {
        status: 200,
        statusText: "OK",
      }),
    );

    await forgotPassword("user@example.com");

    expect(mockApiFetch).toHaveBeenCalledTimes(1);
    const [url, options] = mockApiFetch.mock.calls[0];
    expect(url).toContain("/v1/auth/forgot-password");
    expect(options.method).toBe("POST");
    expect(options.body).toBe(JSON.stringify({ email: "user@example.com" }));
    expect(options.timeout).toBe(10000);
  });

  it("throws on non-OK response with error body message", async () => {
    const mockApiFetch = apiFetch as Mock;
    mockApiFetch.mockResolvedValue(
      new Response(JSON.stringify({ message: "SMTP not configured" }), {
        status: 500,
        statusText: "Internal Server Error",
      }),
    );

    await expect(forgotPassword("user@example.com")).rejects.toThrow(
      "SMTP not configured",
    );
  });

  it("throws generic error when response body has no message", async () => {
    const mockApiFetch = apiFetch as Mock;
    mockApiFetch.mockResolvedValue(
      new Response("", {
        status: 500,
        statusText: "Internal Server Error",
      }),
    );

    await expect(forgotPassword("user@example.com")).rejects.toThrow(
      "Failed to send reset link",
    );
  });

  it("succeeds silently on 200 OK (anti-enumeration)", async () => {
    const mockApiFetch = apiFetch as Mock;
    mockApiFetch.mockResolvedValue(
      new Response(JSON.stringify({}), {
        status: 200,
        statusText: "OK",
      }),
    );

    // Should not throw for any reason
    await expect(
      forgotPassword("nonexistent@example.com"),
    ).resolves.toBeUndefined();
  });
});

describe("resetPassword", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("calls POST /v1/auth/reset-password with token and new_password", async () => {
    const mockApiFetch = apiFetch as Mock;
    mockApiFetch.mockResolvedValue(
      new Response(JSON.stringify({ message: "Password reset successfully" }), {
        status: 200,
        statusText: "OK",
      }),
    );

    const result = await resetPassword("reset-token-123", "newSecurePass1");

    expect(mockApiFetch).toHaveBeenCalledTimes(1);
    const [url, options] = mockApiFetch.mock.calls[0];
    expect(url).toContain("/v1/auth/reset-password");
    expect(options.method).toBe("POST");
    expect(options.body).toBe(
      JSON.stringify({ token: "reset-token-123", new_password: "newSecurePass1" }),
    );
    expect(result).toBe("Password reset successfully");
  });

  it("returns default message when response body has no message field", async () => {
    const mockApiFetch = apiFetch as Mock;
    mockApiFetch.mockResolvedValue(
      new Response(JSON.stringify({}), {
        status: 200,
        statusText: "OK",
      }),
    );

    const result = await resetPassword("token", "password123");
    expect(result).toBe("Password reset successfully");
  });

  it("throws on non-OK response with error body message", async () => {
    const mockApiFetch = apiFetch as Mock;
    mockApiFetch.mockResolvedValue(
      new Response(
        JSON.stringify({ message: "This reset link is invalid or has expired." }),
        {
          status: 400,
          statusText: "Bad Request",
        },
      ),
    );

    await expect(resetPassword("bad-token", "password123")).rejects.toThrow(
      "This reset link is invalid or has expired.",
    );
  });

  it("throws generic error when response is not OK and has no parseable body", async () => {
    const mockApiFetch = apiFetch as Mock;
    mockApiFetch.mockResolvedValue(
      new Response("Internal Server Error", {
        status: 500,
        statusText: "Internal Server Error",
      }),
    );

    await expect(resetPassword("token", "password123")).rejects.toThrow(
      "Failed to reset password",
    );
  });
});
