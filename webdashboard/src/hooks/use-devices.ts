import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";

import {
  fetchDevices,
  createDevice,
  updateDevice,
  deleteDevice,
  type UpdateDeviceRequest,
} from "@/lib/api";

export const devicesKeys = {
  all: ["devices"] as const,
};

export function useDevices() {
  const queryClient = useQueryClient();

  const devicesQuery = useQuery({
    queryKey: devicesKeys.all,
    queryFn: fetchDevices,
  });

  const createMutation = useMutation({
    mutationFn: createDevice,
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: devicesKeys.all });
    },
  });

  const updateMutation = useMutation({
    mutationFn: ({ id, data }: { id: string; data: UpdateDeviceRequest }) =>
      updateDevice(id, data),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: devicesKeys.all });
    },
  });

  const deleteMutation = useMutation({
    mutationFn: deleteDevice,
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: devicesKeys.all });
    },
  });

  return {
    devices: devicesQuery.data ?? [],
    isLoading: devicesQuery.isLoading,
    error: devicesQuery.error,
    refetch: devicesQuery.refetch,

    createDevice: createMutation.mutateAsync,
    updateDevice: (id: string, data: UpdateDeviceRequest) =>
      updateMutation.mutateAsync({ id, data }),
    deleteDevice: deleteMutation.mutateAsync,

    isCreating: createMutation.isPending,
    isUpdating: updateMutation.isPending,
    isDeleting: deleteMutation.isPending,
  };
}

