import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";

import {
  fetchDevices,
  createDevice,
  updateDevice,
  deleteDevice,
  Device,
  DeviceInfo,
  fetchDeviceInfo,
} from "@/lib/api";

export const devicesKeys = {
  all: ["devices"] as const,
  info: (id: string) => ["devices", id, "info"] as const,
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
      queryClient.invalidateQueries({ queryKey: devicesKeys.all });
      queryClient.invalidateQueries({ queryKey: ["agents-status"] });
    },
  });

  const updateMutation = useMutation({
    mutationFn: ({ id, data }: { id: string; data: Partial<Device> }) =>
      updateDevice(id, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: devicesKeys.all });
      queryClient.invalidateQueries({ queryKey: ["agents-status"] });
    },
  });

  const deleteMutation = useMutation({
    mutationFn: deleteDevice,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: devicesKeys.all });
      queryClient.invalidateQueries({ queryKey: ["agents-status"] });
    },
  });

  return {
    devices: devicesQuery.data ?? [],
    isLoading: devicesQuery.isLoading,
    error: devicesQuery.error,
    refetch: devicesQuery.refetch,

    createDevice: createMutation.mutateAsync,
    updateDevice: (id: string, data: Partial<Device>) =>
      updateMutation.mutateAsync({ id, data }),
    deleteDevice: deleteMutation.mutateAsync,

    isCreating: createMutation.isPending,
    isUpdating: updateMutation.isPending,
    isDeleting: deleteMutation.isPending,
  };
}

export function useDeviceInfo(id: string, enabled: boolean = true) {
  return useQuery({
    queryKey: devicesKeys.info(id),
    queryFn: () => fetchDeviceInfo(id),
    enabled: !!id && enabled,
    staleTime: 60000 * 30, // Cache for 30 minutes
    refetchOnWindowFocus: false, // Don't refetch on window focus to avoid noise
  });
}
