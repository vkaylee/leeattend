import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";

import { 
  fetchLocations, 
  createLocation, 
  updateLocation, 
  deleteLocation,
  CreateWorkLocationRequest,
  UpdateWorkLocationRequest
} from "@/lib/api/locations";

export const locationKeys = {
  all: () => ["locations"],
};

export function useLocations() {
  return useQuery({
    queryKey: locationKeys.all(),
    queryFn: fetchLocations,
    initialData: [],
    retry: false,
    meta: { suppressErrorToast: [403] },
  });
}

export function useCreateLocation() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (data: CreateWorkLocationRequest) => createLocation(data),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: locationKeys.all() });
    },
  });
}

export function useUpdateLocation() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ id, data }: { id: string; data: UpdateWorkLocationRequest }) => updateLocation(id, data),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: locationKeys.all() });
    },
  });
}

export function useDeleteLocation() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => deleteLocation(id),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: locationKeys.all() });
    },
  });
}
