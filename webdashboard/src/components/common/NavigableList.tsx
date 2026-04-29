import React, { RefObject, useEffect, useRef, useState, useCallback } from "react";

import { SearchX, Loader2, ChevronRight } from "lucide-react";
import { useTranslation } from "react-i18next";

import InfiniteScrollSentinel from "@/components/common/InfiniteScrollSentinel";
import SearchBox from "@/components/common/SearchBox";

import { useInfiniteScroll } from "@/hooks/use-infinite-scroll";
import { useKeyboardListNav } from "@/hooks/use-keyboard-list-nav";
import { useStickyHeader } from "@/hooks/use-sticky-header";
import { cn } from "@/lib/utils";



// ---------------------------------------------------------------------------
// Column definition
// ---------------------------------------------------------------------------

export interface NavigableListColumn<T> {
    /** Unique key for this column */
    key: string;
    /** Header label (already translated) — can be a ReactNode for sort icons, etc. */
    header: React.ReactNode;
    /** How to render a cell for a given row item */
    cell: (item: T, index: number) => React.ReactNode;
    /** Optional header className override */
    headerClassName?: string;
    /** Optional cell className override */
    cellClassName?: string;
    /** If true, column is hidden (useful for conditional columns) */
    hidden?: boolean;
    /** If provided, header becomes clickable (e.g. for sorting) */
    onHeaderClick?: () => void;
}

// ---------------------------------------------------------------------------
// Component props
// ---------------------------------------------------------------------------

export interface NavigableListProps<T> {
    /** Column definitions */
    columns: NavigableListColumn<T>[];
    /** Data items to render */
    items: T[];
    /** Unique key extractor for each item */
    itemKey: (item: T) => string;
    /** Called when user clicks a row or presses Enter on focused row */
    onSelect: (item: T) => void;
    /** Currently selected item key (for active highlight) */
    selectedKey?: string | null;
    /** Whether data is loading (show skeletons) */
    isLoading?: boolean;
    /** Number of skeleton rows to show while loading */
    skeletonRows?: number;
    /** Empty state message */
    emptyMessage?: string;
    /** Show a chevron at end of each row */
    showChevron?: boolean;
    /** Show an active indicator bar on selected row */
    showActiveBar?: boolean;
    /** Prefix for row element IDs (for scroll-into-view) */
    rowIdPrefix?: string;
    /** data-testid prefix for rows (item key appended) */
    testIdPrefix?: string;
    /** Extra deps to reset keyboard focus */
    resetDeps?: unknown[];

    // --- Search ---
    /** Search value (controlled) */
    searchValue?: string;
    /** Search change handler */
    onSearchChange?: (value: string) => void;
    /** Search placeholder */
    searchPlaceholder?: string;
    /** Ref for search input */
    searchInputRef?: RefObject<HTMLInputElement | null>;
    /** data-testid for search input */
    searchTestId?: string;

    // --- Infinite scroll ---
    hasNextPage?: boolean;
    isFetchingNextPage?: boolean;
    fetchNextPage?: () => void;

    // --- Appearance ---
    /** Container className override */
    className?: string;
    /** Max height CSS class (default: "max-h-[60vh]") */
    maxHeightClass?: string;
    /** Additional content above the table (filters, stats) */
    headerSlot?: React.ReactNode;
    /** Ref callback to get the internal keyboard handler (for external search inputs) */
    onKeyDownRef?: React.MutableRefObject<((e: React.KeyboardEvent<HTMLInputElement>) => void) | null>;
    /** Enable smart header: auto-hide on scroll down, reveal on scroll up (full-page mode only) */
    smartHeader?: boolean;
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

export default function NavigableList<T>({
    columns,
    items,
    itemKey,
    onSelect,
    selectedKey,
    isLoading = false,
    skeletonRows = 5,
    emptyMessage,
    showChevron = true,
    showActiveBar = false,
    rowIdPrefix = "nav-row",
    testIdPrefix,
    resetDeps = [],
    searchValue,
    onSearchChange,
    searchPlaceholder,
    searchInputRef,
    searchTestId,
    hasNextPage,
    isFetchingNextPage,
    fetchNextPage,
    className,
    maxHeightClass = "max-h-[60vh]",
    headerSlot,
    onKeyDownRef,
    smartHeader = false,
}: NavigableListProps<T>) {
    const { t } = useTranslation();

    // Keyboard navigation
    const listNav = useKeyboardListNav({
        items,
        onSelect,
        rowIdPrefix,
        resetDeps,
    });

    // Expose keyboard handler to external consumers
    useEffect(() => {
        if (onKeyDownRef) {
            onKeyDownRef.current = listNav.onKeyDown;
        }
    }, [onKeyDownRef, listNav.onKeyDown]);

    // Infinite scroll
    const loadMoreRef = useInfiniteScroll<HTMLTableRowElement>({
        hasNextPage: hasNextPage ?? false,
        isFetchingNextPage: isFetchingNextPage ?? false,
        fetchNextPage: fetchNextPage ?? (() => { }),
    });

    const visibleColumns = columns.filter((c) => !c.hidden);
    const colSpan = visibleColumns.length + (showChevron ? 1 : 0);

    // Measure sticky search bar height for thead offset in full-page mode
    const isFullPage = !maxHeightClass;
    const smartHeaderEnabled = isFullPage && smartHeader;
    const stickySearchRef = useRef<HTMLDivElement>(null);
    const [measuredSearchHeight, setMeasuredSearchHeight] = useState(0);

    // Smart header (auto-hide / reveal on scroll)
    const { headerRef: smartHeaderRef, isVisible: smartHeaderVisible, headerHeight: smartHeaderHeight } = useStickyHeader({
        enabled: smartHeaderEnabled,
    });

    // For non-smart-header full-page mode: measure search bar height via ResizeObserver
    useEffect(() => {
        if (!isFullPage || smartHeaderEnabled || !stickySearchRef.current) return;
        const el = stickySearchRef.current;
        const update = () => setMeasuredSearchHeight(el.offsetHeight);
        update();
        const ro = new ResizeObserver(update);
        ro.observe(el);
        return () => ro.disconnect();
    }, [isFullPage, smartHeaderEnabled]);

    // Compute offset: smart header mode derives from hook state, otherwise from measured search height
    const stickyOffset = smartHeaderEnabled
        ? (smartHeaderVisible ? smartHeaderHeight : 0)
        : (isFullPage ? measuredSearchHeight : 0);

    return (
        <div className={cn("flex flex-col gap-6", className)}>
            {/* Smart header wrapper — auto-hide on scroll down, reveal on scroll up */}
            {smartHeaderEnabled ? (
                <div
                    ref={smartHeaderRef}
                    className={cn(
                        "sticky top-0 z-20 backdrop-blur-md transition-transform duration-300 ease-out",
                        smartHeaderVisible ? "translate-y-0" : "-translate-y-full"
                    )}
                >
                    <div className="flex flex-col gap-6 pb-2">
                        {headerSlot}
                        {onSearchChange && (
                            <SearchBox
                                value={searchValue ?? ""}
                                onChange={onSearchChange}
                                onKeyDown={listNav.onKeyDown}
                                inputRef={searchInputRef}
                                placeholder={searchPlaceholder}
                                data-testid={searchTestId ?? "navigable-list-search"}
                            />
                        )}
                    </div>
                </div>
            ) : (
                <>
                    {/* Optional header slot (filters, stats) */}
                    {headerSlot}

                    {/* In full-page mode, search + header stick together as one group */}
                    {isFullPage && onSearchChange && (
                        <div ref={stickySearchRef} className="sticky top-0 z-20 backdrop-blur-md pt-1 pb-2 -mt-1">
                            <SearchBox
                                value={searchValue ?? ""}
                                onChange={onSearchChange}
                                onKeyDown={listNav.onKeyDown}
                                inputRef={searchInputRef}
                                placeholder={searchPlaceholder}
                                data-testid={searchTestId ?? "navigable-list-search"}
                            />
                        </div>
                    )}
                </>
            )}

            {/* In constrained mode, search renders normally */}
            {!isFullPage && onSearchChange && (
                <SearchBox
                    value={searchValue ?? ""}
                    onChange={onSearchChange}
                    onKeyDown={listNav.onKeyDown}
                    inputRef={searchInputRef}
                    placeholder={searchPlaceholder}
                    data-testid={searchTestId ?? "navigable-list-search"}
                />
            )}

            {/* Table container */}
            <div className={cn("bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-[2.5rem] shadow-sm", isFullPage ? "overflow-x-clip" : "overflow-hidden")}>
                <div className={cn(isFullPage ? "" : "overflow-auto", maxHeightClass)}>
                    <table className="w-full text-left border-collapse">
                        {/* Header */}
                        <thead>
                            <tr
                                className="bg-slate-50/95 dark:bg-slate-800/95 backdrop-blur-sm border-b border-slate-100 dark:border-slate-800 sticky z-10"
                                style={{ top: isFullPage ? stickyOffset : 0 }}
                            >
                                {visibleColumns.map((col) => (
                                    <th
                                        key={col.key}
                                        onClick={col.onHeaderClick}
                                        className={cn(
                                            "px-8 py-5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em]",
                                            col.onHeaderClick && "cursor-pointer hover:text-primary-600 dark:hover:text-primary-400 transition-colors group",
                                            col.headerClassName
                                        )}
                                    >
                                        {col.header}
                                    </th>
                                ))}
                                {showChevron && <th className="px-4 py-5 w-10" />}
                            </tr>
                        </thead>

                        {/* Body */}
                        <tbody className="divide-y divide-slate-50 dark:divide-slate-800">
                            {isLoading ? (
                                // Loading skeletons
                                Array.from({ length: skeletonRows }).map((_, i) => (
                                    <tr key={`skeleton-${i}`}>
                                        {visibleColumns.map((col) => (
                                            <td key={col.key} className={cn("px-8 py-5", col.cellClassName)}>
                                                <div className="h-4 w-24 bg-slate-200 dark:bg-slate-800 rounded animate-pulse" />
                                            </td>
                                        ))}
                                        {showChevron && <td className="px-4 py-5" />}
                                    </tr>
                                ))
                            ) : items.length === 0 ? (
                                // Empty state
                                <tr>
                                    <td colSpan={colSpan} className="px-10 py-32">
                                        <div className="flex flex-col items-center justify-center text-center animate-in fade-in zoom-in-95 duration-500 max-w-sm mx-auto">
                                            <div className="w-20 h-20 mb-6 rounded-3xl bg-slate-50 dark:bg-slate-800/80 flex items-center justify-center text-slate-300 dark:text-slate-600 shadow-inner border border-slate-100 dark:border-slate-700/50 rotate-3">
                                                <SearchX size={32} className="-rotate-3" />
                                            </div>
                                            <h3 className="text-sm font-bold text-slate-700 dark:text-slate-200 uppercase tracking-widest mb-2">
                                                {emptyMessage ?? t("common.no_data", "No data")}
                                            </h3>
                                        </div>
                                    </td>
                                </tr>
                            ) : (
                                // Data rows
                                items.map((item, index) => {
                                    const key = itemKey(item);
                                    const isSelected = selectedKey != null && key === selectedKey;
                                    const isFocused = listNav.focusedIndex === index;

                                    return (
                                        <tr
                                            key={key}
                                            id={`${rowIdPrefix}-${index}`}
                                            data-testid={testIdPrefix ? `${testIdPrefix}-${key}` : undefined}
                                            onClick={() => onSelect(item)}
                                            className={cn(
                                                "cursor-pointer transition-all group relative",
                                                isSelected
                                                    ? "bg-primary-50 dark:bg-primary-950/50"
                                                    : isFocused
                                                        ? "bg-primary-50/50 dark:bg-primary-900/20 ring-1 ring-inset ring-primary-500/50"
                                                        : "hover:bg-slate-50 dark:hover:bg-slate-800/50"
                                            )}
                                        >
                                            {visibleColumns.map((col) => (
                                                <td key={col.key} className={cn("px-8 py-5", col.cellClassName)}>
                                                    {col.cell(item, index)}
                                                </td>
                                            ))}
                                            {showChevron && (
                                                <td className="px-4 py-5 text-right">
                                                    <ChevronRight
                                                        size={16}
                                                        className={cn(
                                                            "transition-all duration-200 ml-auto",
                                                            isSelected
                                                                ? "translate-x-1 text-primary-600 dark:text-primary-400"
                                                                : "text-slate-300 dark:text-slate-600 group-hover:text-primary-500 dark:group-hover:text-primary-400 group-hover:translate-x-1"
                                                        )}
                                                    />
                                                </td>
                                            )}
                                            {showActiveBar && isSelected && (
                                                <div className="absolute left-0 top-0 bottom-0 w-1.5 bg-primary-600 rounded-r-full shadow-lg shadow-primary-500/50" />
                                            )}
                                        </tr>
                                    );
                                })
                            )}

                            {/* Infinite scroll sentinel */}
                            {fetchNextPage && (
                                <InfiniteScrollSentinel
                                    sentinelRef={loadMoreRef}
                                    isFetchingNextPage={isFetchingNextPage ?? false}
                                    colSpan={colSpan}
                                />
                            )}
                        </tbody>
                    </table>
                </div>
            </div>
        </div>
    );
}
