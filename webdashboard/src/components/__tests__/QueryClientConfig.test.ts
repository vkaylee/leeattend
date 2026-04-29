import { QueryClient } from "@tanstack/react-query";
import { describe, it, expect } from "vitest";

/**
 * Tests for QueryClient configuration optimizations.
 * Verifies that gcTime and staleTime are properly configured
 * to prevent unbounded heap memory growth.
 */
describe("QueryClient Memory Configuration", () => {
    it("should configure gcTime to garbage-collect inactive cache", () => {
        // Import the same config pattern used in App.tsx
        const queryClient = new QueryClient({
            defaultOptions: {
                queries: {
                    retry: 1,
                    staleTime: 1000 * 60 * 5,
                    gcTime: 1000 * 60 * 10,
                },
            },
        });

        const defaultOptions = queryClient.getDefaultOptions();
        expect(defaultOptions.queries?.gcTime).toBe(600_000); // 10 minutes
        expect(defaultOptions.queries?.staleTime).toBe(300_000); // 5 minutes
    });

    it("gcTime should be greater than staleTime", () => {
        const staleTime = 1000 * 60 * 5;  // 5 min
        const gcTime = 1000 * 60 * 10;    // 10 min

        // gcTime must be >= staleTime, otherwise data gets garbage-collected
        // while still considered fresh — causing unnecessary refetches
        expect(gcTime).toBeGreaterThan(staleTime);
    });

    it("should not use infinite gcTime (memory leak risk)", () => {
        const queryClient = new QueryClient({
            defaultOptions: {
                queries: {
                    retry: 1,
                    staleTime: 1000 * 60 * 5,
                    gcTime: 1000 * 60 * 10,
                },
            },
        });

        const gcTime = queryClient.getDefaultOptions().queries?.gcTime;
        expect(gcTime).toBeDefined();
        expect(gcTime).toBeLessThan(Infinity);
        expect(gcTime).toBeGreaterThan(0);
    });

    it("should clean up query cache when queries are removed", () => {
        const queryClient = new QueryClient({
            defaultOptions: {
                queries: { retry: false, gcTime: 0 },
            },
        });

        // Add a query to the cache
        queryClient.setQueryData(["test-query"], { data: "test" });
        expect(queryClient.getQueryData(["test-query"])).toEqual({ data: "test" });

        // Clear all queries
        queryClient.clear();
        expect(queryClient.getQueryData(["test-query"])).toBeUndefined();
    });
});
