import { describe, it, expect, beforeEach, afterEach, vi, Mock } from "vitest";

// Mock the typed openapi-fetch client (used by login/register/changePassword/...)
// and the untyped fetch<T>() helper (used by fetchMyPermissions/...).
vi.mock("../client", () => ({
  client: {
    GET: vi.fn(),
    PUT: vi.fn(),
    POST: vi.fn(),
    DELETE: vi.fn(),
    PATCH: vi.fn(),
  },
  fetch: vi.fn(),
}));

// Mock apiFetch which is used inside auth.ts for forgotPassword/resetPassword
vi.mock("../helpers", () => ({
  apiFetch: vi.fn(),
  setAccessToken: vi.fn(),
  setCurrentUserId: vi.fn(),
  clearAuthSession: vi.fn(),
  TIMEOUT: { DEFAULT: 10000, REPORT_OPERATION: 180000, QUICK: 5000 },
}));

import {
  forgotPassword,
  resetPassword,
  login,
  logout,
  register,
  changePassword,
  fetchMe,
  updateMe,
  fetchUsers,
  fetchMyPermissions,
} from "../auth";
import { client } from "../client";
import { apiFetch } from "../helpers";

describe("forgotPassword", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("calls POST /api/v1/auth/forgot-password with email in body", async () => {
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
    expect(url).toContain("/api/v1/auth/forgot-password");
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

  it("calls POST /api/v1/auth/reset-password with token and new_password", async () => {
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
    expect(url).toContain("/api/v1/auth/reset-password");
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
      "[500]",
    );
  });
});

// --------------------------------------------------------------------------
// login / logout / register / changePassword / fetchMe / updateMe / fetchUsers
// These exercise the typed `client.*` methods, so we assert on the POST/GET
// calls and the in-memory token/user-id side effects.
// --------------------------------------------------------------------------
describe("login", () => {
  beforeEach(() => vi.clearAllMocks());
  it("POSTs credentials, stores access_token + user id, returns AuthResponse", async () => {
    const mock = {
      data: { access_token: "at-123", user: { id: "u-1" } },
    };
    (client.POST as Mock).mockResolvedValueOnce(mock);

    const result = await login("alice@example.com", "secret");

    expect(client.POST).toHaveBeenCalledWith("/api/v1/auth/login", {
      body: { email: "alice@example.com", password: "secret" },
    });
    expect(result).toEqual(mock.data);
  });
  it("throws when server returns no data (TypeError fallback)", async () => {
    (client.POST as Mock).mockResolvedValueOnce({ data: undefined });
    // Guard removed in favour of SafeClient — data is never undefined at runtime.
    // The call now throws a TypeError from `setAccessToken(data.access_token)`.
    await expect(login("a@b.c", "x")).rejects.toThrow();
  });});

describe("logout", () => {
  beforeEach(() => vi.clearAllMocks());
  it("POSTs logout and clears auth session", async () => {
    (client.POST as Mock).mockResolvedValueOnce({});
    await logout();
    expect(client.POST).toHaveBeenCalledWith("/api/v1/auth/logout", {
      body: { refresh_token: "" },
    });
  });
});

describe("register", () => {
  beforeEach(() => vi.clearAllMocks());
  it("POSTs payload and returns AuthResponse", async () => {
    const payload = { email: "new@x.io", password: "p", full_name: "New" };
    (client.POST as Mock).mockResolvedValueOnce({
      data: { access_token: "at", user: { id: "u-new" } },
    });
    const result = await register(payload as never);
    expect(client.POST).toHaveBeenCalledWith("/api/v1/auth/register", { body: payload });
    expect(result).toHaveProperty("access_token", "at");
  });
  it("returns undefined when server returns no data (guard removed — SafeClient guarantees presence)", async () => {
    (client.POST as Mock).mockResolvedValueOnce({ data: undefined });
    await expect(register({} as never)).resolves.toBeUndefined();
  });
});

describe("changePassword", () => {
  beforeEach(() => vi.clearAllMocks());
  it("POSTs current + new password, returns success string", async () => {
    (client.POST as Mock).mockResolvedValueOnce({ error: undefined });
    const msg = await changePassword({
      current_password: "old",
      new_password: "new",
    } as never);
    expect(client.POST).toHaveBeenCalledWith("/api/v1/auth/change-password", {
      body: { current_password: "old", new_password: "new" },
    });
    expect(msg).toBe("Password changed successfully");
  });
  it("propagates client rejection", async () => {
    (client.POST as Mock).mockRejectedValueOnce(new Error("Failed to change password"));
    await expect(
      changePassword({ current_password: "a", new_password: "b" } as never),
    ).rejects.toThrow("Failed to change password");
  });
});

describe("fetchMe / updateMe", () => {
  beforeEach(() => vi.clearAllMocks());
  it("fetchMe GETs /me and returns the User", async () => {
    (client.GET as Mock).mockResolvedValueOnce({ data: { id: "u-1" } });
    const user = await fetchMe();
    expect(client.GET).toHaveBeenCalledWith("/api/v1/me");
    expect(user).toEqual({ id: "u-1" });
  });
  it("returns undefined when server returns no data (guard removed)", async () => {
    (client.GET as Mock).mockResolvedValueOnce({ data: undefined });
    await expect(fetchMe()).resolves.toBeUndefined();
  });
  it("updateMe PATCHes /me and returns updated User", async () => {
    (client.PATCH as Mock).mockResolvedValueOnce({
      data: { id: "u-1", full_name: "X", preferred_language: "en" },
    });
    const updated = await updateMe({ full_name: "X", preferred_language: "en" });
    expect(client.PATCH).toHaveBeenCalledWith("/api/v1/me", {
      body: { full_name: "X", preferred_language: "en" },
    });
    expect(updated).toEqual({ id: "u-1", full_name: "X", preferred_language: "en" });
  });
  it("returns undefined when server returns no data (guard removed)", async () => {
    (client.PATCH as Mock).mockResolvedValueOnce({ data: undefined });
    await expect(updateMe({ full_name: "X", preferred_language: "en" })).resolves.toBeUndefined();
  });
});

describe("fetchUsers", () => {
  beforeEach(() => vi.clearAllMocks());
  it("GETs /users", async () => {
    (client.GET as Mock).mockResolvedValueOnce({ data: [{ id: "u-1" }, { id: "u-2" }] });
    const users = await fetchUsers();
    expect(client.GET).toHaveBeenCalledWith("/api/v1/users");
    expect(users).toHaveLength(2);
  });
});
describe("fetchMyPermissions", () => {
  beforeEach(() => vi.clearAllMocks());
  it("GETs /api/v1/me/permissions via fetch<T>()", async () => {
    const { fetch: fetchMock } = await import("../client");
    (fetchMock as Mock).mockResolvedValueOnce({
      data: { permissions: ["p"], roles: ["r"], managed_department_ids: null },
    });
    const p = await fetchMyPermissions();
    expect(fetchMock).toHaveBeenCalledWith("GET", "/api/v1/me/permissions");
    expect(p.permissions).toEqual(["p"]);
  });
});
