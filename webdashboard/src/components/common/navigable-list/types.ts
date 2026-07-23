
import type { ReactNode } from "react";

export interface NavigableListColumn<T> {
  cellClassName?: string;
  header: ReactNode;
  headerClassName?: string;
  hidden?: boolean;
  key: string;
  onHeaderClick?: () => void;
  render: (item: T) => ReactNode;
}

export interface NavigableListProps<T> {
  className?: string;
  columns: NavigableListColumn<T>[];
  emptyMessage?: string;
  fetchNextPage?: () => void;
  hasNextPage?: boolean;
  headerSlot?: ReactNode;
  isFetchingNextPage?: boolean;
  isLoading?: boolean;
  itemKey: (item: T) => string;
  items: T[];
  maxHeightClass?: string;
  mobileBreakpoint?: "md" | "lg" | "xl";
  onKeyDownRef?: React.RefObject<((e: React.KeyboardEvent<HTMLInputElement>) => void) | null | undefined>;
  onSearchChange?: (val: string) => void;
  onSelect: (item: T) => void;
  renderMobileCard?: (item: T, index: number) => ReactNode;
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

export interface ListNav {
  focusedIndex: number;
  onKeyDown: (e: React.KeyboardEvent<HTMLInputElement>) => void;
  setFocusedIndex: (index: number) => void;
}

export interface ListNavSimple {
  focusedIndex: number;
  setFocusedIndex: (index: number) => void;
}

export interface SearchProps {
  listNav: { onKeyDown: (e: React.KeyboardEvent<HTMLInputElement>) => void };
  onSearchChange?: (val: string) => void;
  searchInputRef?: React.RefObject<HTMLInputElement | null>;
  searchPlaceholder?: string;
  searchTestId?: string;
  searchValue?: string;
}

export const MOBILE_VISIBILITY = {
  md: { cards: "md:hidden", table: "hidden md:block" },
  lg: { cards: "lg:hidden", table: "hidden lg:block" },
  xl: { cards: "xl:hidden", table: "hidden xl:block" },
} as const;
