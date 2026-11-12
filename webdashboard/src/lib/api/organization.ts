import type {
  MemberExtraPermission,
  MemberRolesResponse,
  Organization,
  OrganizationMember,
  OrganizationRole,
  PermissionCatalogItem,
} from "@/types/organization";

import { client } from "./client";

export async function fetchOrganization(): Promise<Organization> {
  const { data } = await client.GET<Organization>("/api/v1");
  return data;
}

export async function updateOrganization(patch: Partial<Organization>): Promise<Organization> {
  const { data } = await client.PATCH<Organization>("/api/v1", { body: patch });
  return data;
}

interface OrganizationMemberListResponse {
  data: OrganizationMember[];
}

export async function fetchOrganizationMembers(): Promise<OrganizationMember[]> {
  const { data } = await client.GET<OrganizationMemberListResponse>("/api/v1/members");
  return data.data;
}

export const fetchAllOrganizationMembers = fetchOrganizationMembers;

interface OrganizationRoleListResponse {
  data: OrganizationRole[];
}

export async function fetchOrganizationRoles(): Promise<OrganizationRole[]> {
  const { data } = await client.GET<OrganizationRoleListResponse>("/api/v1/roles");
  return data.data;
}

export async function addOrganizationMember(email: string, roleId: string): Promise<void> {
  await client.POST("/api/v1/members", { body: { email, role_id: roleId } });
}

export async function removeOrganizationMember(userId: string): Promise<void> {
  await client.DELETE(`/api/v1/members/${userId}`);
}

export async function updateOrganizationMemberRole(userId: string, roleId: string): Promise<void> {
  await client.PATCH(`/api/v1/members/${userId}`, { body: { role_id: roleId } });
}

export async function fetchMemberRoles(userId: string): Promise<MemberRolesResponse> {
  const { data } = await client.GET<MemberRolesResponse>(`/api/v1/members/${userId}/roles`);
  return data;
}

export async function assignExtraRole(userId: string, roleId: string, expiresAt: string): Promise<unknown> {
  const { data } = await client.POST(`/api/v1/members/${userId}/roles`, { body: { role_id: roleId, expires_at: expiresAt } });
  return data;
}

export async function revokeExtraRole(userId: string, roleId: string): Promise<void> {
  await client.DELETE(`/api/v1/members/${userId}/roles/${roleId}`);
}

export async function fetchMemberPermissions(userId: string): Promise<MemberExtraPermission[]> {
  const { data } = await client.GET<MemberExtraPermission[]>(`/api/v1/members/${userId}/permissions`);
  return data;
}

export async function assignExtraPermission(userId: string, permission: string, expiresAt: string): Promise<unknown> {
  const { data } = await client.POST(`/api/v1/members/${userId}/permissions`, { body: { permission, expires_at: expiresAt } });
  return data;
}

export async function revokeExtraPermission(userId: string, permId: string): Promise<void> {
  await client.DELETE(`/api/v1/members/${userId}/permissions/${permId}`);
}

export async function fetchPermissionCatalog(): Promise<PermissionCatalogItem[]> {
  const { data } = await client.GET<PermissionCatalogItem[]>("/api/v1/permissions/catalog");
  return data;
}

export async function createOrganizationRole(data: { name: string; description?: string; color: string }): Promise<OrganizationRole> {
  const { data: resData } = await client.POST<OrganizationRole>("/api/v1/roles", { body: data });
  return resData;
}

export async function updateOrganizationRole(roleId: string, data: { name?: string; description?: string; color?: string }): Promise<OrganizationRole> {
  const { data: resData } = await client.PUT<OrganizationRole>(`/api/v1/roles/${roleId}`, { body: data });
  return resData;
}

export async function deleteOrganizationRole(roleId: string): Promise<void> {
  await client.DELETE(`/api/v1/roles/${roleId}`);
}

export async function fetchRolePermissions(roleId: string): Promise<string[]> {
  const { data: resData } = await client.GET<string[]>(`/api/v1/roles/${roleId}/permissions`);
  return resData;
}

export async function updateRolePermissions(roleId: string, permissions: string[]): Promise<void> {
  await client.PUT(`/api/v1/roles/${roleId}/permissions`, { body: { permissions } });
}
