import type { TFunction } from "i18next";

import type { Department, Employee } from "@/lib/api";

export interface TreeNode {
  active_employee_count: number;
  children: TreeNode[];
  dept: Department;
  description: string | null;
  id: string;
  manager_id: string | null;
  name: string;
}

export type DepartmentWithParent = Department & {
  description: string | null;
  manager_id: string | null;
  parent_id: string | null;
};

export interface DepartmentNodeData {
  hasSource: boolean;
  isCollapsed: boolean;
  isMatch: boolean;
  isRelevant: boolean;
  manager: Employee | undefined;
  node: TreeNode;
  onClick: (dept: Department) => void;
  onConfig: (dept: Department) => void;
  onDelete: (dept: Department) => void;
  onEdit: (dept: Department) => void;
  onToggleCollapse: (id: string) => void;
  sourcePosition: "bottom" | "left";
  t: TFunction;
  targetPosition: "top" | "left";
}

export interface NodeBuildContext {
  autoExpanded: Set<string>;
  collapsedNodes: Record<string, boolean>;
  employees: Employee[];
  hasMatchingDescendants: Record<string, boolean>;
  matchedIds: Set<string>;
  onClick: (dept: Department) => void;
  onConfig: (dept: Department) => void;
  onDelete: (dept: Department) => void;
  onEdit: (dept: Department) => void;
  searchActive: boolean;
  setCollapsedNodes: React.Dispatch<React.SetStateAction<Record<string, boolean>>>;
  t: TFunction;
}

export interface PositionOptions {
  depth: number;
  isVertical: boolean;
  parentId: string | null;
  parentSourceHandle: string;
  targetPosition: "top" | "left";
}
