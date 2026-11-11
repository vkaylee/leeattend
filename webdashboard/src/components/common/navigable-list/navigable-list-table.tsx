import type { TFunction } from "i18next";

import { ChevronRight } from "lucide-react";
import React from "react";

import { useHorizontalDragScroll } from "@/hooks/use-horizontal-drag-scroll";
import { cn } from "@/lib/utils";

import type { ListNavSimple, NavigableListColumn } from "./types";

import { NavigableListMobileCards } from "./navigable-list-mobile-cards";
import { InfiniteScrollSentinel, NavigableListSkeleton } from "./navigable-list-skeleton";
import { MOBILE_VISIBILITY } from "./types";

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
  listNav: ListNavSimple;
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
      {visibleColumns.map((col, colIdx) => (
        <td className={cn("px-8 py-5 relative", col.cellClassName)} key={col.key}>
          {showActiveBar && isSelected && colIdx === 0 ? (
            <div className="absolute left-0 top-0 bottom-0 w-1 bg-primary-500" />
          ) : null}
          {col.render(item)}
        </td>
      ))}
      {showChevron ? (
        <td className="px-4 py-5 w-10 text-right">
          <ChevronRight className={cn("w-4 h-4 transition-all", getChevronClass(isFocused))} />
        </td>
      ) : null}
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
  listNav: ListNavSimple;
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

export function NavigableListTable<T>({
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
  mobileBreakpoint,
  mobileLoadMoreRef,
  onSelect,
  renderMobileCard,
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
  listNav: ListNavSimple & { onKeyDown: (e: React.KeyboardEvent<HTMLInputElement>) => void };
  loadMoreRef: React.Ref<HTMLTableRowElement>;
  maxHeightClass: string;
  mobileBreakpoint: "md" | "lg" | "xl";
  mobileLoadMoreRef: React.Ref<HTMLDivElement>;
  onSelect: (item: T) => void;
  renderMobileCard?: (item: T, index: number) => React.ReactNode;
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
  const visibility = MOBILE_VISIBILITY[mobileBreakpoint];
  const {
    onClickCapture: handleHorizontalDragClick,
    setRef: setHorizontalDragScrollRef,
  } = useHorizontalDragScroll<HTMLDivElement>();
  const table = (
    <div className={cn("bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-[2.5rem] shadow-sm", isFullPage ? "overflow-x-clip" : "overflow-hidden")}>
      <div
        className={cn(isFullPage ? "overflow-x-auto overscroll-contain" : "overflow-auto overscroll-contain", maxHeightClass)}
        onClickCapture={handleHorizontalDragClick}
        ref={setHorizontalDragScrollRef}
      >
        <table className="w-full min-w-[1100px] text-left border-collapse">
          <thead>
            <tr
              className="bg-slate-50/95 dark:bg-slate-800/95 backdrop-blur-sm border-b border-slate-100 dark:border-slate-800 sticky z-10"
              style={{ top: isFullPage ? stickyOffset : 0 }}
            >
              {visibleColumns.map((col) => (
                <th
                  className={cn(
                    "px-8 py-4 text-xs font-semibold text-slate-700 dark:text-slate-300 uppercase tracking-wider",
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

  if (!renderMobileCard) return table;

  return (
    <>
      <div className={visibility.table}>{table}</div>
      <div className={visibility.cards}>
        <NavigableListMobileCards
          emptyMessage={emptyMessage}
          isFetchingNextPage={isFetchingNextPage ?? false}
          isLoading={isLoading}
          itemKey={itemKey}
          items={items}
          loadMoreRef={mobileLoadMoreRef}
          renderMobileCard={renderMobileCard}
          skeletonRows={skeletonRows}
          t={t}
        />
      </div>
    </>
  );
}
