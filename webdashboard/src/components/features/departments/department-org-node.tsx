import type { TFunction } from "i18next";

import { Handle, Position } from "@xyflow/react";
import { 
  Building2,
  ChevronDown,
  MoreHorizontal, 
  Pencil, 
  Trash2, 
  UserCircle, 
  Users 
} from "lucide-react";
import React from "react";

import type { Department, Employee } from "@/lib/api";

import Button from "@/components/ui/button";
import { DropdownMenu, DropdownMenuTrigger, DropdownMenuContent, DropdownMenuItem } from "@/components/ui/dropdown-menu";
import { Typography } from "@/components/ui/typography";
import { cn } from "@/lib/utils";

import type { DepartmentNodeData, TreeNode } from "./department-org-types";

interface DepartmentNodeActionsProps {
  node: TreeNode;
  onDelete: (dept: Department) => void;
  onEdit: (dept: Department) => void;
  t: TFunction;
}

const DepartmentNodeActions = ({
  node,
  onDelete,
  onEdit,
  t
}: DepartmentNodeActionsProps) => {
  if (node.id === "virtual-root") return null;

  return (
    <div className="absolute top-4 right-4 z-20" data-actions-menu>
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button
            aria-label={t("departments.columns.actions")}
            className="h-11 w-11 rounded-lg hover:text-primary-600 dark:hover:text-primary-400 border-none bg-transparent hover:bg-slate-50 dark:hover:bg-slate-800"
            icon={<MoreHorizontal size={14} />}
            size="sm"
            variant="outline"
          />
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end">
          <DropdownMenuItem onClick={(e) => { e.stopPropagation(); onEdit(node.dept); }}>
            <Pencil className="mr-2 h-4 w-4" />
            {t("common.edit")}
          </DropdownMenuItem>
          <DropdownMenuItem className="text-rose-600 dark:text-rose-400" onClick={(e) => { e.stopPropagation(); onDelete(node.dept); }}>
            <Trash2 className="mr-2 h-4 w-4" />
            {t("common.delete")}
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    </div>
  );
};

interface DepartmentNodeMetricsProps {
  manager: Employee | undefined;
  node: TreeNode;
  t: TFunction;
}

const DepartmentNodeMetrics = ({
  manager,
  node,
  t
}: DepartmentNodeMetricsProps) => {
  return (
    <div className="space-y-2 pt-3 border-t border-slate-50 dark:border-slate-800 text-[10px]">
      <div className="flex items-center justify-between text-slate-500 dark:text-slate-400 font-bold uppercase tracking-wider">
        <span className="flex items-center gap-1.5">
          <Users className="opacity-60" size={12} />
          {t("departments.active_employees_label")}
        </span>
        <span className="text-slate-800 dark:text-slate-200">
          {node.active_employee_count}
        </span>
      </div>
      
      {node.id !== "virtual-root" && (
        <div className="flex items-center justify-between text-slate-500 dark:text-slate-400 font-bold uppercase tracking-wider">
          <span className="flex items-center gap-1.5">
            <UserCircle className="opacity-60" size={12} />
            {t("departments.manager_label")}
          </span>
          <span className="text-slate-800 dark:text-slate-200 max-w-[120px] truncate text-right">
            {manager ? manager.full_name : t("departments.not_assigned").toUpperCase()}
          </span>
        </div>
      )}
    </div>
  );
};

interface DepartmentNodeHandlesProps {
  hasSource: boolean;
  sourcePosition: "bottom" | "left";
  targetPosition: "top" | "left";
}

const DepartmentNodeHandles = ({
  hasSource,
  sourcePosition,
  targetPosition
}: DepartmentNodeHandlesProps) => {
  return (
    <>
      {targetPosition === "left" ? (
        <Handle
          className="!top-[46px] !bg-slate-300 !border !border-slate-400 !w-2 !h-2"
          id="left-target"
          position={Position.Left}
          type="target"
        />
      ) : (
        <Handle
          className="!left-1/2 !bg-slate-300 !border !border-slate-400 !w-2 !h-2"
          id="top-target"
          position={Position.Top}
          type="target"
        />
      )}

      {hasSource ? (
        sourcePosition === "left" ? (
          <Handle
            className="!top-[46px] !bg-slate-300 !border !border-slate-400 !w-2 !h-2"
            id="left-source"
            position={Position.Left}
            type="source"
          />
        ) : (
          <Handle
            className="!left-1/2 !bg-slate-300 !border !border-slate-400 !w-2 !h-2"
            id="bottom-source"
            position={Position.Bottom}
            type="source"
          />
        )
      ) : null}
    </>
  );
};

// Custom Node component for React Flow
export const DepartmentNode = ({ data }: { data: DepartmentNodeData }) => {
  const { 
    hasSource,
    isCollapsed,
    isMatch,
    isRelevant,
    manager,
    node,
    onClick,
    onDelete,
    onEdit,
    onToggleCollapse,
    sourcePosition,
    t,
    targetPosition
  } = data;
  
  const hasChildren = node.children.length > 0;

  return (
    <div className="relative text-left w-[260px]">
      <DepartmentNodeHandles
        hasSource={hasSource}
        sourcePosition={sourcePosition}
        targetPosition={targetPosition}
      />

      <div
        aria-expanded={hasChildren ? !isCollapsed : undefined}
        aria-selected={isMatch}
        className={cn(
          "bg-white dark:bg-slate-900 border p-5 rounded-3xl shadow-sm",
          "transition-all duration-300 relative overflow-hidden flex flex-col",
          "focus:outline-none focus-visible:ring-2 focus-visible:ring-primary-500",
          node.id === "virtual-root" ? "cursor-default" : "cursor-pointer",
          isMatch ? "ring-2 ring-primary-500 shadow-xl shadow-primary-500/10 border-primary-500" : "border-slate-100 dark:border-slate-800 hover:shadow-lg hover:border-slate-200 dark:hover:border-slate-700",
          isRelevant ? "opacity-100" : "opacity-40 hover:opacity-100"
        )}
        onClick={() => {
          if (node.id === "virtual-root") return;
          onClick(node.dept);
        }}
        onKeyDown={(e) => {
          if (node.id === "virtual-root") return;
          if (e.key === "Enter" || e.key === " ") {
            e.preventDefault();
            onClick(node.dept);
          }
        }}
        role="treeitem"
        tabIndex={0}
      >
        <div className={cn(
          "absolute top-0 left-0 right-0 h-1.5",
          isMatch ? "bg-primary-500 animate-pulse" : "bg-slate-200 dark:bg-slate-800"
        )} />

        <DepartmentNodeActions
          node={node}
          onDelete={onDelete}
          onEdit={onEdit}
          t={t}
        />

        <div className="flex items-center gap-3 mb-3 pr-6 mt-1">
          <div className="w-8 h-8 rounded-xl bg-primary-50 dark:bg-primary-950/50 flex items-center justify-center text-primary-500 dark:text-primary-400 shadow-inner">
            <Building2 size={16} />
          </div>
          <Typography className="font-bold text-slate-800 dark:text-slate-100 text-sm truncate uppercase tracking-tight" variant="h3">
            {node.name}
          </Typography>
        </div>

        <Typography className="text-[10px] text-slate-500 dark:text-slate-400 line-clamp-1 italic font-semibold mb-4 opacity-70" variant="p">
          {node.description ?? t("common.no_description")}
        </Typography>

        <DepartmentNodeMetrics
          manager={manager}
          node={node}
          t={t}
        />
      </div>

      {hasChildren ? (
        <Button
          aria-label={isCollapsed ? t("departments.expand_all") : t("departments.collapse_all")}
          className="
            absolute bottom-1.5 left-1/2 -translate-x-1/2 translate-y-1/2 z-20 
            h-11 w-11 min-w-0 rounded-full bg-white p-0 dark:bg-slate-800 border border-slate-200 dark:border-slate-700
            flex items-center justify-center text-slate-500 dark:text-slate-400 shadow-sm
            hover:border-primary-500 hover:text-primary-500 transition-all cursor-pointer focus:outline-none focus:ring-2 focus:ring-primary-500
          "
          onClick={(e) => {
            e.stopPropagation();
            onToggleCollapse(node.id);
          }}
          size="icon"
          variant="outline"
        >
          {isCollapsed ? <ChevronDown size={14} /> : <ChevronDown className="rotate-180 transition-transform duration-300" size={14} />}
        </Button>
      ) : null}
    </div>
  );
};
