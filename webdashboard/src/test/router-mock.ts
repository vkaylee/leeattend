/**
 * Shared reactive TanStack Router mock for tests.
 *
 * Usage (in test file):
 *   import { type RouterMockState } from "../../test/router-mock";
 *
 *   const { state, resetSearch } = vi.hoisted(() => {
 *     const DEFAULT = { tab: "recorded" };
 *     return {
 *       state: { _search: { ...DEFAULT }, _listeners: new Set(), DEFAULT } as RouterMockState,
 *       resetSearch: () => { state._search = { ...state.DEFAULT }; },
 *     };
 *   });
 *
 *   vi.mock("@tanstack/react-router", async () => {
 *     const { buildMockImpl } = await import("../../test/router-mock");
 *     return buildMockImpl(state);
 *   });
 *
 *   // In beforeEach:
 *   resetSearch();
 *
 * @see ADR-006 for the URL-first state model
 */

import { createElement, useEffect, useState, type ReactNode } from "react";

/** Mutable state shared between the mock implementation and test code. */
export interface RouterMockState {
  _listeners: Set<() => void>;
  _search: Record<string, unknown>;
  DEFAULT: Record<string, unknown>;
}

/**
 * Creates a reactive TanStack Router mock from pre-created state.
 *
 * Must be called from inside vi.mock's async factory because it depends on React.
 * The state object must be created inside vi.hoisted() so it exists before
 * vi.mock's factory runs (Vitest hoists vi.mock before module-level code).
 */
export function buildMockImpl(state: RouterMockState): Record<string, unknown> {
  function notify(): void {
    state._listeners.forEach((fn) => {
      fn();
    });
  }

  function useSearchState(): Record<string, unknown> {
    const [search, setSearch] = useState(state._search);
    useEffect(() => {
      const listener = () => {
        setSearch({ ...state._search });
      };
      state._listeners.add(listener);
      return () => {
        state._listeners.delete(listener);
      };
    }, []);
    return search;
  }

  return {
    useNavigate: () =>
      (opts: {
        to?: string;
        search?:
          | ((prev: Record<string, unknown>) => Record<string, unknown>)
          | Record<string, unknown>;
        replace?: boolean;
      }) => {
        if (opts.search) {
          if (typeof opts.search === "function") {
            state._search = opts.search(state._search);
          } else {
            state._search = opts.search;
          }
          notify();
        }
      },

    useLocation: () => ({ pathname: "/", search: "", hash: "" }),

    useParams: () => ({}),

    useBlocker: () => ({
      proceed: () => undefined,
      reset: () => undefined,
      status: "idle" as const,
    }),

    useSearch: useSearchState,

    createFileRoute: () => () => ({
      useSearch: useSearchState,
    }),

    Link: ({
      to,
      search,
      children,
      ...rest
    }: {
      to: string;
      search?: Record<string, unknown>;
      children: ReactNode;
      [key: string]: unknown;
    }) => {
      let href = to;
      if (search && typeof search === "object") {
        const params = new URLSearchParams();
        Object.entries(search).forEach(([key, value]) => {
          const stringValue =
            value && typeof value === "object"
              ? JSON.stringify(value)
              : String(value);
          params.append(key, stringValue);
        });
        const qs = params.toString();
        if (qs) href += `?${qs}`;
      }
      return createElement("a", { href, ...rest }, children);
    },
  };
}

/**
 * Convenience helper: creates a "resetSearch" function for the given state.
 * Call this from inside vi.hoisted() alongside the state creation.
 *
 * Usage inside vi.hoisted:
 *   const initial = { tab: "requests" };
 *   const state = { _search: { ...initial }, _listeners: new Set(), DEFAULT: { ...initial } };
 *   return { state, resetSearch: makeResetSearch(state) };
 */
export function makeResetSearch(state: RouterMockState): () => void {
  return () => {
    state._search = { ...state.DEFAULT };
  };
}
