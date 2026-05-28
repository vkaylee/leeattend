import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { fetchLocations, createLocation, updateLocation, deleteLocation, CreateWorkLocationRequest, UpdateWorkLocationRequest } from "@/lib/api/locations";

export const locationKeys = {
  all: (wsId: string) => ["locations", wsId] as const,
};

export function useLocations(wsId: string) {
  return useQuery({
    queryKey: locationKeys.all(wsId),
    queryFn: () => fetchLocations(wsId),
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
