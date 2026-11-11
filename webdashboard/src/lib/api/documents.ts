import { client } from "./client";

export interface DocumentFolder {
  id: string;
  name: string;
  parent_folder_id?: string | null;
}

export interface DistributeDocumentResponse {
  count: number;
  status: string;
}

export interface PendingAcknowledgement {
  created_at?: string;
  document_id: string;
  employee_id: string;
  id: string;
  status: string;
}

export interface AcknowledgeDocumentResponse {
  status: string;
}

export interface TemplatePreviewResponse {
  filled_content: string;
}

export async function fetchFolders(): Promise<DocumentFolder[]> {
  const { data } = await client.GET<DocumentFolder[]>("/api/v1/documents/folders");
  return data;
}

export async function createFolder(name: string, parentFolderId?: string): Promise<DocumentFolder> {
  const { data } = await client.POST<DocumentFolder>("/api/v1/documents/folders", {
    body: {
      name,
      parent_folder_id: parentFolderId ?? null,
    },
  });
  return data;
}

export async function renameFolder(id: string, name: string): Promise<DocumentFolder> {
  const { data } = await client.PUT<DocumentFolder>("/api/v1/documents/folders/{id}", {
    params: {
      path: { id },
    },
    body: {
      name,
    },
  });
  return data;
}

export async function deleteFolder(id: string): Promise<void> {
  await client.DELETE("/api/v1/documents/folders/{id}", {
    params: {
      path: { id },
    },
  });
}

export async function distributeDocument(
  documentId: string,
  targetEmployeeIds: string[],
): Promise<DistributeDocumentResponse> {
  const { data } = await client.POST<DistributeDocumentResponse>("/api/v1/documents/distributions", {
    body: {
      document_id: documentId,
      target_employee_ids: targetEmployeeIds,
    },
  });
  return data;
}

export async function fetchPendingAcknowledgements(): Promise<PendingAcknowledgement[]> {
  const { data } = await client.GET<PendingAcknowledgement[]>("/api/v1/ess/documents/pending");
  return data;
}

export async function acknowledgeDocument(id: string): Promise<AcknowledgeDocumentResponse> {
  const { data } = await client.POST<AcknowledgeDocumentResponse>("/api/v1/ess/documents/{id}/acknowledge", {
    params: {
      path: { id },
    },
  });
  return data;
}

export async function previewTemplate(
  templateContent: string,
  employeeId: string,
  manualVariables?: Record<string, string>,
): Promise<TemplatePreviewResponse> {
  const { data } = await client.POST<TemplatePreviewResponse>("/api/v1/documents/templates/preview", {
    body: {
      template_content: templateContent,
      employee_id: employeeId,
      manual_variables: manualVariables ?? null,
    },
  });
  return data;
}
