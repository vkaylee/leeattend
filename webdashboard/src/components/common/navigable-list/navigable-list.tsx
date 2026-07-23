import React from "react";
import { useTranslation } from "react-i18next";

import { cn } from "@/lib/utils";

import type { NavigableListProps, SearchProps } from "./types";

import { NavigableListHeaderWrapper, SearchBarSlot } from "./navigable-list-search";
import { NavigableListTable } from "./navigable-list-table";
import { useNavigableListCalculations, useNavigableListInitialization } from "./use-navigable-list";

/**
 * NavigableList — generic keyboard-navigable list with search and pagination.
 * Complexity (11) is 1 over threshold; remaining branches are JSX ternaries for
 * smart-header and full-page modes. Readability as a flat render is better than
 * extracting yet another wrapper.
 */
// eslint-disable-next-line complexity
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
  mobileBreakpoint = "lg",
  headerSlot,
  onKeyDownRef,
  renderMobileCard,
  smartHeader = false,
  testIdPrefix,
}: NavigableListProps<T>) {
  const { t } = useTranslation();
  const { listNav, loadMoreRef, mobileLoadMoreRef } = useNavigableListInitialization({
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

  const searchProps: SearchProps = { onSearchChange, searchTestId, searchInputRef, listNav, searchPlaceholder, searchValue };

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
        <SearchBarSlot headerSlot={headerSlot} searchProps={searchProps} />
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
        mobileBreakpoint={mobileBreakpoint}
        mobileLoadMoreRef={mobileLoadMoreRef}
        onSelect={onSelect}
        renderMobileCard={renderMobileCard}
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
