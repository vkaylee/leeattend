/**
 * Lightweight shared pagination state hook.
 *
 * Replaces handrolled `useState` + `setPage` + `pageSize` patterns (~11 files)
 * with a single hook that returns page state, navigation helpers, and a
 * `reset()` that should be called when filters/search change so users always
 * land back on page 1.
 *
 * Usage:
 * ```
 * const { page, setPage, pageSize, setPageSize, reset } = usePagination();
 * // server call uses `page` and `pageSize`
 * // on search/filter change → reset()
 * ```
 */

import { useCallback, useState } from "react";

export interface UsePaginationOptions {
  initialPage?: number;
  initialPageSize?: number;
}

export interface UsePaginationReturn {
  page: number;
  setPage: (page: number) => void;
  pageSize: number;
  setPageSize: (size: number) => void;
  /** Reset to initial page (call when filters/search change). */
  reset: () => void;
}

export function usePagination(
  options: UsePaginationOptions = {},
): UsePaginationReturn {
  const { initialPage = 1, initialPageSize = 20 } = options;

  const [page, setPage] = useState(initialPage);
  const [pageSize, setPageSize] = useState(initialPageSize);

  const reset = useCallback(() => {
    setPage(initialPage);
  }, [initialPage]);

  return { page, setPage, pageSize, setPageSize, reset };
}
