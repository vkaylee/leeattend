import { Link } from "@tanstack/react-router";
import { ChevronRight, User } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import PermissionGate from "@/components/common/permission-gate";
import { Typography } from "@/components/ui/typography";
import { ROUTES } from "@/lib/routes";

import type { WorkspaceRole } from "../../../lib/api/types";

import { RoleCardActions, RoleIcon, RoleInfo } from "./role-card-subcomponents";

interface RoleCardProps {
  activeMenu: string | null;
  onDelete: (id: string) => void;
  onEdit: (role: WorkspaceRole) => void;
  role: WorkspaceRole;
  setActiveMenu: (id: string | null) => void;
}

export function RoleCard({ role, activeMenu, setActiveMenu, onEdit, onDelete }: RoleCardProps) {
  const { t } = useTranslation();

  return (
    <div className="bg-white dark:bg-slate-900 rounded-2xl sm:rounded-[2rem] border border-slate-100 dark:border-slate-800 p-5 sm:p-6 shadow-sm hover:shadow-xl hover:shadow-slate-200/50 dark:hover:shadow-black/20 transition-all duration-300 group">
      <div className="flex items-start justify-between mb-4">
        <div className="flex items-center gap-4">
          <RoleIcon role={role} />
          <RoleInfo role={role} />
        </div>

        <RoleCardActions
          activeMenu={activeMenu}
          onDelete={onDelete}
          onEdit={onEdit}
          role={role}
          setActiveMenu={setActiveMenu}
        />
      </div>

      <Typography className="text-sm font-medium text-slate-500 dark:text-slate-400 mb-6 line-clamp-2 min-h-[40px]" variant="p">
        {role.description ?? t("rolesPage.noDescription", { defaultValue: "No description provided." })}
      </Typography>

      <div className="space-y-4">
        <div className="pt-4 border-t border-slate-100 dark:border-slate-800 flex items-center justify-between">
          <div className="flex -space-x-3">
            <div className="w-8 h-8 rounded-full border-2 border-white dark:border-slate-900 bg-gradient-to-tr from-slate-200 to-slate-300 dark:from-slate-700 dark:to-slate-600 flex items-center justify-center text-[10px] font-bold text-slate-600 dark:text-slate-200 shadow-sm">
              <User size={14} />
            </div>
          </div>
          <PermissionGate permission="members:roles:manage">
            <Link
              className="text-xs font-bold text-primary-600 dark:text-primary-400 hover:text-primary-700 dark:hover:text-primary-300 opacity-0 group-hover:opacity-100 transition-opacity flex items-center gap-1"
              to={ROUTES.SETTINGS.PERMISSIONS}
            >
              {t("rolesPage.manageAccess", { defaultValue: "Manage Access" })} <ChevronRight size={14} />
            </Link>
          </PermissionGate>
        </div>
      </div>
    </div>
  );
}
