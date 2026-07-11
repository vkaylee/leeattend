import type {
  CreateCustomFieldDefinitionRequest,
  CustomFieldDefinition,
  CustomFieldValueResponse,
  SetCustomFieldValueRequest,
  UpdateCustomFieldDefinitionRequest,
} from "./types";

import { client } from "./client";

export async function fetchCustomFieldDefinitions(
  wsId: string,
  entityType?: string
): Promise<CustomFieldDefinition[]> {
  const { data } = await client.GET("/workspaces/{ws_id}/custom-fields/definitions", {
    params: {
      path: { ws_id: wsId },
      query: { entity_type: entityType },
    },
  });
  return data;
}

export async function createCustomFieldDefinition(
  wsId: string,
  body: CreateCustomFieldDefinitionRequest
): Promise<CustomFieldDefinition> {
  const { data } = await client.POST("/workspaces/{ws_id}/custom-fields/definitions", {
    params: {
      path: { ws_id: wsId },
    },
    body,
  });
  return data;
}

export async function getCustomFieldDefinition(
  wsId: string,
  id: string
): Promise<CustomFieldDefinition> {
  const { data } = await client.GET("/workspaces/{ws_id}/custom-fields/definitions/{id}", {
    params: {
      path: { ws_id: wsId, id },
    },
  });
  return data;
}

export async function updateCustomFieldDefinition(
  wsId: string,
  id: string,
  body: UpdateCustomFieldDefinitionRequest
): Promise<CustomFieldDefinition> {
  const { data } = await client.PATCH("/workspaces/{ws_id}/custom-fields/definitions/{id}", {
    params: {
      path: { ws_id: wsId, id },
    },
    body,
  });
  return data;
}

export async function deleteCustomFieldDefinition(
  wsId: string,
  id: string
): Promise<void> {
  await client.DELETE("/workspaces/{ws_id}/custom-fields/definitions/{id}", {
    params: {
      path: { ws_id: wsId, id },
    },
  });
}

export async function fetchCustomFieldValues(
  wsId: string,
  entityType: string,
  entityId: string
): Promise<CustomFieldValueResponse[]> {
  const { data } = await client.GET(
    "/workspaces/{ws_id}/custom-fields/values/{entity_type}/{entity_id}",
    {
      params: {
        path: { ws_id: wsId, entity_type: entityType, entity_id: entityId },
      },
    }
  );
  return data;
}

export async function setCustomFieldValues(
  wsId: string,
  entityType: string,
  entityId: string,
  values: SetCustomFieldValueRequest[]
): Promise<CustomFieldValueResponse[]> {
  const { data } = await client.PUT(
    "/workspaces/{ws_id}/custom-fields/values/{entity_type}/{entity_id}",
    {
      params: {
        path: { ws_id: wsId, entity_type: entityType, entity_id: entityId },
      },
      body: { values },
    }
  );
  return data;
}
