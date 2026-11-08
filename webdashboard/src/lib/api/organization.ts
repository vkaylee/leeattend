import { client } from "./client";
import type {
  Organization,
  OrganizationMember,
  OrganizationRole,
  MemberRolesResponse,
  PermissionCatalogItem,
} from "@/types/organization";

export async function fetchOrganization(): Promise<Organization> {
  const { data } = await client.GET<Organization>("/v1");
  return data;
}

export async function updateOrganization(patch: Partial<Organization>): Promise<Organization> {
  const { data } = await client.PATCH<Organization, Partial<Organization>>("/v1", patch);
  return data;
}

export async function fetchOrganizationMembers(): Promise<OrganizationMember[]> {
  const { data } = await client.GET<OrganizationMember[]>("/v1/members");
  return data;
}

export async function fetchOrganizationRoles(): Promise<OrganizationRole[]> {
  const { data } = await client.GET<OrganizationRole[]>("/v1/roles");
  return data;
}

export async function fetchMemberRoles(userId: string): Promise<MemberRolesResponse> {
  const { data } = await client.GET<MemberRolesResponse>(`/v1/members/${userId}/roles`);
  return data;
}

export async function fetchPermissionCatalog(): Promise<PermissionCatalogItem[]> {
  const { data } = await client.GET<PermissionCatalogItem[]>("/v1/permissions/catalog");
  return data;
}
