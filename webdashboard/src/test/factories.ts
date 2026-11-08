import type { Organization, OrganizationMember } from "@/types/organization";
import type { User } from "@/lib/api";
import { generateUUID } from "@/utils/uuid";

export const userFactory = (overrides: Partial<User> = {}): User => {
  const id = generateUUID();
  return {
    id,
    email: `user_${id.slice(0, 8)}@example.com`,
    full_name: `User ${id.slice(0, 4)}`,
    avatar_url: undefined,
    bio: undefined,
    created_at: new Date().toISOString(),
    ...overrides,
  };
};

export const organizationFactory = (
  overrides: Partial<Organization> = {},
): Organization => {
  const id = generateUUID();
  return {
    name: `Organization ${id.slice(0, 8)}`,
    date_format: "DD/MM/YYYY",
    time_format: "24h",
    timezone: "Asia/Ho_Chi_Minh",
    theme_primary_color: "#10b981",
    default_language: "en",
    default_holiday_ot_multiplier: "1.5",
    holiday_overnight_strategy: "split",
    created_at: new Date().toISOString(),
    ...overrides,
  };
};

export const memberFactory = (
  overrides: Partial<OrganizationMember> = {},
): OrganizationMember => {
  return {
    user_id: generateUUID(),
    role_id: "role-member",
    extra_role_ids: [],
    email: "test@example.com",
    ...overrides,
  };
};
