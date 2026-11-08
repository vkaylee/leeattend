import type {
  CreateCustomFieldDefinitionRequest,
  CustomFieldDefinition,
  CustomFieldValueResponse,
  SetCustomFieldValueRequest,
  UpdateCustomFieldDefinitionRequest,
} from "./types";

import { client } from "./client";

export async function fetchCustomFieldDefinitions(
  entityType?: string,
): Promise<CustomFieldDefinition[]> {
  const { data } = await client.GET<CustomFieldDefinition[]>("/v1/custom-fields/definitions", {
    params: {
      query: { entity_type: entityType },
    },
  });
  return data;
}

export async function createCustomFieldDefinition(
  body: CreateCustomFieldDefinitionRequest,
): Promise<CustomFieldDefinition> {
  const { data } = await client.POST<CustomFieldDefinition>("/v1/custom-fields/definitions", {
    body,
  });
  return data;
}

export async function getCustomFieldDefinition(
  id: string,
): Promise<CustomFieldDefinition> {
  const { data } = await client.GET<CustomFieldDefinition>("/v1/custom-fields/definitions/{id}", {
    params: {
      path: { id },
    },
  });
  return data;
}

export async function updateCustomFieldDefinition(
  id: string,
  body: UpdateCustomFieldDefinitionRequest,
): Promise<CustomFieldDefinition> {
  const { data } = await client.PATCH<CustomFieldDefinition>("/v1/custom-fields/definitions/{id}", {
    params: {
      path: { id },
    },
    body,
  });
  return data;
}

export async function deleteCustomFieldDefinition(id: string): Promise<void> {
  await client.DELETE("/v1/custom-fields/definitions/{id}", {
    params: {
      path: { id },
    },
  });
}

export async function fetchCustomFieldValues(
  entityType: string,
  entityId: string,
): Promise<CustomFieldValueResponse[]> {
  const { data } = await client.GET<CustomFieldValueResponse[]>(
    "/v1/custom-fields/values/{entity_type}/{entity_id}",
    {
      params: {
        path: { entity_type: entityType, entity_id: entityId },
      },
    },
  );
  return data;
}

export async function setCustomFieldValues(
  entityType: string,
  entityId: string,
  values: SetCustomFieldValueRequest[],
): Promise<CustomFieldValueResponse[]> {
  const { data } = await client.PUT<CustomFieldValueResponse[]>(
    "/v1/custom-fields/values/{entity_type}/{entity_id}",
    {
      params: {
        path: { entity_type: entityType, entity_id: entityId },
      },
      body: { values },
    },
  );
  return data;
}
