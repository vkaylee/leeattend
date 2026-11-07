import { Link } from "@tanstack/react-router";
import { ArrowLeftRight, Eye, MoreHorizontal, Pencil, UserMinus, UserPlus } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import PermissionGate from "@/components/common/permission-gate";
import Button from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import type { Employee } from "@/lib/api/types";
import type { EmployeeModalState } from "../types/employee-modals";

interface EmployeeRowActionsProps {
  employee: Employee;
  onOpenModal?: (modal: EmployeeModalState) => void;
  onSelect: (employee: Employee) => void;
}

export function EmployeeRowActions({
  employee,
  onOpenModal,
  onSelect,
}: EmployeeRowActionsProps) {
  const { t } = useTranslation();
  const isActive = Boolean(employee.active_employment_id);

  return (
    <div className="flex items-center justify-end" onClick={(e) => { e.stopPropagation(); }}>
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button
            aria-label={t("common.actions", "Thao tác")}
            className="size-8 p-0 rounded-xl hover:bg-slate-100 dark:hover:bg-slate-800 text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 transition-colors"
            data-testid={`employee-actions-${employee.id}`}
            type="button"
            variant="ghost"
          >
            <MoreHorizontal size={16} />
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end" className="w-48">
          <DropdownMenuItem
            asChild
            className="flex items-center gap-2.5 text-xs cursor-pointer"
          >
            <Link
              className="flex items-center gap-2.5 w-full"
              data-testid={`view-profile-${employee.id}`}
              params={{ employeeId: employee.id }}
              to="/employees/$employeeId"
            >
              <Eye className="text-slate-400" size={14} />
              <span>{t("employees.details.view_profile", "Xem hồ sơ 360°")}</span>
            </Link>
          </DropdownMenuItem>
          <PermissionGate permission="employees:update">
            <DropdownMenuItem
              className="flex items-center gap-2.5 text-xs cursor-pointer"
              onClick={() => {
                onOpenModal?.({ type: "edit", employee });
              }}
            >
              <Pencil className="text-slate-400" size={14} />
              <span>{t("employees.details.edit_profile", "Sửa thông tin")}</span>
            </DropdownMenuItem>

            <DropdownMenuItem
              className="flex items-center gap-2.5 text-xs cursor-pointer"
              onClick={() => {
                onOpenModal?.({ type: "transfer", employee });
              }}
            >
              <ArrowLeftRight className="text-slate-400" size={14} />
              <span>{t("employees.details.transfer", "Chuyển phòng ban")}</span>
            </DropdownMenuItem>

            <DropdownMenuSeparator />

            {isActive ? (
              <DropdownMenuItem
                className="flex items-center gap-2.5 text-xs cursor-pointer text-rose-600 dark:text-rose-400 focus:text-rose-600 focus:bg-rose-50 dark:focus:bg-rose-950/50"
                onClick={() => {
                  onOpenModal?.({ type: "terminate", employee });
                }}
              >
                <UserMinus size={14} />
                <span>{t("employees.details.terminate", "Thôi việc")}</span>
              </DropdownMenuItem>
            ) : (
              <DropdownMenuItem
                className="flex items-center gap-2.5 text-xs cursor-pointer text-emerald-600 dark:text-emerald-400 focus:text-emerald-600 focus:bg-emerald-50 dark:focus:bg-emerald-950/50"
                onClick={() => {
                  onOpenModal?.({ type: "rehire", employee });
                }}
              >
                <UserPlus size={14} />
                <span>{t("employees.details.rehire", "Tuyển dụng lại")}</span>
              </DropdownMenuItem>
            )}
          </PermissionGate>
        </DropdownMenuContent>
      </DropdownMenu>
    </div>
  );
}
