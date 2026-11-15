import type { TFunction } from "i18next";

import {
  ChevronDown,
  ChevronRight,
  Eye,
  MoreHorizontal,
  Pencil, 
  Trash2
} from "lucide-react";
import React from "react";

import type { Department, Employee } from "@/lib/api";

import Button from "@/components/ui/button";
import { DropdownMenu, DropdownMenuTrigger, DropdownMenuContent, DropdownMenuItem } from "@/components/ui/dropdown-menu";

import type { TreeNode } from "./department-org-types";

interface DepartmentOrgMobileNodeProps {
  depth: number;
  isCollapsed: boolean;
  isExplicitlyCollapsed: boolean;
  isMatch: boolean;
  manager: Employee | undefined;
  node: TreeNode;
  onClick: (dept: Department) => void;
  onDelete: (dept: Department) => void;
  onEdit: (dept: Department) => void;
  onToggleCollapse: (id: string, currentCollapsed: boolean) => void;
  t: TFunction;
}

export const DepartmentOrgMobileNode = ({
  depth,
  isCollapsed,
  isExplicitlyCollapsed,
  isMatch,
  manager,
  node,
  onClick,
  onDelete,
  onEdit,
  onToggleCollapse,
  t
}: DepartmentOrgMobileNodeProps) => {
  const hasChildren = node.children.length > 0;

  return (
    <div 
      className="flex items-center gap-3 p-4 border-b border-slate-50 dark:border-slate-800 hover:bg-slate-50/50 dark:hover:bg-slate-800/10"
      style={{ paddingLeft: `${Math.max(16, depth * 24)}px` }}
    >
      {hasChildren ? (
        <Button
          className="w-6 h-6 min-w-0 p-0 flex items-center justify-center text-slate-400 dark:text-slate-500"
          onClick={() => {
            onToggleCollapse(node.id, isExplicitlyCollapsed);
          }}
          size="icon"
          variant="ghost"
        >
          {isCollapsed ? <ChevronRight size={16} /> : <ChevronDown size={16} />}
        </Button>
      ) : (
        <div className="w-6" />
      )}

      <div
        className="flex-1 min-w-0 cursor-pointer"
        onClick={() => {
          if (node.id !== "virtual-root") {
            onClick(node.dept);
          }
        }}
        onKeyDown={(e) => {
          if (node.id !== "virtual-root" && (e.key === "Enter" || e.key === " ")) {
            e.preventDefault();
            onClick(node.dept);
          }
        }}
        role="button"
        tabIndex={0}
      >
        <div className="flex items-center gap-2">
          <span className={`font-bold text-sm uppercase tracking-tight ${isMatch ? "text-primary-500" : "text-slate-700 dark:text-slate-200"}`}>
            {node.name}
          </span>
          <span className="text-[10px] font-bold bg-slate-100 dark:bg-slate-800 px-2 py-0.5 rounded-md text-slate-500">
            {node.active_employee_count} {t("departments.columns.employees").toLowerCase()}
          </span>
        </div>
        {node.id !== "virtual-root" && (
          <div className="text-[10px] text-slate-400 dark:text-slate-500 font-semibold uppercase tracking-wider flex items-center gap-1 mt-0.5">
            <span>{t("departments.manager_label")}:</span>
            <span className="text-slate-600 dark:text-slate-300">
              {manager ? manager.full_name : t("departments.not_assigned").toUpperCase()}
            </span>
          </div>
        )}
      </div>

      {node.id !== "virtual-root" && (
        <>
          <Button
            className="h-8 w-8 px-0"
            icon={<Eye size={14} />}
            onClick={() => { onClick(node.dept); }}
            size="sm"
            variant="ghost"
          />

          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button
                className="h-8 w-8 px-0"
                icon={<MoreHorizontal size={14} />}
                size="sm"
                variant="ghost"
              />
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end">
              <DropdownMenuItem onClick={() => { onEdit(node.dept); }}>
                <Pencil className="mr-2 h-4 w-4" />
                {t("common.edit")}
              </DropdownMenuItem>
              <DropdownMenuItem className="text-rose-600 dark:text-rose-400" onClick={() => { onDelete(node.dept); }}>
                <Trash2 className="mr-2 h-4 w-4" />
                {t("common.delete")}
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </>
      )}
    </div>
  );
};
