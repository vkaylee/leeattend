import { Link } from "@tanstack/react-router";
import {
  CheckCircle2,
  Edit2,
  MoreVertical,
  Shield,
  ShieldAlert,
  ShieldCheck,
  Trash2,
  User,
  Users,
} from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import PermissionGate from "@/components/common/permission-gate";
import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import { cn } from "@/lib/utils";
import { ROUTES } from "@/lib/routes";

import type { OrganizationRole } from "../../../lib/api/types";

const colorMap: Record<string, string> = {
  rose: "bg-rose-50 text-rose-600 dark:bg-rose-500/10 dark:text-rose-400 border-rose-200/50 dark:border-rose-500/20",
  blue: "bg-blue-50 text-blue-600 dark:bg-blue-500/10 dark:text-blue-400 border-blue-200/50 dark:border-blue-500/20",
  emerald: "bg-emerald-50 text-emerald-600 dark:bg-emerald-500/10 dark:text-emerald-400 border-emerald-200/50 dark:border-emerald-500/20",
  slate: "bg-slate-50 text-slate-600 dark:bg-slate-500/10 dark:text-slate-400 border-slate-200/50 dark:border-slate-500/20",
};

const iconColorMap: Record<string, string> = {
  rose: "text-rose-500 dark:text-rose-400",
  blue: "text-blue-500 dark:text-blue-400",
  emerald: "text-emerald-500 dark:text-emerald-400",
  slate: "text-slate-500 dark:text-slate-400",
};

interface RoleIconProps {
  role: OrganizationRole;
}

export const RoleIcon = ({ role }: RoleIconProps) => {
  const bgColors = colorMap[role.color] || colorMap.slate;
  const iconColors = iconColorMap[role.color] || iconColorMap.slate;
  
  const Icon =
    role.is_system && role.name.toLowerCase() === "admin"
      ? ShieldAlert
      : role.is_system && role.name.toLowerCase() === "owner"
      ? ShieldCheck
      : role.is_system
      ? User
      : Shield;

  return (
    <div className={cn("w-14 h-14 rounded-2xl flex items-center justify-center border", bgColors)}>
      <Icon className={iconColors} size={24} />
    </div>
  );
};

interface RoleCardActionsProps {
  activeMenu: string | null;
  onDelete: (id: string) => void;
  onEdit: (role: OrganizationRole) => void;
  role: OrganizationRole;
  setActiveMenu: (id: string | null) => void;
}

export const RoleCardActions = ({
  activeMenu,
  onDelete,
  onEdit,
  role,
  setActiveMenu,
}: RoleCardActionsProps) => {
  const { t } = useTranslation();
  
  return (
    <PermissionGate permission="roles:manage">
      <div className="relative">
        <Button
          aria-label={t("common.actions", "Thao tác")}
          className="p-2 text-slate-400 hover:text-slate-600 dark:hover:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-800 rounded-xl transition-colors"
          onClick={() => { setActiveMenu(activeMenu === role.id ? null : role.id); }}
          type="button"
          variant="ghost"
        >
          <MoreVertical size={20} />
        </Button>

        {activeMenu === role.id && (
          <div className="absolute right-0 mt-2 w-48 bg-white dark:bg-slate-900 rounded-2xl shadow-xl shadow-slate-200/50 dark:shadow-black/40 border border-slate-100 dark:border-slate-800 py-2 z-10 animate-in fade-in zoom-in-95 duration-200">
            {!role.is_system && (
              <Button className="w-full flex items-center gap-3 px-4 py-2.5 text-sm font-medium text-slate-600 dark:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-800 transition-colors"
                onClick={() => { onEdit(role); }}
                type="button"
                variant="ghost"
              >
                <Edit2 size={16} /> {t("rolesPage.editRole", { defaultValue: "Edit Role" })}
              </Button>
            )}
            <Link
              className="w-full flex items-center gap-3 px-4 py-2.5 text-sm font-medium text-slate-600 dark:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-800 transition-colors"
              to={ROUTES.SETTINGS.PERMISSIONS}
            >
              <CheckCircle2 size={16} /> {t("rolesPage.matrix", { defaultValue: "Matrix" })}
            </Link>
            {!role.is_system && (
              <>
                <div className="h-px bg-slate-100 dark:bg-slate-800 my-1"></div>
                <Button className="w-full flex items-center gap-3 px-4 py-2.5 text-sm font-medium text-rose-600 dark:text-rose-400 hover:bg-rose-50 dark:hover:bg-rose-500/10 transition-colors"
                  onClick={() => { onDelete(role.id); }}
                  type="button"
                  variant="destructive"
                >
                  <Trash2 size={16} /> {t("rolesPage.deleteRole", { defaultValue: "Delete Role" })}
                </Button>
              </>
            )}
          </div>
        )}
      </div>
    </PermissionGate>
  );
};

interface RoleInfoProps {
  role: OrganizationRole;
}

export const RoleInfo = ({ role }: RoleInfoProps) => {
  const { t } = useTranslation();
  
  return (
    <div>
      <div className="flex items-center gap-2">
        <Typography className="text-lg font-bold text-slate-800 dark:text-slate-200" variant="h3">{role.name}</Typography>
        {role.is_system ? (
          <span className="px-2 py-0.5 rounded-lg text-[10px] font-bold uppercase tracking-wider bg-slate-100 dark:bg-slate-800 text-slate-500 dark:text-slate-400">
            {t("common.system", { defaultValue: "System" })}
          </span>
        ) : null}
      </div>
      <div className="flex items-center gap-1.5 text-sm font-medium text-slate-500 mt-1">
        <Users size={14} />
        <span>
          {t("rolesPage.roleId", { defaultValue: "Role ID:" })} {role.id.split("-")[0]}...
        </span>
      </div>
    </div>
  );
};
