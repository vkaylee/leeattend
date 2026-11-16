export function formatRoleName(
  name: string,
  t: (key: string, options?: { defaultValue?: string }) => string,
): string {
  const roleKey = name.toLowerCase().replace(/[\s-]+/g, "_");
  const knownRoles: Record<string, string> = {
    admin: t("roles.admin", { defaultValue: "Quản trị viên" }),
    owner: t("roles.owner", { defaultValue: "Chủ sở hữu" }),
    hr_manager: t("roles.hr_manager", { defaultValue: "Quản lý nhân sự" }),
    shift_manager: t("roles.shift_manager", { defaultValue: "Quản lý ca" }),
    team_lead: t("roles.team_lead", { defaultValue: "Trưởng nhóm" }),
    employee: t("roles.employee", { defaultValue: "Nhân viên" }),
  };
  return knownRoles[roleKey] ?? name;
}
