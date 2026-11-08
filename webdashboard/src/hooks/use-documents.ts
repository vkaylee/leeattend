import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";

/* eslint-disable @typescript-eslint/no-floating-promises -- invalidateQueries fire-and-forget is the standard React Query pattern */
import {
  fetchFolders,
  createFolder,
  renameFolder,
  deleteFolder,
  distributeDocument,
  fetchPendingAcknowledgements,
  acknowledgeDocument,
  previewTemplate,
} from "@/lib/api/documents";

export const documentKeys = {
  all: ["documents"] as const,
  folders: () => [...documentKeys.all, "folders"] as const,
  distributions: () => [...documentKeys.all, "distributions"] as const,
  pendingAcks: () => [...documentKeys.all, "pending-acks"] as const,
};

export function useFolders() {
  return useQuery({
    queryKey: documentKeys.folders(),
    queryFn: () => fetchFolders(),
  });
}

export function useCreateFolder() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ name, parentFolderId }: { name: string; parentFolderId?: string }) =>
      createFolder(name, parentFolderId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: documentKeys.folders() });
    },
  });
}

export function useRenameFolder() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ id, name }: { id: string; name: string }) => renameFolder(id, name),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: documentKeys.folders() });
    },
  });
}

export function useDeleteFolder() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => deleteFolder(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: documentKeys.folders() });
    },
  });
}

export function useDistributeDocument() {
  return useMutation({
    mutationFn: ({ documentId, targetEmployeeIds }: { documentId: string; targetEmployeeIds: string[] }) =>
      distributeDocument(documentId, targetEmployeeIds),
  });
}

export function usePendingAcknowledgements() {
  return useQuery({
    queryKey: documentKeys.pendingAcks(),
    queryFn: () => fetchPendingAcknowledgements(),
  });
}

export function useAcknowledgeDocument() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => acknowledgeDocument(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: documentKeys.pendingAcks() });
    },
  });
}

export function usePreviewTemplate() {
  return useMutation({
    mutationFn: ({ templateContent, employeeId, manualVariables }: { templateContent: string; employeeId: string; manualVariables?: Record<string, string> }) =>
      previewTemplate(templateContent, employeeId, manualVariables),
  });
}
