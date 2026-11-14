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
} from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import PermissionGate from "@/components/common/permission-gate";
import Button from "@/components/ui/button";
import { ROUTES } from "@/lib/routes";
import { cn } from "@/lib/utils";

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
    <div className={cn("w-10 h-10 rounded-xl flex items-center justify-center border", bgColors)}>
      <Icon className={iconColors} size={24} />
    </div>
  );
};

interface RoleListActionsProps {
  activeMenu: string | null;
  onDelete: (id: string) => void;
  onEdit: (role: OrganizationRole) => void;
  role: OrganizationRole;
  setActiveMenu: (id: string | null) => void;
}

export const RoleListActions = ({
  activeMenu,
  onDelete,
  onEdit,
  role,
  setActiveMenu,
}: RoleListActionsProps) => {
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
          <div className="absolute right-0 mt-2 w-48 rounded-2xl border border-slate-100 bg-white py-2 shadow-xl shadow-slate-200/50 animate-in fade-in zoom-in-95 duration-200 dark:border-slate-800 dark:bg-slate-900 z-10">
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

