import { client } from "./client";

export async function fetchFolders(wsId: string) {
  const { data } = await client.GET("/v1/workspaces/{ws_id}/documents/folders", {
    params: {
      path: { ws_id: wsId },
    },
  });
  return data;
}

export async function createFolder(wsId: string, name: string, parentFolderId?: string) {
  const { data } = await client.POST("/v1/workspaces/{ws_id}/documents/folders", {
    params: {
      path: { ws_id: wsId },
    },
    body: {
      name,
      parent_folder_id: parentFolderId ?? null,
    },
  });
  return data;
}

export async function renameFolder(wsId: string, id: string, name: string) {
  const { data } = await client.PUT("/v1/workspaces/{ws_id}/documents/folders/{id}", {
    params: {
      path: { ws_id: wsId, id },
    },
    body: {
      name,
    },
  });
  return data;
}

export async function deleteFolder(wsId: string, id: string) {
  const { data } = await client.DELETE("/v1/workspaces/{ws_id}/documents/folders/{id}", {
    params: {
      path: { ws_id: wsId, id },
    },
  });
  return data;
}

export async function distributeDocument(wsId: string, documentId: string, targetEmployeeIds: string[]) {
  const { data } = await client.POST("/v1/workspaces/{ws_id}/documents/distributions", {
    params: {
      path: { ws_id: wsId },
    },
    body: {
      document_id: documentId,
      target_employee_ids: targetEmployeeIds,
    },
  });
  return data;
}

export async function fetchPendingAcknowledgements() {
  const { data } = await client.GET("/v1/ess/documents/pending", {});
  return data;
}

export async function acknowledgeDocument(id: string) {
  const { data } = await client.POST("/v1/ess/documents/{id}/acknowledge", {
    params: {
      path: { id },
    },
  });
  return data;
}

export async function previewTemplate(wsId: string, templateContent: string, employeeId: string, manualVariables?: Record<string, string>) {
  const { data } = await client.POST("/v1/workspaces/{ws_id}/documents/templates/preview", {
    params: {
      path: { ws_id: wsId },
    },
    body: {
      template_content: templateContent,
      employee_id: employeeId,
      manual_variables: manualVariables ?? null,
    },
  });
  return data;
}
