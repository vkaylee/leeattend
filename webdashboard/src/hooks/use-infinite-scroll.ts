import { useRef, useEffect } from "react";

/**
 * Custom hook that sets up an IntersectionObserver on a sentinel element
 * to trigger fetching the next page of an infinite query.
 *
 * Returns a ref to attach to the sentinel element (e.g. a `<tr>` or `<div>`).
 */
export function useInfiniteScroll<T extends HTMLElement = HTMLElement>({
    hasNextPage,
    isFetchingNextPage,
    fetchNextPage,
    threshold = 0.1,
}: {
    hasNextPage: boolean | undefined;
    isFetchingNextPage: boolean;
    fetchNextPage: () => void;
    threshold?: number;
}) {
    const sentinelRef = useRef<T>(null);

    useEffect(() => {
        if (!sentinelRef.current) return;
        const observer = new IntersectionObserver(
            (entries) => {
                if (entries[0].isIntersecting && hasNextPage && !isFetchingNextPage) {
                    fetchNextPage();
                }
            },
            { threshold }
        );
        observer.observe(sentinelRef.current);
        return () => observer.disconnect();
    }, [hasNextPage, isFetchingNextPage, fetchNextPage, threshold]);

    return sentinelRef;
}
