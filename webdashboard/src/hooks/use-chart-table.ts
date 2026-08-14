import { useMemo, useState } from "react";

/**
 * Client-side pagination + search state shared by every dashboard chart's
 * accessible "table view" twin (the dataviz-required text alternative).
 */
export function useChartTable<T extends object>(
  rows: T[],
  options: {
    /** Keys whose stringified values participate in the text search. */
    searchableKeys?: (keyof T)[];
    /** Rows per page (default 10). */
    pageSize?: number;
  } = {},
) {
  const { searchableKeys = [], pageSize = 10 } = options;
  const [query, setQuery] = useState("");
  const [page, setPage] = useState(1);

  const filtered = useMemo(() => {
    const needle = query.trim().toLowerCase();
    if (!needle) return rows;
    return rows.filter((row) =>
      searchableKeys.some((key) => {
        const value = row[key];
        return value != null && String(value).toLowerCase().includes(needle);
      }),
    );
  }, [rows, query, searchableKeys]);

  const totalPages = Math.max(1, Math.ceil(filtered.length / pageSize));
  const safePage = Math.min(page, totalPages);

  const pageRows = useMemo(
    () => filtered.slice((safePage - 1) * pageSize, safePage * pageSize),
    [filtered, safePage, pageSize],
  );

  return {
    query,
    setQuery,
    page: safePage,
    setPage,
    filtered,
    pageRows,
    totalItems: filtered.length,
    totalPages,
    pageSize,
  };
}
