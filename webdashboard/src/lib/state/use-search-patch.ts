import { useNavigate } from "@tanstack/react-router";
import { useCallback } from "react";

/**
 * Returns a `patch(nextPartial)` function that merges `nextPartial` into
 * the current route's search params via `navigate({ replace: true })`.
 *
 * Use inside components/hooks that already sit under a route with a
 * Zod `validateSearch` schema. The schema is the contract; this hook
 * is just the setter half of `Route.useSearch()`.
 *
 * @example
 * const search = Route.useSearch();
 * const patch = useSearchPatch<typeof search>();
 * patch({ tab: "recorded" });
 */
export function useSearchPatch<TSearch extends Record<string, unknown>>() {
  const navigate = useNavigate();
  return useCallback(
    (patch: Partial<TSearch>) => {
      void navigate({
        to: ".",
        search: (prev: unknown) => ({ ...(prev as TSearch), ...patch }),
        replace: true,
      });
    },
    [navigate],
  );
}
