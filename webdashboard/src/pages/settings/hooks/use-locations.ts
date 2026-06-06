import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";

import type { CreateWorkLocationRequest, UpdateWorkLocationRequest, WorkLocation } from "@/lib/api-generated/models";

import { 
  listLocations, 
  createLocation, 
  updateLocation, 
  deleteLocation,
  getListLocationsQueryKey
} from "@/lib/api-generated/crate-handlers-locations/crate-handlers-locations";

export const locationKeys = {
  all: (wsId: string) => getListLocationsQueryKey(wsId),
};

export function useLocations(wsId: string) {
  return useQuery({
    queryKey: locationKeys.all(wsId),
    queryFn: async () => {
      const res = await listLocations(wsId);
      // listLocations returns res.data which is typed as WorkLocation[] but at runtime is PaginatedLocations (having .data as WorkLocation[])
      const raw = res.data as unknown as { data: WorkLocation[] };
      return raw?.data ?? [];
    },
    enabled: !!wsId,
  });
}

export function useCreateLocation(wsId: string) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (data: CreateWorkLocationRequest) => createLocation(wsId, data),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: locationKeys.all(wsId) });
    },
  });
}

export function useUpdateLocation(wsId: string) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ id, data }: { id: string; data: UpdateWorkLocationRequest }) => updateLocation(wsId, id, data),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: locationKeys.all(wsId) });
    },
  });
}

export function useDeleteLocation(wsId: string) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => deleteLocation(wsId, id),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: locationKeys.all(wsId) });
    },
  });
}
