import type { WorkingCondition, CreateWorkingConditionRequest } from "./types";

import { client } from "./client";

export async function fetchWorkingConditions(wsId: string, employeeId: string): Promise<WorkingCondition[]> {
  const { data } = await client.GET("/workspaces/{workspace_id}/employees/{id}/conditions", {
    params: {
      path: { workspace_id: wsId, id: employeeId },
    },
  });
  if (!data) return [];
  return (data.data ?? []).map((item) => ({
    ...item,
    type: item.type_,
  })) as WorkingCondition[];
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
  if (!data) throw new Error("No response data");
  return {
    ...data,
    type: data.type_,
  } as WorkingCondition;
}

export async function deleteWorkingCondition(wsId: string, employeeId: string, conditionId: string): Promise<void> {
  await client.DELETE("/workspaces/{workspace_id}/employees/{id}/conditions/{cond_id}", {
    params: {
      path: { workspace_id: wsId, id: employeeId, cond_id: conditionId },
    },
  });
}
