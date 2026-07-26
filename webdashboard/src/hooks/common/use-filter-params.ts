import { useSearch } from "@tanstack/react-router";
import { useNavigate } from "@tanstack/react-router";
import { useCallback, useMemo } from "react";

// ── Types ────────────────────────────────────────────────────────────────────

export interface UseFilterParamsOptions<T> {
  /** Key prefix for URL params (e.g. "dept" → dept=id1&dept=id2) */
  keys: Record<keyof T, string>;
  /** Default values (used when URL param is absent) */
  defaults: T;
  /** Transform: URL string → state value */
  decode?: Partial<Record<keyof T, (val: string | null) => T[keyof T]>>;
  /** Transform: state value → URL string */
  encode?: Partial<Record<keyof T, (val: T[keyof T]) => string | null>>;
}

// ── Hook ─────────────────────────────────────────────────────────────────────

/**
 * Generic hook that syncs filter state with URL search params.
 *
 * Reads initial values from the URL on mount and provides a `setFilters`
 * setter that writes back to the URL (with replace, no history push).
 *
 * Usage:
 * ```
 * const { filters, setFilters } = useFilterParams({
 *   keys: { departmentId: "dept", employeeId: "emp" },
 *   defaults: { departmentId: "all", employeeId: "all" },
 *   decode: { departmentId: (v) => v ?? "all" },
 *   encode: { departmentId: (v) => (v === "all" ? null : v) },
 * });
 * ```
 */
export function useFilterParams<T extends Record<string, any>>(
  opts: UseFilterParamsOptions<T>,
) {
  const search = useSearch({ strict: false }) as Record<string, string>;
  const navigate = useNavigate();

  // Read from URL → state
  const filters = useMemo(() => {
    const result = { ...opts.defaults };
    for (const [stateKey, paramKey] of Object.entries(opts.keys)) {
      const raw = search[paramKey as string];
      const decode = opts.decode?.[stateKey as keyof T];
      result[stateKey as keyof T] = decode
        ? decode(raw ?? null)
        : ((raw ?? opts.defaults[stateKey as keyof T]) as T[keyof T]);
    }
    return result as T;
  }, [search, opts]);

  // Write state → URL
  const setFilters = useCallback(
    (updates: Partial<T>) => {
      const params = new URLSearchParams(window.location.search);
      for (const [stateKey, value] of Object.entries(updates)) {
        const paramKey = opts.keys[stateKey as keyof T];
        const encode = opts.encode?.[stateKey as keyof T];
        const encoded = encode
          ? encode(value as any)
          : String(value);
        if (
          encoded === null ||
          encoded === "" ||
          encoded === String(opts.defaults[stateKey as keyof T])
        ) {
          params.delete(paramKey as string);
        } else {
          params.set(paramKey as string, encoded);
        }
      }
      void navigate({
        search: Object.fromEntries(params) as any,
        replace: true,
      });
    },
    [navigate, opts],
  );

  return { filters, setFilters };
}
