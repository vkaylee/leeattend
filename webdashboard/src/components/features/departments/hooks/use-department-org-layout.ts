import type { Edge, Node } from "@xyflow/react";

import { useMemo } from "react";

import type { NodeBuildContext, PositionOptions, TreeNode } from "../department-org-types";

import {
  buildEdge,
  buildNode,
  getOptimalColumns,
  getVerticalHeight,
  optimizeLayout,
} from "./department-layout-helpers";

interface UseDepartmentOrgLayoutOptions extends NodeBuildContext {
  treeRoots: TreeNode[];
}

interface LayoutContext {
  autoExpanded: Set<string>;
  bestAdjustment: number;
  buildContext: NodeBuildContext;
  collapsedNodes: Record<string, boolean>;
  edges: Edge[];
  finalSubtreeWidths: Record<string, number>;
  nodes: Node[];
}

interface ChildPositionOptions {
  depth: number;
  layoutContext: LayoutContext;
  node: TreeNode;
  parentSourceHandle: string;
  startX: number;
  y: number;
}

interface PositionNodeParams {
  layoutContext: LayoutContext;
  node: TreeNode;
  options: PositionOptions;
  startX: number;
  y: number;
}

const positionVerticalChildren = ({ node, startX, y, parentSourceHandle, depth, layoutContext }: ChildPositionOptions) => {
  let nextY = y + 220;
  node.children.forEach((child) => {
    positionNode({
      layoutContext,
      node: child,
      options: {
        depth: depth + 1,
        isVertical: true,
        parentId: node.id,
        parentSourceHandle,
        targetPosition: "left"
      },
      startX: startX + 48,
      y: nextY
    });
    nextY += getVerticalHeight(child, layoutContext.collapsedNodes, layoutContext.autoExpanded);
  });
};

const positionGridChildren = ({ node, startX, y, parentSourceHandle, depth, layoutContext }: ChildPositionOptions) => {
  const numColumns = getOptimalColumns(node.children.length, layoutContext.bestAdjustment);
  const columns: TreeNode[][] = Array.from({ length: numColumns }, () => []);
  node.children.forEach((child, idx) => columns[idx % numColumns].push(child));

  columns.forEach((columnNodes, colIdx) => {
    const colStartX = startX + colIdx * 300;
    let colY = y + 260;
    columnNodes.forEach((child) => {
      positionNode({
        layoutContext,
        node: child,
        options: {
          depth: depth + 1,
          isVertical: true,
          parentId: node.id,
          parentSourceHandle,
          targetPosition: "left"
        },
        startX: colStartX,
        y: colY
      });
      colY += getVerticalHeight(child, layoutContext.collapsedNodes, layoutContext.autoExpanded);
    });
  });
};

const positionHorizontalChildren = ({ node, startX, y, parentSourceHandle, depth, layoutContext }: ChildPositionOptions) => {
  let nextStartX = startX;
  node.children.forEach(child => {
    positionNode({
      layoutContext,
      node: child,
      options: {
        depth: depth + 1,
        isVertical: false,
        parentId: node.id,
        parentSourceHandle,
        targetPosition: "top"
      },
      startX: nextStartX,
      y: y + 260
    });
    nextStartX += layoutContext.finalSubtreeWidths[child.id] + 40;
  });
};

const positionNode = ({
  layoutContext,
  node,
  options,
  startX,
  y
}: PositionNodeParams) => {
  const isCollapsed = layoutContext.collapsedNodes[node.id] && !layoutContext.autoExpanded.has(node.id);
  const width = layoutContext.finalSubtreeWidths[node.id];
  const currentX = startX + width / 2;

  layoutContext.nodes.push(buildNode({
    context: layoutContext.buildContext,
    isVert: options.isVertical,
    node,
    targetPos: options.targetPosition,
    x: currentX - 130,
    y
  }));

  if (options.parentId) {
    layoutContext.edges.push(buildEdge({
      childId: node.id,
      parentId: options.parentId,
      sourceHandle: options.parentSourceHandle,
      targetPosition: options.targetPosition
    }));
  }

  if (node.children.length === 0 || isCollapsed) return;

  const isVerticalLayout = options.depth >= 2 || options.isVertical;
  const childOpts = {
    depth: options.depth,
    layoutContext,
    node,
    parentSourceHandle: options.isVertical ? "left-source" : "bottom-source",
    startX,
    y
  };

  if (isVerticalLayout) {
    positionVerticalChildren(childOpts);
  } else {
    const numColumns = getOptimalColumns(node.children.length, layoutContext.bestAdjustment);
    if (numColumns > 1) {
      positionGridChildren(childOpts);
    } else {
      positionHorizontalChildren(childOpts);
    }
  }
};

export function useDepartmentOrgLayout({
  autoExpanded,
  collapsedNodes,
  employees,
  hasMatchingDescendants,
  matchedIds,
  onClick,
  onDelete,
  onEdit,
  searchActive,
  setCollapsedNodes,
  t,
  treeRoots,
}: UseDepartmentOrgLayoutOptions) {
  const buildContext = useMemo(() => ({
    autoExpanded, collapsedNodes, employees, hasMatchingDescendants, matchedIds,
    onClick, onDelete, onEdit, searchActive, setCollapsedNodes, t
  }), [autoExpanded, collapsedNodes, employees, hasMatchingDescendants, matchedIds, onClick, onDelete, onEdit, searchActive, setCollapsedNodes, t]);

  return useMemo(() => {
    const nodes: Node[] = [];
    const edges: Edge[] = [];

    const { bestAdjustment, finalSubtreeWidths } = optimizeLayout(treeRoots, collapsedNodes, autoExpanded);

    const totalRootsWidth = treeRoots.reduce((sum, root) => sum + finalSubtreeWidths[root.id], 0) + (treeRoots.length - 1) * 60;

    const layoutContext: LayoutContext = {
      autoExpanded, bestAdjustment, buildContext, collapsedNodes, edges, finalSubtreeWidths, nodes
    };

    let rootStartX = -totalRootsWidth / 2;
    treeRoots.forEach(root => {
      positionNode({
        layoutContext,
        node: root,
        options: {
          depth: 0,
          isVertical: false,
          parentId: null,
          parentSourceHandle: "bottom-source",
          targetPosition: "top"
        },
        startX: rootStartX,
        y: 0
      });
      rootStartX += finalSubtreeWidths[root.id] + 60;
    });

    return { rfEdges: edges, rfNodes: nodes };
  }, [treeRoots, collapsedNodes, autoExpanded, buildContext]);
}
