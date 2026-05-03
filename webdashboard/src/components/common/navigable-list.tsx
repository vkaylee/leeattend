import { SearchX, ChevronRight } from "lucide-react";
import React, { RefObject, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import type { TFunction } from "i18next";

import InfiniteScrollSentinel from "@/components/common/infinite-scroll-sentinel";
import SearchBox from "@/components/common/search-box";
import { useInfiniteScroll } from "@/hooks/use-infinite-scroll";
import { useKeyboardListNav } from "@/hooks/use-keyboard-list-nav";
import { useStickyHeader } from "@/hooks/use-sticky-header";
import { cn } from "@/lib/utils";

// ---------------------------------------------------------------------------
// Column definition
// ---------------------------------------------------------------------------

export interface NavigableListColumn<T> {
    cell: (item: T, index: number) => React.ReactNode;
    cellClassName?: string;
    header: React.ReactNode;
    headerClassName?: string;
    hidden?: boolean;
    key: string;
    onHeaderClick?: () => void;
}

// ---------------------------------------------------------------------------
// Component props
// ---------------------------------------------------------------------------

export interface NavigableListProps<T> {
    className?: string;
    columns: NavigableListColumn<T>[];
    emptyMessage?: string;
    fetchNextPage?: () => void;
    hasNextPage?: boolean;
    headerSlot?: React.ReactNode;
    isFetchingNextPage?: boolean;
    isLoading?: boolean;
    itemKey: (item: T) => string;
    items: T[];
    maxHeightClass?: string;
    onKeyDownRef?: React.MutableRefObject<((e: React.KeyboardEvent<HTMLInputElement>) => void) | null>;
    onSearchChange?: (value: string) => void;
    onSelect: (item: T) => void;
    resetDeps?: unknown[];
    rowIdPrefix?: string;
    searchInputRef?: RefObject<HTMLInputElement | null>;
    searchPlaceholder?: string;
    searchTestId?: string;
    searchValue?: string;
    selectedKey?: string | null;
    showActiveBar?: boolean;
    showChevron?: boolean;
    skeletonRows?: number;
    smartHeader?: boolean;
    testIdPrefix?: string;
}

function NavigableListSkeleton<T>({ skeletonRows, visibleColumns, showChevron }: { skeletonRows: number; visibleColumns: NavigableListColumn<T>[]; showChevron: boolean }) {
    return (
        <>
            {Array.from({ length: skeletonRows }).map((_, index) => (
                <tr key={`skeleton-${index}`}>
                    {visibleColumns.map((col) => (
                        <td className={cn("px-8 py-5", col.cellClassName)} key={col.key}>
                            <div className="h-4 w-24 bg-slate-200 dark:bg-slate-800 rounded animate-pulse" />
                        </td>
                    ))}
                    {!!showChevron && <td className="px-4 py-5" />}
                </tr>
            ))}
        </>
    );
}


function NavigableListEmptyState({ colSpan, emptyMessage, t }: { colSpan: number; emptyMessage?: string; t: TFunction }) {
    return (
        <tr>
            <td className="px-10 py-32" colSpan={colSpan}>
                <div className="flex flex-col items-center justify-center text-center animate-in fade-in zoom-in-95 duration-500 max-w-sm mx-auto">
                    <div className="w-20 h-20 mb-6 rounded-3xl bg-slate-50 dark:bg-slate-800/80 flex items-center justify-center text-slate-300 dark:text-slate-600 shadow-inner border border-slate-100 dark:border-slate-700/50 rotate-3">
                        <SearchX className="-rotate-3" size={32} />
                    </div>
                    <h3 className="text-sm font-bold text-slate-700 dark:text-slate-200 uppercase tracking-widest mb-2">
                        {emptyMessage ?? t("common.no_data", "No data")}
                    </h3>
                </div>
            </td>
        </tr>
    );
}

function NavigableListRow<T>({
    item, index, listNav, isSelected, itemKey, testIdPrefix, rowIdPrefix, onSelect, visibleColumns, showChevron, showActiveBar
}: {
    index: number;
    isSelected: boolean;
    item: T;
    itemKey: (item: T) => string;
    listNav: any;
    onSelect: (item: T) => void;
    rowIdPrefix?: string;
    showActiveBar?: boolean;
    showChevron?: boolean;
    testIdPrefix?: string;
    visibleColumns: NavigableListColumn<T>[];
}) {
    const key = itemKey(item);
    const isFocused = listNav.focusedIndex === index;

    return (
        <tr
            className={cn(
                "cursor-pointer transition-all group relative",
                isSelected
                    ? "bg-primary-50 dark:bg-primary-950/50"
                    : isFocused
                        ? "bg-primary-50/50 dark:bg-primary-900/20 ring-1 ring-inset ring-primary-500/50"
                        : "hover:bg-slate-50 dark:hover:bg-slate-800/50"
            )}
            data-testid={testIdPrefix ? `${testIdPrefix}-${key}` : undefined}
            id={`${rowIdPrefix}-${index}`}
            key={key}
            onClick={() => { onSelect(item); }}
        >
            {visibleColumns.map((col) => (
                <td className={cn("px-8 py-5", col.cellClassName)} key={col.key}>
                    {col.cell(item, index)}
                </td>
            ))}
            {!!showChevron && (
                <td className="px-4 py-5 text-right">
                    <ChevronRight
                        className={cn(
                            "transition-all duration-200 ml-auto",
                            isSelected
                                ? "translate-x-1 text-primary-600 dark:text-primary-400"
                                : "text-slate-300 dark:text-slate-600 group-hover:text-primary-500 dark:group-hover:text-primary-400 group-hover:translate-x-1"
                        )}
                        size={16}
                    />
                </td>
            )}
            {!!showActiveBar && !!isSelected && <div className="absolute left-0 top-0 bottom-0 w-1.5 bg-primary-600 rounded-r-full shadow-lg shadow-primary-500/50" />}
        </tr>
    );
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

function NavigableListSearchContent({
    onSearchChange,
    searchTestId,
    searchInputRef,
    listNav,
    searchPlaceholder,
    searchValue,
}: {
    listNav: any;
    onSearchChange?: (value: string) => void;
    searchInputRef?: React.RefObject<HTMLInputElement | null>;
    searchPlaceholder?: string;
    searchTestId?: string;
    searchValue?: string;
}) {
    if (!onSearchChange) return null;
    return (
        <SearchBox
            data-testid={searchTestId ?? "navigable-list-search"}
            inputRef={searchInputRef}
            onChange={onSearchChange}
            onKeyDown={listNav.onKeyDown}
            placeholder={searchPlaceholder}
            value={searchValue ?? ""}
        />
    );
}

function NavigableListHeaderWrapper({
    smartHeaderEnabled,
    smartHeaderVisible,
    smartHeaderRef,
    headerSlot,
    isFullPage,
    stickySearchRef,
    searchProps,
}: {
    headerSlot?: React.ReactNode;
    isFullPage: boolean;
    searchProps: any;
    smartHeaderEnabled: boolean;
    smartHeaderRef: any;
    smartHeaderVisible: boolean;
    stickySearchRef: any;
}) {
    if (smartHeaderEnabled) {
        return (
            <div
                className={cn(
                    "sticky top-0 z-20 backdrop-blur-md transition-transform duration-300 ease-out",
                    smartHeaderVisible ? "translate-y-0" : "-translate-y-full"
                )}
                ref={smartHeaderRef}
            >
                <div className="flex flex-col gap-6 pb-2">
                    {headerSlot}
                    <NavigableListSearchContent {...searchProps} />
                </div>
            </div>
        );
    }

    return (
        <>
            {headerSlot}
            {!!isFullPage && !!searchProps.onSearchChange && (
                <div className="sticky top-0 z-20 backdrop-blur-md pt-1 pb-2 -mt-1" ref={stickySearchRef}>
                    <NavigableListSearchContent {...searchProps} />
                </div>
            )}
        </>
    );
}

function NavigableListTableContent<T>({
    isLoading,
    items,
    itemKey,
    selectedKey,
    listNav,
    onSelect,
    rowIdPrefix,
    showActiveBar,
    showChevron,
    testIdPrefix,
    visibleColumns,
    skeletonRows,
    colSpan,
    emptyMessage,
    t,
    fetchNextPage,
    isFetchingNextPage,
    loadMoreRef
}: {
    colSpan: number;
    emptyMessage?: string;
    fetchNextPage?: () => void;
    isFetchingNextPage?: boolean;
    isLoading: boolean;
    itemKey: (item: T) => string;
    items: T[];
    listNav: any;
    loadMoreRef: any;
    onSelect: (item: T) => void;
    rowIdPrefix?: string;
    selectedKey?: string | null;
    showActiveBar?: boolean;
    showChevron?: boolean;
    skeletonRows: number;
    t: any;
    testIdPrefix?: string;
    visibleColumns: NavigableListColumn<T>[];
}) {
    return (
        <tbody className="divide-y divide-slate-50 dark:divide-slate-800">
            {isLoading ? (
                <NavigableListSkeleton showChevron={!!showChevron} skeletonRows={skeletonRows} visibleColumns={visibleColumns} />
            ) : items.length === 0 ? (
                <NavigableListEmptyState colSpan={colSpan} emptyMessage={emptyMessage} t={t} />
            ) : (
                items.map((item, index) => {
                    const key = itemKey(item);
                    const isSelected = selectedKey != null && key === selectedKey;
                    return (
                        <NavigableListRow
                            index={index}
                            isSelected={isSelected}
                            item={item}
                            itemKey={itemKey}
                            key={key}
                            listNav={listNav}
                            onSelect={onSelect}
                            rowIdPrefix={rowIdPrefix}
                            showActiveBar={showActiveBar}
                            showChevron={showChevron}
                            testIdPrefix={testIdPrefix}
                            visibleColumns={visibleColumns}
                        />
                    );
                })
            )}

            {!!fetchNextPage && (
                <InfiniteScrollSentinel
                    colSpan={colSpan}
                    isFetchingNextPage={isFetchingNextPage ?? false}
                    sentinelRef={loadMoreRef}
                />
            )}
        </tbody>
    );
}

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

    const listNav = useKeyboardListNav({ items, onSelect, rowIdPrefix, resetDeps });

    useEffect(() => {
        if (onKeyDownRef) {
            onKeyDownRef.current = listNav.onKeyDown;
        }
    }, [onKeyDownRef, listNav.onKeyDown]);

    const loadMoreRef = useInfiniteScroll<HTMLTableRowElement>({
        hasNextPage: hasNextPage ?? false,
        isFetchingNextPage: isFetchingNextPage ?? false,
        fetchNextPage: fetchNextPage ?? (() => { /* empty */ }),
    });

    const visibleColumns = columns.filter((c) => !c.hidden);
    const colSpan = visibleColumns.length + (showChevron ? 1 : 0);

    const isFullPage = !maxHeightClass;
    const smartHeaderEnabled = isFullPage && smartHeader;
    const stickySearchRef = useRef<HTMLDivElement>(null);
    const [measuredSearchHeight, setMeasuredSearchHeight] = useState(0);

    const { headerRef: smartHeaderRef, isVisible: smartHeaderVisible, headerHeight: smartHeaderHeight } = useStickyHeader({
        enabled: smartHeaderEnabled,
    });

    useEffect(() => {
        if (!isFullPage || smartHeaderEnabled || !stickySearchRef.current) return;
        const element = stickySearchRef.current;
        const update = () => { setMeasuredSearchHeight(element.offsetHeight); };
        update();
        const ro = new ResizeObserver(update);
        ro.observe(element);
        return () => { ro.disconnect(); };
    }, [isFullPage, smartHeaderEnabled]);

    const stickyOffset = smartHeaderEnabled
        ? (smartHeaderVisible ? smartHeaderHeight : 0)
        : (isFullPage ? measuredSearchHeight : 0);

    const searchProps = { onSearchChange, searchTestId, searchInputRef, listNav, searchPlaceholder, searchValue };

    return (
        <div className={cn("flex flex-col gap-6", className)}>
            <NavigableListHeaderWrapper
                headerSlot={headerSlot}
                isFullPage={isFullPage}
                searchProps={searchProps}
                smartHeaderEnabled={smartHeaderEnabled}
                smartHeaderRef={smartHeaderRef}
                smartHeaderVisible={smartHeaderVisible}
                stickySearchRef={stickySearchRef}
            />

            {!isFullPage && <NavigableListSearchContent {...searchProps} />}

            <div className={cn("bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-[2.5rem] shadow-sm", isFullPage ? "overflow-x-clip" : "overflow-hidden")}>
                <div className={cn(isFullPage ? "" : "overflow-auto", maxHeightClass)}>
                    <table className="w-full text-left border-collapse">
                        <thead>
                            <tr
                                className="bg-slate-50/95 dark:bg-slate-800/95 backdrop-blur-sm border-b border-slate-100 dark:border-slate-800 sticky z-10"
                                style={{ top: isFullPage ? stickyOffset : 0 }}
                            >
                                {visibleColumns.map((col) => (
                                    <th
                                        className={cn(
                                            "px-8 py-5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em]",
                                            col.onHeaderClick && "cursor-pointer hover:text-primary-600 dark:hover:text-primary-400 transition-colors group",
                                            col.headerClassName
                                        )}
                                        key={col.key}
                                        onClick={col.onHeaderClick}
                                    >
                                        {col.header}
                                    </th>
                                ))}
                                {!!showChevron && <th className="px-4 py-5 w-10" />}
                            </tr>
                        </thead>

                        <NavigableListTableContent
                            colSpan={colSpan}
                            emptyMessage={emptyMessage}
                            fetchNextPage={fetchNextPage}
                            isFetchingNextPage={isFetchingNextPage}
                            isLoading={isLoading}
                            itemKey={itemKey}
                            items={items}
                            listNav={listNav}
                            loadMoreRef={loadMoreRef}
                            onSelect={onSelect}
                            rowIdPrefix={rowIdPrefix}
                            selectedKey={selectedKey}
                            showActiveBar={showActiveBar}
                            showChevron={showChevron}
                            skeletonRows={skeletonRows}
                            t={t}
                            testIdPrefix={testIdPrefix}
                            visibleColumns={visibleColumns}
                        />
                    </table>
                </div>
            </div>
        </div>
    );
}
