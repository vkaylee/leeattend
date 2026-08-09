import {
  useInfiniteQuery,
  useMutation,
  useQuery,
  useQueryClient,
  keepPreviousData,
  type InfiniteData,
  type QueryFunctionContext,
  type QueryKey,
  type UseInfiniteQueryResult,
  type UseMutationResult,
  type UseQueryResult,
} from "@tanstack/react-query";

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
  /** Optional validated workspace ID for hooks that already receive the active workspace. */
  workspaceId?: string | null;
  /** Additional gating ANDed with the workspace guard. */
  enabled?: boolean;
  placeholderData?: "keepPreviousData";
  queryFn: (wsId: string) => Promise<TData>;
  queryKey: QueryKey;
  refetchInterval?: number;
  refetchOnWindowFocus?: boolean;
  /**
   * When true and the workspace is present, the query immediately starts
   * loading (enabled=true). Useful for queries that should fire proactively
   * as soon as a workspace is known, like dashboard data.
   * Default: true (query fires when workspace is available).
   */
  staleTime?: number;
}

export function useWorkspaceQuery<TData>(
  options: WorkspaceQueryOptions<TData>,
): UseQueryResult<TData> {
  const { workspace } = useWorkspace();
  const workspaceId = workspace?.id ?? options.workspaceId;

  return useQuery<TData>({
    queryKey: options.queryKey,
    queryFn: () => {
      if (!workspaceId) throw new Error("No workspace ID");
      return options.queryFn(workspaceId);
    },
    enabled: !!workspaceId && (options.enabled ?? true),
    staleTime: options.staleTime,
    placeholderData: options.placeholderData === "keepPreviousData" ? keepPreviousData : undefined,
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
  /** Optional validated workspace ID for hooks that already receive the active workspace. */
  workspaceId?: string | null;
  keys: QueryKey[];
  mutationFn: (wsId: string, vars: TVars) => Promise<TData>;
  onError?: (error: Error, vars: TVars) => void;
  onSuccess?: (data: TData, vars: TVars) => void;
}

interface WorkspaceInfiniteQueryOptions<TData, TPageParam> {
  workspaceId?: string | null;
  enabled?: boolean;
  getNextPageParam: (lastPage: TData, allPages: TData[]) => TPageParam | undefined;
  initialPageParam: TPageParam;
  placeholderData?: "keepPreviousData";
  queryFn: (wsId: string, pageParam: TPageParam) => Promise<TData>;
  queryKey: QueryKey;
}

export function useWorkspaceInfiniteQuery<TData, TPageParam>(
  options: WorkspaceInfiniteQueryOptions<TData, TPageParam>,
): UseInfiniteQueryResult<InfiniteData<TData, TPageParam>> {
  const { workspace } = useWorkspace();
  const workspaceId = workspace?.id ?? options.workspaceId;

  return useInfiniteQuery<TData, Error, InfiniteData<TData, TPageParam>, QueryKey, TPageParam>({
    queryKey: options.queryKey,
    queryFn: ({ pageParam }: QueryFunctionContext<QueryKey, TPageParam>) => {
      if (!workspaceId) throw new Error("No workspace ID");
      return options.queryFn(workspaceId, pageParam as TPageParam);
    },
    enabled: !!workspaceId && (options.enabled ?? true),
    initialPageParam: options.initialPageParam,
    getNextPageParam: options.getNextPageParam,
    placeholderData: options.placeholderData === "keepPreviousData" ? (previous) => previous : undefined,
  });
}

export function useWorkspaceMutation<TVars = void, TData = unknown>(
  options: WorkspaceMutationOptions<TVars, TData>,
): UseMutationResult<TData, Error, TVars> {
  const { workspace } = useWorkspace();
  const queryClient = useQueryClient();

  return useMutation<TData, Error, TVars>({
    mutationFn: (vars: TVars) => {
      const workspaceId = workspace?.id ?? options.workspaceId;
      if (!workspaceId) throw new Error("No workspace ID");
      return options.mutationFn(workspaceId, vars);
    },
    onError: (error, vars) => {
      options.onError?.(error, vars);
    },
    onSuccess: (data, vars) => {
      options.keys.forEach((key) => {
        void queryClient.invalidateQueries({ queryKey: key });
      });
      options.onSuccess?.(data, vars);
    },
  });
}
