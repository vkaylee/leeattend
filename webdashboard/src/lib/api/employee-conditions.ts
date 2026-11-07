import { ApiError } from "./errors";
import { apiFetch } from "./helpers";
import { unwrapDataArray } from "@/types/api-overrides";

import type { WorkingCondition, CreateWorkingConditionRequest } from "./types";

import { client } from "./client";

export interface WorkingConditionWithEmployee extends WorkingCondition {
  employee_id: string;
  employee_name: string;
  employee_code?: string;
}

export async function fetchWorkspaceWorkingConditions(wsId: string): Promise<WorkingConditionWithEmployee[]> {
  const res = await apiFetch(`/v1/workspaces/${encodeURIComponent(wsId)}/working-conditions`);
  if (!res.ok) await ApiError.rejectResponse(res);
  const data = (await res.json()) as (WorkingConditionWithEmployee & { type_?: string })[];
  return data.map((item) => ({
    ...item,
    type: item.type_ ?? item.type,
  }));
}

export async function fetchWorkingConditions(wsId: string, employeeId: string): Promise<WorkingCondition[]> {
  const { data } = await client.GET("/workspaces/{workspace_id}/employees/{id}/conditions", {
    params: {
      path: { workspace_id: wsId, id: employeeId },
    },
  });
  return unwrapDataArray<WorkingCondition & { type_?: string }>(data).map((item) => ({
    ...item,
    type: item.type_ ?? item.type,
  }));
}

export async function createWorkingCondition(
  wsId: string,
  employeeId: string,
  condition: CreateWorkingConditionRequest
): Promise<WorkingCondition> {
  const { data } = await client.POST("/workspaces/{workspace_id}/employees/{id}/conditions", {
    params: {
      path: { workspace_id: wsId, id: employeeId },
    },
    body: condition,
  });
  return {
    ...data,
    type: data.type_,
  };
}

export async function deleteWorkingCondition(wsId: string, employeeId: string, conditionId: string): Promise<void> {
  await client.DELETE("/workspaces/{workspace_id}/employees/{id}/conditions/{cond_id}", {
    params: {
      path: { workspace_id: wsId, id: employeeId, cond_id: conditionId },
    },
  });
}
