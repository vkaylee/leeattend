import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { fetchDevices, createDevice, deleteDevice, Device } from "@/lib/api";

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
      queryClient.invalidateQueries({ queryKey: devicesKeys.all });
    },
  });

  const deleteMutation = useMutation({
    mutationFn: deleteDevice,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: devicesKeys.all });
    },
  });

  return {
    devices: devicesQuery.data ?? [],
    isLoading: devicesQuery.isLoading,
    error: devicesQuery.error,
    refetch: devicesQuery.refetch,

    createDevice: createMutation.mutateAsync,
    deleteDevice: deleteMutation.mutateAsync,

    isCreating: createMutation.isPending,
    isDeleting: deleteMutation.isPending,
  };
}
