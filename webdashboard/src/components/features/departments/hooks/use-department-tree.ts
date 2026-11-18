import type { TFunction } from "i18next";

import { useMemo } from "react";

import type { Department } from "@/lib/api";

import type { DepartmentWithParent, TreeNode } from "../department-org-types";

interface UseDepartmentTreeOptions {
  departments: Department[];
  searchQuery: string;
  t: TFunction;
}

export function useDepartmentTree({
  departments,
  searchQuery,
  t,}: UseDepartmentTreeOptions) {
  // Map departments to include parent_id from localStorage
  const departmentsWithParent = useMemo<DepartmentWithParent[]>(() => {
    let parentMap: Record<string, string | null> = {};
    try {
      const saved = localStorage.getItem("leeattend_dept_parents");
      if (saved) {
        parentMap = JSON.parse(saved) as Record<string, string | null>;
      }
    } catch { /* localStorage unavailable or corrupt — use empty parent map */
      parentMap = {};
    }

    const activeParentMap = { ...parentMap };

    return departments.map(d => ({
      ...d,
      description: d.description ?? null,
      manager_id: d.manager_id ?? null,
      parent_id: activeParentMap[d.id] ?? null
    }));
  }, [departments]);

  // Parse Flat array to Hierarchical Tree
  const treeRoots = useMemo<TreeNode[]>(() => {
    const nodeMap = new Map<string, TreeNode>();

    // First pass: create node wrappers
    departmentsWithParent.forEach(dept => {
      nodeMap.set(dept.id, {
        active_employee_count: dept.active_employee_count,
        children: [],
        dept,
        description: dept.description,
        id: dept.id,
        manager_id: dept.manager_id,
        name: dept.name,
      });
    });

    const roots: TreeNode[] = [];

    // Second pass: link parent/child relations
    departmentsWithParent.forEach(dept => {
      const node = nodeMap.get(dept.id);
      if (!node) return;
      if (dept.parent_id && nodeMap.has(dept.parent_id)) {
        const parentNode = nodeMap.get(dept.parent_id);
        if (parentNode) {
          parentNode.children.push(node);
        }
      } else {
        roots.push(node);
      }
    });

    // If there are multiple root departments, group them under a single virtual root (representing the Company)
    if (roots.length > 1) {
      const totalEmployees = departments.reduce((sum, d) => sum + d.active_employee_count, 0);
      const virtualRoot: TreeNode = {
        active_employee_count: totalEmployees,
        children: roots,
        dept: {
          active_employee_count: totalEmployees,
          created_at: new Date().toISOString(),
          id: "virtual-root",
          manager_id: null,
          name: t("departments.virtual_root_name", { defaultValue: "ORGANIZATION" }),
          updated_at: new Date().toISOString(),
          workspace_id: departments[0]?.workspace_id ?? "",
        },
        description: t("departments.virtual_root_description", { defaultValue: "Sơ đồ phòng ban doanh nghiệp" }),
        id: "virtual-root",
        manager_id: null,
        name: t("departments.virtual_root_name", { defaultValue: "ORGANIZATION" }),
      };
      return [virtualRoot];
    }

    return roots;
  }, [departmentsWithParent,departments, t]);

  // Handle Search Auto-Expand Path & Match Highlighting
  const { autoExpanded, matchedIds, searchActive } = useMemo(() => {
    const expanded = new Set<string>();
    const matches = new Set<string>();
    const active = searchQuery.trim().length > 0;

    if (active) {
      const query = searchQuery.toLowerCase();
      departmentsWithParent.forEach(dept => {
        const isMatch = 
          dept.name.toLowerCase().includes(query) ||
          (dept.description?.toLowerCase().includes(query) ?? false);

        if (isMatch) {
          matches.add(dept.id);
          // Auto-expand all ancestor nodes
          let parentId = dept.parent_id;
          while (parentId) {
            expanded.add(parentId);
            const parent = departmentsWithParent.find(d => d.id === parentId);
            parentId = parent ? parent.parent_id : null;
          }
        }
      });
    }

    return { autoExpanded: expanded, matchedIds: matches, searchActive: active };
  }, [departmentsWithParent, searchQuery]);

  // Check if a specific node contains matches in its descendants
  const hasMatchingDescendants = useMemo<Record<string, boolean>>(() => {
    const cache: Record<string, boolean> = {};

    const check = (node: TreeNode): boolean => {
      if (node.id in cache) return cache[node.id];
      
      const hasDirectMatch = matchedIds.has(node.id);
      const hasChildMatch = node.children.some(child => check(child));
      
      const result = hasDirectMatch || hasChildMatch;
      cache[node.id] = result;
      return result;
    };

    treeRoots.forEach(r => check(r));
    return cache;
  }, [treeRoots, matchedIds]);

  return {
    autoExpanded,
    hasMatchingDescendants,
    matchedIds,
    searchActive,
    treeRoots,
  };
}
