export interface Organization {
  created_at: string;
  date_format: string;
  default_holiday_ot_multiplier: string;
  default_language: string;
  holiday_overnight_strategy: string;
  id: string;
  logo_url?: string | null;
  name: string;
  tagline?: string | null;
  theme_primary_color: string;
  time_format: string;
  timezone: string;
}

export interface OrganizationMember {
  avatar_url?: string | null;
  email: string;
  extra_role_ids: string[];
  full_name?: string | null;
  role_id: string;
  user_id: string;
}

export type RoleScopeType = "global" | "scoped";

export interface OrganizationRole {
  color: string;
  created_at: string;
  description?: string | null;
  id: string;
  is_system: boolean;
  name: string;
  scope_type: RoleScopeType;
  updated_at: string;
}

export interface MemberExtraRole {
  assigned_at?: string | null;
  expires_at: string;
  id: string;
  role_id: string;
  user_id: string;
}

export interface MemberExtraPermission {
  assigned_at?: string | null;
  assigned_by?: string | null;
  expires_at: string;
  id: string;
  permission: string;
  user_id: string;
}

export interface MemberRolesResponse {
  extra: MemberExtraRole[];
  primary: string;
}

export interface PermissionCatalogItem {
  group: string;
  id: string;
  label_key: string;
}
