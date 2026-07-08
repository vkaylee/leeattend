import { QueryClient } from "@tanstack/react-query";

import { describe, expect, it, vi } from "vitest";

import { createAppRouter } from "../router";

// Mock @tanstack/react-router's createRouter to return options for testing
vi.mock("@tanstack/react-router", async (importOriginal) => {
  const original = await importOriginal<typeof import("@tanstack/react-router")>();
  return {
    ...original,
    createRouter: vi.fn((options) => options),
  };
});

describe("createAppRouter Configuration", () => {
  it("should configure router with optimal performance defaults to prevent navigation lag", () => {
    const queryClient = new QueryClient();
    const config = createAppRouter(queryClient) as unknown as {
      defaultPendingMinMs: number;
      defaultPendingMs: number;
      defaultPreload: string;
      defaultPreloadStaleTime: number;
      scrollRestoration: boolean;
    };

    expect(config.defaultPreload).toBe("intent");
    expect(config.defaultPreloadStaleTime).toBe(30000);
    expect(config.defaultPendingMs).toBe(200);
    expect(config.defaultPendingMinMs).toBe(300);
    expect(config.scrollRestoration).toBe(true);
  });
});
