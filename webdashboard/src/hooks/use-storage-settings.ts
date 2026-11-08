import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";

import {
  fetchStorageConfig,
  upsertStorageConfig,
  testStorageConfig,
  deleteStorageConfig,
  type CreateStorageConfigRequest,
} from "@/lib/api/storage-config";

export const storageConfigKeys = {
  all: ["workspace-storage"] as const,
  detail: () => [...storageConfigKeys.all, "detail"] as const,
};

export const useStorageConfig = () => {
  return useQuery({
    queryKey: storageConfigKeys.detail(),
    queryFn: () => fetchStorageConfig(),
  });
};

export const useUpsertStorageConfig = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (payload: CreateStorageConfigRequest) => upsertStorageConfig(payload),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: storageConfigKeys.all });
    },
  });
};

export const useTestStorageConfig = () => {
  return useMutation({
    mutationFn: (payload: CreateStorageConfigRequest) => testStorageConfig(payload),
  });
};

export const useDeleteStorageConfig = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: () => deleteStorageConfig(),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: storageConfigKeys.all });
    },
  });
};
