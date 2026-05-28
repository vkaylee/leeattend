import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { 
  listLocations, 
  createLocation, 
  updateLocation, 
  deleteLocation,
  getListLocationsQueryKey
} from "@/lib/api-generated/crate-handlers-locations/crate-handlers-locations";
import type { CreateWorkLocationRequest, UpdateWorkLocationRequest } from "@/lib/api-generated/models";

export const locationKeys = {
  all: (wsId: string) => getListLocationsQueryKey(wsId),
};

export function useLocations(wsId: string) {
  return useQuery({
    queryKey: locationKeys.all(wsId),
    queryFn: async () => {
      const res = await listLocations(wsId);
      return res.data;
    },
    enabled: !!wsId,
  });
}

export function useCreateLocation(wsId: string) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (data: CreateWorkLocationRequest) => createLocation(wsId, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: locationKeys.all(wsId) });
    },
  });
}

export function useUpdateLocation(wsId: string) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ id, data }: { id: string; data: UpdateWorkLocationRequest }) => updateLocation(wsId, id, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: locationKeys.all(wsId) });
    },
  });
}

export function useDeleteLocation(wsId: string) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => deleteLocation(wsId, id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: locationKeys.all(wsId) });
    },
  });
}
