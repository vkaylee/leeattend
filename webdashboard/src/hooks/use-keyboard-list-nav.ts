import { useState, useEffect, useCallback } from "react";

interface UseKeyboardListNavOptions<T> {
    /** The list of items to navigate */
    items: T[];
    /** Callback when user presses Enter on a focused item */
    onSelect: (item: T) => void;
    /** Prefix for row element IDs, e.g. "summary-row" → getElementById("summary-row-0") */
    rowIdPrefix: string;
    /** Extra dependencies that should reset focusedIndex to 0 */
    resetDeps?: unknown[];
}

interface UseKeyboardListNavReturn {
    /** Currently focused row index */
    focusedIndex: number;
    /** Set focused index manually */
    setFocusedIndex: (index: number) => void;
    /** Keyboard event handler — attach to onKeyDown of search input */
    onKeyDown: (e: React.KeyboardEvent<HTMLInputElement>) => void;
}

/**
 * Reusable hook for keyboard-driven list navigation.
 *
 * Provides ArrowUp/ArrowDown navigation between rows and Enter to select.
 * Automatically scrolls the focused row into view and resets focus
 * when the items list changes.
 */
export function useKeyboardListNav<T>({
    items,
    onSelect,
    rowIdPrefix,
    resetDeps = [],
}: UseKeyboardListNavOptions<T>): UseKeyboardListNavReturn {
    const [focusedIndex, setFocusedIndex] = useState(0);


    // Reset focus when items or extra deps change (derived state)
    const [prevDeps, setPrevDeps] = useState<unknown[]>([items, ...resetDeps]);
    const depsChanged = prevDeps[0] !== items || resetDeps.some((dep, i) => dep !== prevDeps[i + 1]);
    const [hasNavigatedState, setHasNavigatedState] = useState(false);
    if (depsChanged) {
        setPrevDeps([items, ...resetDeps]);
        setFocusedIndex(0);
        setHasNavigatedState(false);
    }

    // Scroll the focused row into view (only after user navigates)
    useEffect(() => {
        if (!hasNavigatedState) return;

        if (focusedIndex === 0) {
            // Special case: scroll container to top to avoid sticky header overlap
            const row = document.getElementById(`${rowIdPrefix}-0`);
            const container = row?.closest(".overflow-auto");
            if (container) {
                container.scrollTo({ top: 0, behavior: "smooth" });
            } else {
                // Full-page mode: scroll the window to the row
                row?.scrollIntoView({ behavior: "smooth", block: "nearest" });
            }
        } else {
            const row = document.getElementById(`${rowIdPrefix}-${focusedIndex}`);
            if (row) {
                row.scrollIntoView({ behavior: "smooth", block: "nearest" });
            }
        }
    }, [focusedIndex, rowIdPrefix, hasNavigatedState]);

    const onKeyDown = useCallback(
        (e: React.KeyboardEvent<HTMLInputElement>) => {
            if (e.key === "ArrowDown") {
                e.preventDefault();
                setHasNavigatedState(true);
                setFocusedIndex((prev) => Math.min(prev + 1, items.length - 1));
            } else if (e.key === "ArrowUp") {
                e.preventDefault();
                setHasNavigatedState(true);
                setFocusedIndex((prev) => Math.max(prev - 1, 0));
            } else if (e.key === "Enter") {
                e.preventDefault();
                if (items.length > 0 && items[focusedIndex]) {
                    e.currentTarget.blur();
                    onSelect(items[focusedIndex]);
                }
            }
        },
        [items, focusedIndex, onSelect]
    );

    return { focusedIndex, setFocusedIndex, onKeyDown };
}

