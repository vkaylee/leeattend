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

interface OrganizationQueryOptions<TData> {
  enabled?: boolean;
  meta?: Record<string, unknown>;
  placeholderData?: "keepPreviousData";
  queryFn: () => Promise<TData>;
  queryKey: QueryKey;
  refetchInterval?: number;
  refetchOnWindowFocus?: boolean;
  staleTime?: number;
}

export function useOrganizationQuery<TData>(
  options: OrganizationQueryOptions<TData>,
): UseQueryResult<TData> {
  return useQuery<TData>({
    queryKey: options.queryKey,
    queryFn: options.queryFn,
    enabled: options.enabled ?? true,
    staleTime: options.staleTime,
    placeholderData: options.placeholderData === "keepPreviousData" ? keepPreviousData : undefined,
    refetchInterval: options.refetchInterval,
    refetchOnWindowFocus: options.refetchOnWindowFocus,
    meta: options.meta,
  });
}

// ── useOrganizationMutation ────────────────────────────────────────────────
//
// Thin wrapper around useMutation that:
//  1. Auto-invalidates the provided `keys` on success.
//  2. Supports a custom `onSuccess` callback that runs AFTER invalidation.
//
// Usage:
//   useOrganizationMutation<PayloadType, ResultType>({
//     mutationFn: (payload) => myApi.create(payload),
//     keys: [myKeys.all()],
//   });

interface OrganizationMutationOptions<TVars, TData> {
  keys: QueryKey[];
  mutationFn: (vars: TVars) => Promise<TData>;
  onError?: (error: Error, vars: TVars) => void;
  onSuccess?: (data: TData, vars: TVars) => void;
}

interface OrganizationInfiniteQueryOptions<TData, TPageParam> {
  enabled?: boolean;
  getNextPageParam: (lastPage: TData, allPages: TData[]) => TPageParam | undefined;
  initialPageParam: TPageParam;
  placeholderData?: "keepPreviousData";
  queryFn: (pageParam: TPageParam) => Promise<TData>;
  queryKey: QueryKey;
}

export function useOrganizationInfiniteQuery<TData, TPageParam>(
  options: OrganizationInfiniteQueryOptions<TData, TPageParam>,
): UseInfiniteQueryResult<InfiniteData<TData, TPageParam>> {
  return useInfiniteQuery<TData, Error, InfiniteData<TData, TPageParam>, QueryKey, TPageParam>({
    queryKey: options.queryKey,
    queryFn: ({ pageParam }: QueryFunctionContext<QueryKey, TPageParam>) =>
      options.queryFn(pageParam as TPageParam),
    enabled: options.enabled ?? true,
    initialPageParam: options.initialPageParam,
    getNextPageParam: options.getNextPageParam,
    placeholderData: options.placeholderData === "keepPreviousData" ? (previous) => previous : undefined,
  });
}

export function useOrganizationMutation<TVars = void, TData = unknown>(
  options: OrganizationMutationOptions<TVars, TData>,
): UseMutationResult<TData, Error, TVars> {
  const queryClient = useQueryClient();

  return useMutation<TData, Error, TVars>({
    mutationFn: options.mutationFn,
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
