import { useRef, useState, useEffect, useCallback } from "react";

/**
 * Smart Header hook — implements the "auto-hide on scroll down, reveal on scroll up" pattern.
 *
 * Attach `headerRef` to the sticky header element. The hook tracks window scroll
 * direction and returns `isVisible` (should the header be shown?) and `headerHeight`
 * (measured height for downstream offset calculations).
 */
export function useStickyHeader({
    /** Minimum scroll distance (px) before toggling visibility */
    threshold = 10,
    /** Whether the feature is enabled (allows conditional opt-in) */
    enabled = true,
} = {}) {
    const headerRef = useRef<HTMLDivElement>(null);
    const [isSticky, setIsSticky] = useState(false);
    const [isVisible, setIsVisible] = useState(true);
    const [headerHeight, setHeaderHeight] = useState(0);

    // Track last scroll position between frames
    const lastScrollY = useRef(0);
    const ticking = useRef(false);

    // ── Measure header height with ResizeObserver ────────────────────────
    useEffect(() => {
        if (!enabled || !headerRef.current) return;
        const el = headerRef.current;
        const update = () => setHeaderHeight(el.offsetHeight);
        update();
        const ro = new ResizeObserver(update);
        ro.observe(el);
        return () => ro.disconnect();
    }, [enabled]);

    // ── Scroll direction tracking ───────────────────────────────────────
    const handleScroll = useCallback(() => {
        if (!enabled) return;

        const currentScrollY = window.scrollY;

        // At the very top — reset everything
        if (currentScrollY < threshold) {
            setIsSticky(false);
            setIsVisible(true);
            lastScrollY.current = currentScrollY;
            return;
        }

        setIsSticky(true);

        const delta = currentScrollY - lastScrollY.current;

        if (delta > threshold) {
            // Scrolling DOWN past threshold → hide
            setIsVisible(false);
            lastScrollY.current = currentScrollY;
        } else if (delta < -threshold) {
            // Scrolling UP past threshold → show
            setIsVisible(true);
            lastScrollY.current = currentScrollY;
        }
        // Small movements within threshold → no change (prevents flickering)
    }, [enabled, threshold]);

    useEffect(() => {
        if (!enabled) return;

        const onScroll = () => {
            if (!ticking.current) {
                ticking.current = true;
                requestAnimationFrame(() => {
                    handleScroll();
                    ticking.current = false;
                });
            }
        };

        window.addEventListener("scroll", onScroll, { passive: true });
        return () => window.removeEventListener("scroll", onScroll);
    }, [enabled, handleScroll]);

    return {
        /** Ref to attach to the header wrapper element */
        headerRef,
        /** Whether header has scrolled past its natural position */
        isSticky,
        /** Whether header should be visible (scrolling up or at top) */
        isVisible,
        /** Measured height of the header element in px */
        headerHeight,
    };
}
