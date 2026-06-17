import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";

import {
  fetchStorageConfig,
  upsertStorageConfig,
  testStorageConfig,
  deleteStorageConfig,
  type CreateStorageConfigRequest,
} from "@/lib/api/storage-config";
import { useWorkspace } from "@/hooks/use-workspace";

export const storageConfigKeys = {
  all: ["workspace-storage"] as const,
  detail: (workspaceId: string) => [...storageConfigKeys.all, workspaceId, "detail"] as const,
};

export const useStorageConfig = () => {
  const { workspace } = useWorkspace();
  const workspaceId = workspace?.id;

  return useQuery({
    queryKey: storageConfigKeys.detail(workspaceId ?? ""),
    queryFn: () => {
      if (!workspaceId) throw new Error("No workspace ID");
      return fetchStorageConfig(workspaceId);
    },
    enabled: !!workspaceId,
  });
};

export const useUpsertStorageConfig = () => {
  const queryClient = useQueryClient();
  const { workspace } = useWorkspace();

  return useMutation({
    mutationFn: (payload: CreateStorageConfigRequest) => {
      const workspaceId = workspace?.id;
      if (!workspaceId) throw new Error("No workspace ID");
      return upsertStorageConfig(workspaceId, payload);
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: storageConfigKeys.all });
    },
  });
};

export const useTestStorageConfig = () => {
  const { workspace } = useWorkspace();

  return useMutation({
    mutationFn: (payload: CreateStorageConfigRequest) => {
      const workspaceId = workspace?.id;
      if (!workspaceId) throw new Error("No workspace ID");
      return testStorageConfig(workspaceId, payload);
    },
  });
};

export const useDeleteStorageConfig = () => {
  const queryClient = useQueryClient();
  const { workspace } = useWorkspace();

  return useMutation({
    mutationFn: () => {
      const workspaceId = workspace?.id;
      if (!workspaceId) throw new Error("No workspace ID");
      return deleteStorageConfig(workspaceId);
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: storageConfigKeys.all });
    },
  });
};
