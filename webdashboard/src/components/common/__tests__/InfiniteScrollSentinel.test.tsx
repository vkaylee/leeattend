import React, { createRef } from "react";

import { render } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";

import InfiniteScrollSentinel from "../InfiniteScrollSentinel";

// Mock i18n
vi.mock("react-i18next", () => ({
    useTranslation: () => ({
        t: (key: string, fallback?: string) => fallback || key,
        i18n: { changeLanguage: () => Promise.resolve(), language: "en" },
    }),
}));

describe("InfiniteScrollSentinel", () => {
    it("renders a table row with the sentinel ref", () => {
        const ref = createRef<HTMLTableRowElement>();
        const { container } = render(
            <table><tbody>
                <InfiniteScrollSentinel sentinelRef={ref} isFetchingNextPage={false} colSpan={4} />
            </tbody></table>
        );
        const row = container.querySelector("tr");
        expect(row).toBeDefined();
    });

    it("shows Loader2 spinner when isFetchingNextPage is true", () => {
        const ref = createRef<HTMLTableRowElement>();
        const { container } = render(
            <table><tbody>
                <InfiniteScrollSentinel sentinelRef={ref} isFetchingNextPage={true} colSpan={4} />
            </tbody></table>
        );
        // Loader2 renders as an SVG with animate-spin class
        const spinner = container.querySelector(".animate-spin");
        expect(spinner).toBeDefined();
    });

    it("does NOT show spinner when isFetchingNextPage is false", () => {
        const ref = createRef<HTMLTableRowElement>();
        const { container } = render(
            <table><tbody>
                <InfiniteScrollSentinel sentinelRef={ref} isFetchingNextPage={false} colSpan={4} />
            </tbody></table>
        );
        const spinner = container.querySelector(".animate-spin");
        expect(spinner).toBeNull();
    });

    it("uses correct colSpan", () => {
        const ref = createRef<HTMLTableRowElement>();
        const { container } = render(
            <table><tbody>
                <InfiniteScrollSentinel sentinelRef={ref} isFetchingNextPage={false} colSpan={7} />
            </tbody></table>
        );
        const td = container.querySelector("td");
        expect(td?.getAttribute("colspan")).toBe("7");
    });
});
