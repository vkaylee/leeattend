import type { TFunction } from "i18next";

import { ChevronRight, Search, Loader2 } from "lucide-react";
import React, { useEffect, useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";

import { useInfiniteScroll } from "@/hooks/use-infinite-scroll";
import { useKeyboardListNav } from "@/hooks/use-keyboard-list-nav";
import { useStickyHeader } from "@/hooks/use-sticky-header";
import { cn } from "@/lib/utils";

export interface NavigableListColumn<T> {
  cellClassName?: string;
  header: React.ReactNode;
  headerClassName?: string;
  hidden?: boolean;
  key: string;
  onHeaderClick?: () => void;
  render: (item: T) => React.ReactNode;
}

interface NavigableListProps<T> {
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
  onKeyDownRef?: React.RefObject<((e: React.KeyboardEvent<HTMLInputElement>) => void) | null | undefined>;
  onSearchChange?: (val: string) => void;
  onSelect: (item: T) => void;
  resetDeps?: unknown[];
  rowIdPrefix?: string;
  searchInputRef?: React.RefObject<HTMLInputElement | null>;
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

function NavigableListSkeleton({
  skeletonRows,
  colSpan,
}: {
  colSpan: number;
  skeletonRows: number;
}) {
  return (
    <>
      {Array.from({ length: skeletonRows }).map((_, i) => (
        <tr className="animate-pulse border-b border-slate-50 dark:border-slate-800" key={i}>
          <td className="px-8 py-6" colSpan={colSpan}>
            <div className="h-4 bg-slate-100 dark:bg-slate-800 rounded-lg w-2/3" />
          </td>
        </tr>
      ))}
    </>
  );
}

function InfiniteScrollSentinel({
  isFetchingNextPage,
  sentinelRef,
  colSpan,
}: {
  colSpan: number;
  isFetchingNextPage: boolean;
  sentinelRef: React.Ref<HTMLTableRowElement>;
}) {
  return (
    <tr ref={sentinelRef}>
      <td className="px-8 py-4 text-center" colSpan={colSpan}>
        {isFetchingNextPage ? (
          <div className="flex items-center justify-center gap-2 text-slate-400">
            <Loader2 className="animate-spin" size={16} />
            <span className="text-[10px] font-bold uppercase tracking-widest">Loading...</span>
          </div>
        ) : null}
      </td>
    </tr>
  );
}

interface TableContentProps<T> {
  colSpan: number;
  emptyMessage?: string;
  fetchNextPage?: () => void;
  isFetchingNextPage?: boolean;
  isLoading: boolean;
  itemKey: (item: T) => string;
  items: T[];
  listNav: {
    focusedIndex: number;
    setFocusedIndex: (index: number) => void;
  };
  loadMoreRef: React.Ref<HTMLTableRowElement>;
  onSelect: (item: T) => void;
  rowIdPrefix: string;
  selectedKey?: string | null;
  showActiveBar: boolean;
  showChevron: boolean;
  skeletonRows: number;
  t: TFunction;
  testIdPrefix?: string;
  visibleColumns: NavigableListColumn<T>[];
}

function getRowClass(isFocused: boolean, isSelected: boolean): string {
  if (isFocused) return "bg-slate-50/80 dark:bg-slate-800/50 border-l-primary-500 shadow-inner";
  if (isSelected) return "bg-primary-50/30 dark:bg-primary-900/10 border-l-primary-400/50";
  return "hover:bg-slate-50/50 dark:hover:bg-slate-800/30";
}

function getChevronClass(isFocused: boolean): string {
  return isFocused ? "text-primary-500 translate-x-1" : "text-slate-300 dark:text-slate-700 opacity-0 group-hover:opacity-100";
}

function renderTableRow<T>({
  item,
  index,
  itemKey,
  listNav,
  onSelect,
  rowIdPrefix,
  selectedKey,
  showActiveBar,
  showChevron,
  testIdPrefix,
  visibleColumns,
}: {
  index: number;
  item: T;
  itemKey: (item: T) => string;
  listNav: { focusedIndex: number; setFocusedIndex: (index: number) => void };
  onSelect: (item: T) => void;
  rowIdPrefix: string;
  selectedKey?: string | null;
  showActiveBar: boolean;
  showChevron: boolean;
  testIdPrefix?: string;
  visibleColumns: NavigableListColumn<T>[];
}) {
  const key = itemKey(item);
  const isFocused = listNav.focusedIndex === index;
  const isSelected = selectedKey === key;
  const rowClass = cn("group cursor-pointer transition-all relative outline-none border-l-4 border-l-transparent", getRowClass(isFocused, isSelected));
  return (
    <tr
      className={rowClass}
      id={rowIdPrefix ? `${rowIdPrefix}-${index}` : undefined}
      key={key}
      {...(testIdPrefix ? { "data-testid": `${testIdPrefix}-${index}` } : {})}
      onClick={() => { listNav.setFocusedIndex(index); onSelect(item); }}
      onMouseEnter={() => { listNav.setFocusedIndex(index); }}
      tabIndex={-1}
    >
      {visibleColumns.map((col) => (
        <td className="px-8 py-5" key={col.key}>{col.render(item)}</td>
      ))}
      {showChevron ? (
        <td className="px-4 py-5 w-10 text-right">
          <ChevronRight className={cn("w-4 h-4 transition-all", getChevronClass(isFocused))} />
        </td>
      ) : null}
      {showActiveBar && isSelected ? <div className="absolute left-0 top-0 bottom-0 w-1 bg-primary-500" /> : null}
    </tr>
  );
}

function NavigableListTableContent<T>({
  isLoading,
  items,
  skeletonRows,
  colSpan,
  visibleColumns,
  showChevron,
  showActiveBar,
  selectedKey,
  onSelect,
  rowIdPrefix,
  itemKey,
  listNav,
  emptyMessage,
  t,
  fetchNextPage,
  isFetchingNextPage,
  loadMoreRef,
  testIdPrefix,
}: TableContentProps<T>) {
  if (isLoading && items.length === 0) {
    return (
      <tbody>
        <NavigableListSkeleton colSpan={colSpan} skeletonRows={skeletonRows} />
      </tbody>
    );
  }

  if (items.length === 0) {
    return (
      <tbody>
        <tr>
          <td className="px-8 py-20 text-center text-slate-400 font-bold uppercase tracking-widest text-[10px]" colSpan={colSpan}>
            {emptyMessage ?? t("common.no_data")}
          </td>
        </tr>
      </tbody>
    );
  }

  return (
    <tbody className="divide-y divide-slate-50 dark:divide-slate-800">
      {items.map((item, index) => renderTableRow({ item, index, itemKey, listNav, onSelect, rowIdPrefix, selectedKey, showActiveBar, showChevron, testIdPrefix, visibleColumns }))}
      {fetchNextPage ? (
        <InfiniteScrollSentinel
          colSpan={colSpan}
          isFetchingNextPage={isFetchingNextPage ?? false}
          sentinelRef={loadMoreRef}
        />
      ) : null}
    </tbody>
  );
}

function NavigableListTable<T>({
  colSpan,
  emptyMessage,
  fetchNextPage,
  isFetchingNextPage,
  isFullPage,
  isLoading,
  itemKey,
  items,
  listNav,
  loadMoreRef,
  maxHeightClass,
  onSelect,
  rowIdPrefix,
  selectedKey,
  showActiveBar,
  showChevron,
  skeletonRows,
  stickyOffset,
  t,
  testIdPrefix,
  visibleColumns,
}: {
  colSpan: number;
  emptyMessage?: string;
  fetchNextPage?: () => void;
  isFetchingNextPage?: boolean;
  isFullPage: boolean;
  isLoading: boolean;
  itemKey: (item: T) => string;
  items: T[];
  listNav: {
    focusedIndex: number;
    onKeyDown: (e: React.KeyboardEvent<HTMLInputElement>) => void;
    setFocusedIndex: (index: number) => void;
  };
  loadMoreRef: React.Ref<HTMLTableRowElement>;
  maxHeightClass: string;
  onSelect: (item: T) => void;
  rowIdPrefix: string;
  selectedKey?: string | null;
  showActiveBar: boolean;
  showChevron: boolean;
  skeletonRows: number;
  stickyOffset: number;
  t: TFunction;
  testIdPrefix?: string;
  visibleColumns: NavigableListColumn<T>[];
}) {
  return (
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
              {showChevron ? <th className="px-4 py-5 w-10" /> : null}
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
  );
}

function useNavigableListSticky({
  isFullPage,
  smartHeaderEnabled,
}: {
  isFullPage: boolean;
  smartHeaderEnabled: boolean;
}) {
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

  return {
    stickySearchRef,
    smartHeaderRef,
    smartHeaderVisible,
    stickyOffset,
  };
}

function useNavigableListInitialization<T>({
  items,
  onSelect,
  rowIdPrefix,
  resetDeps,
  onKeyDownRef,
  hasNextPage,
  isFetchingNextPage,
  fetchNextPage,
}: {
  fetchNextPage?: () => void;
  hasNextPage?: boolean;
  isFetchingNextPage?: boolean;
  items: T[];
  onKeyDownRef?: React.RefObject<((e: React.KeyboardEvent<HTMLInputElement>) => void) | null | undefined>;
  onSelect: (item: T) => void;
  resetDeps: unknown[];
  rowIdPrefix: string;
}) {
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

  return { listNav, loadMoreRef };
}

function useNavigableListCalculations<T>({
  columns,
  maxHeightClass,
  smartHeader,
  showChevron,
}: {
  columns: NavigableListColumn<T>[];
  maxHeightClass: string;
  showChevron: boolean;
  smartHeader: boolean;
}) {
  const visibleColumns = useMemo(() => columns.filter((col) => !col.hidden), [columns]);
  const isFullPage = maxHeightClass === "full";
  const smartHeaderEnabled = smartHeader && isFullPage;

  const {
    stickySearchRef,
    smartHeaderRef,
    smartHeaderVisible,
    stickyOffset,
  } = useNavigableListSticky({ isFullPage, smartHeaderEnabled });

  const colSpan = visibleColumns.length + (showChevron ? 1 : 0);

  return {
    visibleColumns,
    isFullPage,
    smartHeaderEnabled,
    stickySearchRef,
    smartHeaderRef,
    smartHeaderVisible,
    stickyOffset,
    colSpan,
  };
}

function NavigableListSearchContent({
  searchValue,
  onSearchChange,
  searchPlaceholder,
  searchInputRef,
  listNav,
  searchTestId,
}: {
  listNav: { onKeyDown: (e: React.KeyboardEvent<HTMLInputElement>) => void };
  onSearchChange?: (val: string) => void;
  searchInputRef?: React.RefObject<HTMLInputElement | null>;
  searchPlaceholder?: string;
  searchTestId?: string;
  searchValue?: string;
}) {
  if (searchValue === undefined) return null;

  return (
    <div className="relative group">
      <Search className="absolute left-6 top-1/2 -translate-y-1/2 text-slate-300 dark:text-slate-600 group-focus-within:text-primary-500 transition-colors" size={18} />
      <input
        className="w-full bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-[2rem] pl-16 pr-8 py-5 text-sm font-bold shadow-sm focus:ring-4 focus:ring-primary-500/10 focus:border-primary-200 dark:focus:border-primary-800 outline-none transition-all placeholder:text-slate-300 dark:placeholder:text-slate-700"
        onChange={(e) => onSearchChange?.(e.target.value)}
        onFocus={(e) => { e.target.select(); }}
        onKeyDown={listNav.onKeyDown}
        placeholder={searchPlaceholder}
        ref={searchInputRef}
        type="text"
        value={searchValue}
        {...(searchTestId ? { "data-testid": searchTestId } : {})}
      />
    </div>
  );
}

function NavigableListHeaderWrapper({
  smartHeaderEnabled,
  smartHeaderRef,
  smartHeaderVisible,
  stickySearchRef,
  headerSlot,
  isFullPage,
  searchProps,
}: {
  headerSlot?: React.ReactNode;
  isFullPage: boolean;
  searchProps: {
    listNav: { onKeyDown: (e: React.KeyboardEvent<HTMLInputElement>) => void };
    onSearchChange?: (val: string) => void;
    searchInputRef?: React.RefObject<HTMLInputElement | null>;
    searchPlaceholder?: string;
    searchTestId?: string;
    searchValue?: string;
  };
  smartHeaderEnabled: boolean;
  smartHeaderRef: React.Ref<HTMLDivElement>;
  smartHeaderVisible: boolean;
  stickySearchRef: React.Ref<HTMLDivElement>;
}) {
  return (
    <>
      {smartHeaderEnabled ? (
        <div
          className={cn(
            "fixed top-0 left-0 right-0 z-[60] bg-white/80 dark:bg-slate-900/80 backdrop-blur-md border-b border-slate-100 dark:border-slate-800 transition-all duration-300",
            smartHeaderVisible ? "translate-y-0 opacity-100" : "-translate-y-full opacity-0"
          )}
          ref={smartHeaderRef}
        >
          <div className="max-w-7xl mx-auto px-6 py-4 flex items-center justify-between gap-6">
            <div className="flex-1 max-w-2xl">
              <NavigableListSearchContent {...searchProps} />
            </div>
            {headerSlot}
          </div>
        </div>
      ) : null}

      {isFullPage && !smartHeaderEnabled ? (
        <div className="sticky top-0 z-40 bg-slate-50/50 dark:bg-slate-900/50 backdrop-blur-sm pb-6 -mx-2 px-2" ref={stickySearchRef}>
          <div className="flex flex-col md:flex-row md:items-center justify-between gap-6">
            <div className="flex-1 max-w-2xl">
              <NavigableListSearchContent {...searchProps} />
            </div>
            {headerSlot}
          </div>
        </div>
      ) : null}
    </>
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
  testIdPrefix,
}: NavigableListProps<T>) {
  const { t } = useTranslation();
  const { listNav, loadMoreRef } = useNavigableListInitialization({
    fetchNextPage,
    hasNextPage,
    isFetchingNextPage,
    items,
    onKeyDownRef,
    onSelect,
    resetDeps,
    rowIdPrefix,
  });

  const {
    visibleColumns,
    isFullPage,
    smartHeaderEnabled,
    stickySearchRef,
    smartHeaderRef,
    smartHeaderVisible,
    stickyOffset,
    colSpan,
  } = useNavigableListCalculations({ columns, maxHeightClass, smartHeader, showChevron });

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

      {!isFullPage ? (
        <div className="flex flex-col md:flex-row md:items-center justify-between gap-6">
          <div className="flex-1 max-w-2xl">
            <NavigableListSearchContent {...searchProps} />
          </div>
          {headerSlot}
        </div>
      ) : null}

      <NavigableListTable
        colSpan={colSpan}
        emptyMessage={emptyMessage}
        fetchNextPage={fetchNextPage}
        isFetchingNextPage={isFetchingNextPage}
        isFullPage={isFullPage}
        isLoading={isLoading}
        itemKey={itemKey}
        items={items}
        listNav={listNav}
        loadMoreRef={loadMoreRef}
        maxHeightClass={maxHeightClass}
        onSelect={onSelect}
        rowIdPrefix={rowIdPrefix}
        selectedKey={selectedKey}
        showActiveBar={showActiveBar}
        showChevron={showChevron}
        skeletonRows={skeletonRows}
        stickyOffset={stickyOffset}
        t={t}
        testIdPrefix={testIdPrefix}
        visibleColumns={visibleColumns}
      />
    </div>
  );
}
