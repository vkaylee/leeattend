/**
 * @vitest-environment jsdom
 */
import { renderHook } from "@testing-library/react";

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";

import { useInfiniteScroll } from "../use-infinite-scroll";

describe("useInfiniteScroll", () => {
    let observeMock: ReturnType<typeof vi.fn>;
    let disconnectMock: ReturnType<typeof vi.fn>;
    let originalIO: typeof IntersectionObserver;
    let capturedCallback: IntersectionObserverCallback | null = null;

    beforeEach(() => {
        observeMock = vi.fn();
        disconnectMock = vi.fn();
        capturedCallback = null;
        originalIO = window.IntersectionObserver;

        // Use a class-style mock so `new IntersectionObserver(...)` works
        class MockIO {
            constructor(callback: IntersectionObserverCallback) {
                capturedCallback = callback;
            }
            observe = observeMock;
            unobserve = vi.fn();
            disconnect = disconnectMock;
        }
 
        window.IntersectionObserver = MockIO as unknown as typeof IntersectionObserver;
    });

    afterEach(() => {
        window.IntersectionObserver = originalIO;
    });

    it("returns a ref object", () => {
        const { result } = renderHook(() =>
            useInfiniteScroll({
                hasNextPage: true,
                isFetchingNextPage: false,
                fetchNextPage: vi.fn(),
            })
        );
        expect(result.current).toBeDefined();
        expect(result.current.current).toBeNull();
    });

    it("calls fetchNextPage when intersection fires with hasNextPage=true", () => {
        const fetchNextPage = vi.fn();
        const div = document.createElement("div");
        document.body.appendChild(div);

        renderHook(() => {
            const ref = useInfiniteScroll<HTMLDivElement>({
                hasNextPage: true,
                isFetchingNextPage: false,
                fetchNextPage,
            });
            // Simulate ref attachment
 
            (ref as unknown as { current: HTMLDivElement | null }).current = div;
            return ref;
        });

        // Simulate intersection
        capturedCallback?.(
            [{ isIntersecting: true } as IntersectionObserverEntry],
            {} as IntersectionObserver
        );
        expect(fetchNextPage).toHaveBeenCalled();
        document.body.removeChild(div);
    });

    it("does NOT call fetchNextPage when isFetchingNextPage is true", () => {
        const fetchNextPage = vi.fn();
        const div = document.createElement("div");
        document.body.appendChild(div);

        renderHook(() => {
            const ref = useInfiniteScroll<HTMLDivElement>({
                hasNextPage: true,
                isFetchingNextPage: true,
                fetchNextPage,
            });
 
            (ref as unknown as { current: HTMLDivElement | null }).current = div;
            return ref;
        });

        capturedCallback?.(
            [{ isIntersecting: true } as IntersectionObserverEntry],
            {} as IntersectionObserver
        );
        expect(fetchNextPage).not.toHaveBeenCalled();
        document.body.removeChild(div);
    });

    it("does NOT call fetchNextPage when hasNextPage is false", () => {
        const fetchNextPage = vi.fn();
        const div = document.createElement("div");
        document.body.appendChild(div);

        renderHook(() => {
            const ref = useInfiniteScroll<HTMLDivElement>({
                hasNextPage: false,
                isFetchingNextPage: false,
                fetchNextPage,
            });
 
            (ref as unknown as { current: HTMLDivElement | null }).current = div;
            return ref;
        });

        capturedCallback?.(
            [{ isIntersecting: true } as IntersectionObserverEntry],
            {} as IntersectionObserver
        );
        expect(fetchNextPage).not.toHaveBeenCalled();
        document.body.removeChild(div);
    });
});
