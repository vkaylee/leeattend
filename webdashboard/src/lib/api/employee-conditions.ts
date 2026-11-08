import { unwrapDataArray } from "@/types/api-overrides";

import type { CreateWorkingConditionRequest, WorkingCondition } from "./types";

import { client } from "./client";

export interface WorkingConditionWithEmployee extends WorkingCondition {
  employee_id: string;
  employee_name: string;
  employee_code?: string;
}

type WorkingConditionWire = WorkingCondition;

function normalizeCondition(item: WorkingConditionWire): WorkingCondition {
  return { ...item, type: item.type_ ?? item.type };
}

export async function fetchWorkspaceWorkingConditions(): Promise<WorkingConditionWithEmployee[]> {
  const { data } = await client.GET<WorkingConditionWithEmployee[]>("/v1/working-conditions");
  return data.map((item) => normalizeCondition(item) as WorkingConditionWithEmployee);
}

export async function fetchWorkingConditions(employeeId: string): Promise<WorkingCondition[]> {
  const { data } = await client.GET<WorkingConditionWire[] | { data: WorkingConditionWire[] }>(
    "/v1/employees/{id}/conditions",
    {
      params: {
        path: { id: employeeId },
      },
    },
  );
  return unwrapDataArray<WorkingConditionWire>(data).map(normalizeCondition);
}

export async function createWorkingCondition(
  employeeId: string,
  condition: CreateWorkingConditionRequest,
): Promise<WorkingCondition> {
  const { data } = await client.POST<WorkingConditionWire>("/v1/employees/{id}/conditions", {
    params: {
      path: { id: employeeId },
    },
    body: condition,
  });
  return normalizeCondition(data);
}

export async function deleteWorkingCondition(employeeId: string, conditionId: string): Promise<void> {
  await client.DELETE("/v1/employees/{id}/conditions/{cond_id}", {
    params: {
      path: { id: employeeId, cond_id: conditionId },
    },
  });
}
