import { useQuery, useMutation, useQueryClient, type QueryKey, type UseQueryResult, type UseMutationResult } from "@tanstack/react-query";

import { useWorkspace } from "@/hooks/use-workspace";

// ── useWorkspaceQuery ─────────────────────────────────────────────────────
//
// Thin wrapper around useQuery that:
//  1. Guards with `enabled: !!workspace` so the query never fires without a
//     selected workspace.
//  2. Extracts `workspace.id` and passes it as the first argument to
//     `queryFn`, removing the manual "if (!wsId) throw" boilerplate from
//     every hook file.
//  3. Accepts an optional `enabled` extra sentinel ANDed with the workspace
//     guard (useful when the hook takes additional required params).
//
// Usage:
//   useWorkspaceQuery<MyType>({
//     queryKey: myKeys.list(wsId, ...),
//     queryFn: (wsId) => myApi.callMethod(wsId, ...),
//   });

interface WorkspaceQueryOptions<TData> {
  queryKey: QueryKey;
  queryFn: (wsId: string) => Promise<TData>;
  /** Additional gating ANDed with the workspace guard. */
  enabled?: boolean;
  /**
   * When true and the workspace is present, the query immediately starts
   * loading (enabled=true). Useful for queries that should fire proactively
   * as soon as a workspace is known, like dashboard data.
   * Default: true (query fires when workspace is available).
   */
  staleTime?: number;
  refetchInterval?: number;
  refetchOnWindowFocus?: boolean;
}

export function useWorkspaceQuery<TData>(
  options: WorkspaceQueryOptions<TData>,
): UseQueryResult<TData, Error> {
  const { workspace } = useWorkspace();

  return useQuery<TData, Error>({
    queryKey: options.queryKey,
    queryFn: () => {
      if (!workspace?.id) throw new Error("No workspace ID");
      return options.queryFn(workspace.id);
    },
    enabled: !!workspace?.id && (options.enabled ?? true),
    staleTime: options.staleTime,
    refetchInterval: options.refetchInterval,
    refetchOnWindowFocus: options.refetchOnWindowFocus,
  });
}

// ── useWorkspaceMutation ──────────────────────────────────────────────────
//
// Thin wrapper around useMutation that:
//  1. Guards mutationFn with workspace presence (throws if no workspace).
//  2. Extracts `workspace.id` and passes it as the first argument to
//     `mutationFn` — the generic TVars becomes the *second* arg.
//  3. Auto-invalidates the provided `keys` on success.
//  4. Supports a custom `onSuccess` callback that runs AFTER invalidation.
//
// Usage:
//   useWorkspaceMutation<PayloadType, ResultType>({
//     mutationFn: (wsId, payload) => myApi.create(wsId, payload),
//     keys: [myKeys.all],
//   });

interface WorkspaceMutationOptions<TVars, TData> {
  mutationFn: (wsId: string, vars: TVars) => Promise<TData>;
  keys: QueryKey[];
  onSuccess?: (data: TData, vars: TVars) => void;
}

export function useWorkspaceMutation<TVars = void, TData = unknown>(
  options: WorkspaceMutationOptions<TVars, TData>,
): UseMutationResult<TData, Error, TVars> {
  const { workspace } = useWorkspace();
  const queryClient = useQueryClient();

  return useMutation<TData, Error, TVars>({
    mutationFn: (vars: TVars) => {
      if (!workspace?.id) throw new Error("No workspace ID");
      return options.mutationFn(workspace.id, vars);
    },
    onSuccess: (data, vars) => {
      options.keys.forEach((key) => {
        void queryClient.invalidateQueries({ queryKey: key });
      });
      options.onSuccess?.(data, vars);
    },
  });
}
