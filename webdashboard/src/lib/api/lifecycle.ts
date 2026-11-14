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

import { client } from "./client";
import { ApiError } from "./errors";
import { apiFetch } from "./helpers";

export interface LifecycleInstanceSummary {
  checklist_type: string;
  completed_at?: string | null;
  completed_items: number;
  created_at: string;
  department_name?: string | null;
  employee_code: string;
  employee_id: string;
  employee_name: string;
  employment_id: string;
  id: string;
  progress_percent: number;
  started_at: string;
  status: string;
  template_id?: string | null;
  template_name?: string | null;
  total_items: number;
  updated_at: string;
}

export async function fetchLifecycleInstances(): Promise<LifecycleInstanceSummary[]> {
  const res = await apiFetch("/api/v1/lifecycle/instances");
  if (!res.ok) {
    throw new ApiError(res.status, "Failed to fetch organization lifecycle instances");
  }
  return res.json() as Promise<LifecycleInstanceSummary[]>;
}

export async function fetchLifecycleTemplates(): Promise<LifecycleChecklistTemplate[]> {
  const { data } = await client.GET<LifecycleChecklistTemplate[]>("/api/v1/lifecycle-templates");
  return data;
}

export async function createLifecycleTemplate(
  body: CreateLifecycleChecklistTemplateRequest
): Promise<LifecycleChecklistTemplate> {
  const { data } = await client.POST<LifecycleChecklistTemplate>("/api/v1/lifecycle-templates", {
    body,
  });
  return data;
}

export async function getLifecycleTemplate(
  id: string
): Promise<LifecycleChecklistTemplate> {
  const { data } = await client.GET<LifecycleChecklistTemplate>("/api/v1/lifecycle-templates/{id}", {
    params: {
      path: { id },
    },
  });
  return data;
}

export async function updateLifecycleTemplate(
  id: string,
  body: UpdateLifecycleChecklistTemplateRequest
): Promise<LifecycleChecklistTemplate> {
  const { data } = await client.PATCH<LifecycleChecklistTemplate>("/api/v1/lifecycle-templates/{id}", {
    params: {
      path: { id },
    },
    body,
  });
  return data;
}

export async function deleteLifecycleTemplate(
  id: string
): Promise<void> {
  await client.DELETE("/api/v1/lifecycle-templates/{id}", {
    params: {
      path: { id },
    },
  });
}

// Template Items CRUD
export async function fetchLifecycleTemplateItems(
  id: string
): Promise<LifecycleChecklistTemplateItem[]> {
  const { data } = await client.GET<LifecycleChecklistTemplateItem[]>("/api/v1/lifecycle-templates/{id}/items", {
    params: {
      path: { id },
    },
  });
  return data;
}

export async function createLifecycleTemplateItem(
  id: string,
  body: CreateLifecycleChecklistTemplateItemRequest
): Promise<LifecycleChecklistTemplateItem> {
  const { data } = await client.POST<LifecycleChecklistTemplateItem>("/api/v1/lifecycle-templates/{id}/items", {
    params: {
      path: { id },
    },
    body,
  });
  return data;
}

export async function getLifecycleTemplateItem(
  tid: string,
  iid: string
): Promise<LifecycleChecklistTemplateItem> {
  const { data } = await client.GET<LifecycleChecklistTemplateItem>("/api/v1/lifecycle-templates/{tid}/items/{iid}", {
    params: {
      path: { tid, iid },
    },
  });
  return data;
}

export async function updateLifecycleTemplateItem(
  tid: string,
  iid: string,
  body: UpdateLifecycleChecklistTemplateItemRequest
): Promise<LifecycleChecklistTemplateItem> {
  const { data } = await client.PUT<LifecycleChecklistTemplateItem>("/api/v1/lifecycle-templates/{tid}/items/{iid}", {
    params: {
      path: { tid, iid },
    },
    body,
  });
  return data;
}

export async function deleteLifecycleTemplateItem(
  tid: string,
  iid: string
): Promise<void> {
  await client.DELETE("/api/v1/lifecycle-templates/{tid}/items/{iid}", {
    params: {
      path: { tid, iid },
    },
  });
}

// Checklist Instances
export async function fetchLifecycleChecklists(
  empId: string
): Promise<LifecycleChecklistResponse[]> {
  const { data } = await client.GET<LifecycleChecklistResponse[]>("/api/v1/employments/{emp_id}/lifecycle", {
    params: {
      path: { emp_id: empId },
    },
  });
  return data;
}

export async function createLifecycleChecklist(
  empId: string,
  body: CreateChecklistRequest
): Promise<LifecycleChecklistResponse> {
  const { data } = await client.POST<LifecycleChecklistResponse>("/api/v1/employments/{emp_id}/lifecycle", {
    params: {
      path: { emp_id: empId },
    },
    body,
  });
  return data;
}

export async function getLifecycleChecklist(
  id: string
): Promise<LifecycleChecklistResponse> {
  const { data } = await client.GET<LifecycleChecklistResponse>("/api/v1/lifecycle/{id}", {
    params: {
      path: { id },
    },
  });
  return data;
}

export async function updateChecklistItemStatus(
  id: string,
  iid: string,
  body: UpdateChecklistItemStatusRequest
): Promise<LifecycleChecklistResponse> {
  const { data } = await client.PATCH<LifecycleChecklistResponse>("/api/v1/lifecycle/{id}/items/{iid}", {
    params: {
      path: { id, iid },
    },
    body,
  });
  return data;
}

export async function fetchMyPendingTasks(): Promise<PendingTaskResponse[]> {
  const { data } = await client.GET<PendingTaskResponse[]>("/api/v1/lifecycle/my-tasks");
  return data;
}
