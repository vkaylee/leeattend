import type {
  CreateLifecycleChecklistTemplateRequest,
  LifecycleChecklistTemplate,
  UpdateLifecycleChecklistTemplateRequest,
  CreateLifecycleChecklistTemplateItemRequest,
  LifecycleChecklistTemplateItem,
  UpdateLifecycleChecklistTemplateItemRequest,
  LifecycleChecklistResponse,
  CreateChecklistRequest,
  PendingTaskResponse,
  UpdateChecklistItemStatusRequest,
} from "./types";

import { ApiError } from "./errors";
import { apiFetch } from "./helpers";
import { client } from "./client";

export interface LifecycleInstanceSummary {
  id: string;
  workspace_id: string;
  employment_id: string;
  employee_id: string;
  employee_name: string;
  employee_code: string;
  department_name?: string | null;
  template_id?: string | null;
  template_name?: string | null;
  checklist_type: string;
  status: string;
  progress_percent: number;
  total_items: number;
  completed_items: number;
  started_at: string;
  completed_at?: string | null;
  created_at: string;
  updated_at: string;
}

export async function fetchWorkspaceLifecycleInstances(
  wsId: string
): Promise<LifecycleInstanceSummary[]> {
  const res = await apiFetch(`/api/v1/workspaces/${wsId}/lifecycle/instances`);
  if (!res.ok) {
    throw new ApiError(res.status, "Failed to fetch workspace lifecycle instances");
  }
  return res.json() as Promise<LifecycleInstanceSummary[]>;
}

export async function fetchLifecycleTemplates(
  wsId: string
): Promise<LifecycleChecklistTemplate[]> {
  const { data } = await client.GET("/workspaces/{ws_id}/lifecycle-templates", {
    params: {
      path: { ws_id: wsId },
    },
  });
  return data;
}

export async function createLifecycleTemplate(
  wsId: string,
  body: CreateLifecycleChecklistTemplateRequest
): Promise<LifecycleChecklistTemplate> {
  const { data } = await client.POST("/workspaces/{ws_id}/lifecycle-templates", {
    params: {
      path: { ws_id: wsId },
    },
    body,
  });
  return data;
}

export async function getLifecycleTemplate(
  wsId: string,
  id: string
): Promise<LifecycleChecklistTemplate> {
  const { data } = await client.GET("/workspaces/{ws_id}/lifecycle-templates/{id}", {
    params: {
      path: { ws_id: wsId, id },
    },
  });
  return data;
}

export async function updateLifecycleTemplate(
  wsId: string,
  id: string,
  body: UpdateLifecycleChecklistTemplateRequest
): Promise<LifecycleChecklistTemplate> {
  const { data } = await client.PATCH("/workspaces/{ws_id}/lifecycle-templates/{id}", {
    params: {
      path: { ws_id: wsId, id },
    },
    body,
  });
  return data;
}

export async function deleteLifecycleTemplate(
  wsId: string,
  id: string
): Promise<void> {
  await client.DELETE("/workspaces/{ws_id}/lifecycle-templates/{id}", {
    params: {
      path: { ws_id: wsId, id },
    },
  });
}

// Template Items CRUD
export async function fetchLifecycleTemplateItems(
  wsId: string,
  id: string
): Promise<LifecycleChecklistTemplateItem[]> {
  const { data } = await client.GET("/workspaces/{ws_id}/lifecycle-templates/{id}/items", {
    params: {
      path: { ws_id: wsId, id },
    },
  });
  return data;
}

export async function createLifecycleTemplateItem(
  wsId: string,
  id: string,
  body: CreateLifecycleChecklistTemplateItemRequest
): Promise<LifecycleChecklistTemplateItem> {
  const { data } = await client.POST("/workspaces/{ws_id}/lifecycle-templates/{id}/items", {
    params: {
      path: { ws_id: wsId, id },
    },
    body,
  });
  return data;
}

export async function getLifecycleTemplateItem(
  wsId: string,
  tid: string,
  iid: string
): Promise<LifecycleChecklistTemplateItem> {
  const { data } = await client.GET("/workspaces/{ws_id}/lifecycle-templates/{tid}/items/{iid}", {
    params: {
      path: { ws_id: wsId, tid, iid },
    },
  });
  return data;
}

export async function updateLifecycleTemplateItem(
  wsId: string,
  tid: string,
  iid: string,
  body: UpdateLifecycleChecklistTemplateItemRequest
): Promise<LifecycleChecklistTemplateItem> {
  const { data } = await client.PUT("/workspaces/{ws_id}/lifecycle-templates/{tid}/items/{iid}", {
    params: {
      path: { ws_id: wsId, tid, iid },
    },
    body,
  });
  return data;
}

export async function deleteLifecycleTemplateItem(
  wsId: string,
  tid: string,
  iid: string
): Promise<void> {
  await client.DELETE("/workspaces/{ws_id}/lifecycle-templates/{tid}/items/{iid}", {
    params: {
      path: { ws_id: wsId, tid, iid },
    },
  });
}

// Checklist Instances
export async function fetchLifecycleChecklists(
  wsId: string,
  empId: string
): Promise<LifecycleChecklistResponse[]> {
  const { data } = await client.GET("/workspaces/{ws_id}/employments/{emp_id}/lifecycle", {
    params: {
      path: { ws_id: wsId, emp_id: empId },
    },
  });
  return data;
}

export async function createLifecycleChecklist(
  wsId: string,
  empId: string,
  body: CreateChecklistRequest
): Promise<LifecycleChecklistResponse> {
  const { data } = await client.POST("/workspaces/{ws_id}/employments/{emp_id}/lifecycle", {
    params: {
      path: { ws_id: wsId, emp_id: empId },
    },
    body,
  });
  return data;
}

export async function getLifecycleChecklist(
  wsId: string,
  id: string
): Promise<LifecycleChecklistResponse> {
  const { data } = await client.GET("/workspaces/{ws_id}/lifecycle/{id}", {
    params: {
      path: { ws_id: wsId, id },
    },
  });
  return data;
}

export async function updateChecklistItemStatus(
  wsId: string,
  id: string,
  iid: string,
  body: UpdateChecklistItemStatusRequest
): Promise<LifecycleChecklistResponse> {
  const { data } = await client.PATCH("/workspaces/{ws_id}/lifecycle/{id}/items/{iid}", {
    params: {
      path: { ws_id: wsId, id, iid },
    },
    body,
  });
  return data;
}

// Dashboard Pending Tasks
export async function fetchMyPendingTasks(
  wsId: string
): Promise<PendingTaskResponse[]> {
  const { data } = await client.GET("/workspaces/{ws_id}/lifecycle/my-tasks", {
    params: {
      path: { ws_id: wsId },
    },
  });
  return data;
}
