import type { Node, Edge } from "@xyflow/react";

import type { TreeNode, NodeBuildContext } from "../department-org-types";

export const getVerticalHeight = (
  node: TreeNode,
  collapsedNodes: Record<string, boolean>,
  autoExpanded: Set<string>
): number => (node.children.length === 0 || (collapsedNodes[node.id] && !autoExpanded.has(node.id)))
  ? 220
  : 220 + node.children.reduce((sum, child) => sum + getVerticalHeight(child, collapsedNodes, autoExpanded), 0);

export const getOptimalColumns = (numChildren: number, adjustment = 0): number => {
  if (numChildren <= 4) return 1;
  const maxCols = Math.min(12, numChildren);
  let bestC = 2, bestDiff = Infinity;
  for (let c = 2; c <= maxCols; c++) {
    const diff = Math.abs((c * 300) / (Math.ceil(numChildren / c) * 220) - 1.5);
    if (diff < bestDiff) [bestC, bestDiff] = [c, diff];
  }
  return Math.max(2, Math.min(maxCols, bestC + adjustment));
};

export const measureLayout = (
  treeRoots: TreeNode[],
  collapsedNodes: Record<string, boolean>,
  autoExpanded: Set<string>,
  adjustment: number
) => {
  const tempSubtreeWidths: Record<string, number> = {};
  let maxNodeY = 0;

  const getWidth = (node: TreeNode, depth = 0, isParentVertical = false): number => {
    const isVert = depth >= 2 || isParentVertical;
    const isCol = collapsedNodes[node.id] && !autoExpanded.has(node.id);
    const hasNoChildren = node.children.length === 0;

    if (hasNoChildren || isCol) {
      tempSubtreeWidths[node.id] = 300;
      return 300;
    }

    if (isVert) {
      tempSubtreeWidths[node.id] = 300;
      node.children.forEach((child) => { getWidth(child, depth + 1, true); });
      return 300;
    }

    const cols = getOptimalColumns(node.children.length, adjustment);
    if (cols > 1) {
      tempSubtreeWidths[node.id] = cols * 300;
      node.children.forEach((child) => { getWidth(child, depth + 1, false); });
      return tempSubtreeWidths[node.id];
    }

    const totalWidth = node.children.reduce((sum, c) => sum + getWidth(c, depth + 1, false), 0) + (node.children.length - 1) * 40;
    tempSubtreeWidths[node.id] = Math.max(300, totalWidth);
    return tempSubtreeWidths[node.id];
  };

  const positionNodeTemp = (node: TreeNode, y: number, depth = 0, isVertical = false) => {
    const isVerticalLayout = depth >= 2 || isVertical;
    const isCollapsed = collapsedNodes[node.id] && !autoExpanded.has(node.id);
    maxNodeY = Math.max(maxNodeY, y + 220);
    if (node.children.length === 0 || isCollapsed) return;
    if (isVerticalLayout) {
      let nextY = y + 220;
      node.children.forEach((child) => {
        positionNodeTemp(child, nextY, depth + 1, true);
        nextY += getVerticalHeight(child, collapsedNodes, autoExpanded);
      });
    } else {
      const numColumns = getOptimalColumns(node.children.length, adjustment);
      if (numColumns > 1) {
        const columns: TreeNode[][] = Array.from({ length: numColumns }, () => []);
        node.children.forEach((child, idx) => { columns[idx % numColumns].push(child); });
        columns.forEach((columnNodes) => {
          let colY = y + 260;
          columnNodes.forEach((child) => {
            positionNodeTemp(child, colY, depth + 1, true);
            colY += getVerticalHeight(child, collapsedNodes, autoExpanded);
          });
        });
      } else {
        node.children.forEach((child) => { positionNodeTemp(child, y + 260, depth + 1, false); });
      }
    }
  };

  let totalRootsWidth = 0;
  treeRoots.forEach(root => {
    totalRootsWidth += getWidth(root, 0, false);
  });
  totalRootsWidth += (treeRoots.length - 1) * 60;

  treeRoots.forEach(root => {
    positionNodeTemp(root, 0, 0, false);
  });

  return {
    height: Math.max(220, maxNodeY),
    subtreeWidths: tempSubtreeWidths,
    width: Math.max(300, totalRootsWidth),
  };
};

export const optimizeLayout = (
  treeRoots: TreeNode[],
  collapsedNodes: Record<string, boolean>,
  autoExpanded: Set<string>
) => {
  const defaultLayout = measureLayout(treeRoots, collapsedNodes, autoExpanded, 0);
  const defaultRatio = defaultLayout.width / defaultLayout.height;
  let bestAdjustment = 0;
  let finalSubtreeWidths = defaultLayout.subtreeWidths;

  const tryAdjustment = (adj: number) => {
    const layout = measureLayout(treeRoots, collapsedNodes, autoExpanded, adj);
    if (Math.abs(layout.width / layout.height - 1.5) < Math.abs(defaultRatio - 1.5)) {
      bestAdjustment = adj;
      finalSubtreeWidths = layout.subtreeWidths;
    }
  };

  if (defaultRatio < 1.3) tryAdjustment(1);
  else if (defaultRatio > 1.8) tryAdjustment(-1);

  return { bestAdjustment, finalSubtreeWidths };
};

interface BuildNodeParams {
  context: NodeBuildContext;
  isVert: boolean;
  node: TreeNode;
  targetPos: "top" | "left";
  x: number;
  y: number;
}

export const buildNode = ({
  context,
  isVert,
  node,
  targetPos,
  x,
  y
}: BuildNodeParams): Node => {
  const isCollapsedNode = context.collapsedNodes[node.id] && !context.autoExpanded.has(node.id);
  const hasChildrenNode = node.children.length > 0 && !isCollapsedNode;
  
  return {
    data: {
      hasSource: hasChildrenNode,
      isCollapsed: isCollapsedNode,
      isMatch: context.matchedIds.has(node.id),
      isRelevant: !context.searchActive || context.matchedIds.has(node.id) || context.hasMatchingDescendants[node.id],
      manager: context.employees.find(e => e.id === node.manager_id),
      node,
      onClick: context.onClick,
      onDelete: context.onDelete,
      onEdit: context.onEdit,
      onToggleCollapse: (id: string) => {
        context.setCollapsedNodes(prev => ({ ...prev, [id]: !prev[id] }));
      },
      sourcePosition: isVert ? "left" : "bottom",
      t: context.t,
      targetPosition: targetPos
    },
    id: node.id,
    position: { x, y },
    type: "department"
  };
};

interface BuildEdgeParams {
  childId: string;
  parentId: string;
  sourceHandle: string;
  targetPosition: "top" | "left";
}

export const buildEdge = ({
  childId,
  parentId,
  sourceHandle,
  targetPosition
}: BuildEdgeParams): Edge => {
  return {
    ariaLabel: "Đường nối sơ đồ tổ chức",
    id: `edge-${parentId}-${childId}`,
    source: parentId,
    sourceHandle,
    style: { stroke: "#cbd5e1", strokeWidth: 2 },
    target: childId,
    targetHandle: targetPosition === "left" ? "left-target" : "top-target",
    type: "ortho"
  };
};
