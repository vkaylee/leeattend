import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
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
  folders: (wsId: string) => [...documentKeys.all, "folders", wsId] as const,
  distributions: (wsId: string) => [...documentKeys.all, "distributions", wsId] as const,
  pendingAcks: () => [...documentKeys.all, "pending-acks"] as const,
};

export function useFolders(wsId: string | undefined | null) {
  return useQuery({
    queryKey: documentKeys.folders(wsId!),
    queryFn: () => fetchFolders(wsId!),
    enabled: !!wsId,
  });
}

export function useCreateFolder(wsId: string | undefined | null) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ name, parentFolderId }: { name: string; parentFolderId?: string }) =>
      createFolder(wsId!, name, parentFolderId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: documentKeys.folders(wsId!) });
    },
  });
}

export function useRenameFolder(wsId: string | undefined | null) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ id, name }: { id: string; name: string }) =>
      renameFolder(wsId!, id, name),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: documentKeys.folders(wsId!) });
    },
  });
}

export function useDeleteFolder(wsId: string | undefined | null) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => deleteFolder(wsId!, id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: documentKeys.folders(wsId!) });
    },
  });
}

export function useDistributeDocument(wsId: string | undefined | null) {
  return useMutation({
    mutationFn: ({ documentId, targetEmployeeIds }: { documentId: string; targetEmployeeIds: string[] }) =>
      distributeDocument(wsId!, documentId, targetEmployeeIds),
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

export function usePreviewTemplate(wsId: string | undefined | null) {
  return useMutation({
    mutationFn: ({ templateContent, employeeId, manualVariables }: { templateContent: string; employeeId: string; manualVariables?: Record<string, string> }) =>
      previewTemplate(wsId!, templateContent, employeeId, manualVariables),
  });
}
