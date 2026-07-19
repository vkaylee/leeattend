import type { WorkingCondition, CreateWorkingConditionRequest } from "./types";

import { client } from "./client";

export async function fetchWorkingConditions(wsId: string, employeeId: string): Promise<WorkingCondition[]> {
  const { data } = await client.GET("/workspaces/{workspace_id}/employees/{id}/conditions", {
    params: {
      path: { workspace_id: wsId, id: employeeId },
    },
  });
  // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
  return (data.data ?? []).map((item) => ({
    ...item,
    type: item.type_,
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
