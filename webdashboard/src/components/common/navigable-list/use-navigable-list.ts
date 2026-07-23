import { useEffect, useMemo, useRef, useState } from "react";

import { useInfiniteScroll } from "@/hooks/use-infinite-scroll";
import { useKeyboardListNav } from "@/hooks/use-keyboard-list-nav";
import { useStickyHeader } from "@/hooks/use-sticky-header";

import type { NavigableListColumn } from "./types";

export function useNavigableListSticky({
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

export function useNavigableListInitialization<T>({
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
  const mobileLoadMoreRef = useInfiniteScroll<HTMLDivElement>({
    hasNextPage: hasNextPage ?? false,
    isFetchingNextPage: isFetchingNextPage ?? false,
    fetchNextPage: fetchNextPage ?? (() => { /* empty */ }),
  });

  return { listNav, loadMoreRef, mobileLoadMoreRef };
}

export function useNavigableListCalculations<T>({
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
