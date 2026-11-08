import type { TFunction } from "i18next";

import {
  Background,
  Controls,
  MiniMap,
  ReactFlow,
  type Edge,
  type Node
} from "@xyflow/react";
import { Maximize, Minimize } from "lucide-react";
import React, { useEffect, useState } from "react";

import type { Department, Employee } from "@/lib/api";

import Button from "@/components/ui/button";
import { cn } from "@/lib/utils";

import type { TreeNode } from "./department-org-types";

import { OrthoEdge } from "./department-org-edge";
import { DepartmentOrgMobileNode } from "./department-org-mobile-node";
import { DepartmentNode } from "./department-org-node";
import { useDepartmentOrgLayout } from "./hooks/use-department-org-layout";
import { useDepartmentTree } from "./hooks/use-department-tree";

import "@xyflow/react/dist/style.css";

export interface DepartmentOrgChartProps {
  departments: Department[];
  employees: Employee[];
  onClick: (dept: Department) => void;
  onConfig: (dept: Department) => void;
  onDelete: (dept: Department) => void;
  onEdit: (dept: Department) => void;
  searchQuery: string;
  t: TFunction;
}

interface DepartmentOrgChartControlsProps {
  isFullscreen: boolean;
  onCollapseAll: () => void;
  onExpandAll: () => void;
  onToggleFullscreen: () => void;
  t: TFunction;
}

const DepartmentOrgChartControls = ({
  isFullscreen,
  onCollapseAll,
  onExpandAll,
  onToggleFullscreen,
  t
}: DepartmentOrgChartControlsProps) => {
  return (
    <div className="hidden md:flex absolute top-6 right-6 z-30 items-center gap-3 transition-all duration-300">
      <div className="flex items-center gap-1.5 bg-white/85 dark:bg-slate-900/85 backdrop-blur-md border border-slate-200/50 dark:border-slate-800/50 p-1.5 rounded-2xl shadow-xl shadow-slate-100/50 dark:shadow-black/40">
        <Button
          aria-label={isFullscreen ? t("common.exit_fullscreen", { defaultValue: "Exit Fullscreen" }) : t("common.fullscreen", { defaultValue: "Fullscreen" })}
          className="h-8 w-8 px-0 rounded-xl hover:bg-slate-100/50 dark:hover:bg-slate-800/50"
          icon={isFullscreen ? <Minimize size={14} /> : <Maximize size={14} />}
          onClick={onToggleFullscreen}
          size="sm"
          variant="ghost"
        />
      </div>

      <div className="flex items-center gap-2 bg-white/85 dark:bg-slate-900/85 backdrop-blur-md border border-slate-200/50 dark:border-slate-800/50 p-1.5 rounded-2xl shadow-xl shadow-slate-100/50 dark:shadow-black/40">
        <Button
          className="h-8 px-3 rounded-xl text-[9px] font-bold tracking-wider hover:bg-slate-100/50 dark:hover:bg-slate-800/50"
          onClick={onExpandAll}
          size="sm"
          variant="ghost"
        >
          {t("departments.expand_all").toUpperCase()}
        </Button>
        <Button
          className="h-8 px-3 rounded-xl text-[9px] font-bold tracking-wider hover:bg-slate-100/50 dark:hover:bg-slate-800/50"
          onClick={onCollapseAll}
          size="sm"
          variant="ghost"
        >
          {t("departments.collapse_all").toUpperCase()}
        </Button>
      </div>
    </div>
  );
};

const nodeTypes = {
  department: DepartmentNode
};

const edgeTypes = {
  ortho: OrthoEdge
};

interface DepartmentOrgDesktopChartProps {
  rfEdges: Edge[];
  rfNodes: Node[];
  t: TFunction;
  treeRoots: TreeNode[];
}

const DepartmentOrgDesktopChart = ({
  rfEdges,
  rfNodes,
  t,
  treeRoots
}: DepartmentOrgDesktopChartProps) => {
  if (treeRoots.length === 0) {
    return (
      <div className="p-12 text-center text-slate-500 font-bold tracking-wider uppercase text-xs">
        {t("departments.messages.no_departments")}
      </div>
    );
  }

  return (
    <ReactFlow
      edges={rfEdges}
      edgesFocusable={false}
      edgeTypes={edgeTypes}
      fitView
      fitViewOptions={{ padding: 0.1 }}
      maxZoom={1.5}
      minZoom={0.2}
      nodes={rfNodes}
      nodesConnectable={false}
      nodesDraggable={true}
      nodesFocusable={false}
      nodeTypes={nodeTypes}
      proOptions={{ hideAttribution: true }}
    >
      <Background color="#cbd5e1" gap={24} size={1.5} />
      <Controls className="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-xl" showInteractive={false} />
      <MiniMap 
        className="bg-white/80 dark:bg-slate-900/80 backdrop-blur border border-slate-200 dark:border-slate-800 rounded-2xl hidden lg:block"
        maskColor="rgba(0, 0, 0, 0.05)"
        nodeColor={() => "var(--color-primary-500, #3b82f6)"}
      />
    </ReactFlow>
  );
};

export function DepartmentOrgChart({
  departments,
  employees,
  onClick,
  onConfig,
  onDelete,
  onEdit,
  searchQuery,
  t}: DepartmentOrgChartProps) {
  const [isFullscreen, setIsFullscreen] = useState(false);
  const [collapsedNodes, setCollapsedNodes] = useState<Record<string, boolean>>({});

  const {
    autoExpanded,
    hasMatchingDescendants,
    matchedIds,
    searchActive,
    treeRoots
  } = useDepartmentTree({
    departments,
    searchQuery,
    t});

  const handleExpandAll = () => {
    setCollapsedNodes({});
  };

  const handleCollapseAll = () => {
    const newCollapsed: Record<string, boolean> = {};
    const findNodesWithChildren = (nodes: TreeNode[]) => {
      nodes.forEach(node => {
        if (node.children.length > 0) {
          newCollapsed[node.id] = true;
          findNodesWithChildren(node.children);
        }
      });
    };
    findNodesWithChildren(treeRoots);
    setCollapsedNodes(newCollapsed);
  };

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape" && isFullscreen) {
        setIsFullscreen(false);
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [isFullscreen]);

  const { rfNodes, rfEdges } = useDepartmentOrgLayout({
    autoExpanded,
    collapsedNodes,
    employees,
    hasMatchingDescendants,
    matchedIds,
    onClick,
    onConfig,
    onDelete,
    onEdit,
    searchActive,
    setCollapsedNodes,
    t,
    treeRoots
  });

  const renderMobileNode = (node: TreeNode, depth = 0) => {
    const isExplicitlyCollapsed = collapsedNodes[node.id] ?? false;
    const isCollapsed = isExplicitlyCollapsed && !autoExpanded.has(node.id);
    const hasChildren = node.children.length > 0;
    const isMatch = matchedIds.has(node.id);
    const isRelevant = !searchActive || isMatch || hasMatchingDescendants[node.id];

    if (!isRelevant) return null;

    return (
      <React.Fragment key={node.id}>
        <DepartmentOrgMobileNode
          depth={depth}
          isCollapsed={isCollapsed}
          isExplicitlyCollapsed={isExplicitlyCollapsed}
          isMatch={isMatch}
          manager={employees.find(e => e.id === node.manager_id)}
          node={node}
          onClick={onClick}
          onConfig={onConfig}
          onDelete={onDelete}
          onEdit={onEdit}
          onToggleCollapse={(id, currentCollapsed) => {
            setCollapsedNodes(prev => ({
              ...prev,
              [id]: !currentCollapsed
            }));
          }}
          t={t}
        />
        {hasChildren && !isCollapsed ? node.children.map(child => renderMobileNode(child, depth + 1)) : null}
      </React.Fragment>
    );
  };

  return (
    <div
      className={cn(
        "bg-white dark:bg-slate-900 overflow-hidden flex flex-col relative transition-all duration-300",
        isFullscreen
          ? "fixed inset-0 z-[100] h-dvh w-dvw p-6"
          : "w-full h-full min-h-[500px]"
      )}
    >
      <DepartmentOrgChartControls
        isFullscreen={isFullscreen}
        onCollapseAll={handleCollapseAll}
        onExpandAll={handleExpandAll}
        onToggleFullscreen={() => { setIsFullscreen(prev => !prev); }}
        t={t}
      />

      <div className="hidden md:block flex-1 overflow-hidden bg-slate-50/35 dark:bg-slate-950/20 relative outline-none select-none">
        <DepartmentOrgDesktopChart
          rfEdges={rfEdges}
          rfNodes={rfNodes}
          t={t}
          treeRoots={treeRoots}
        />
      </div>

      <div className="md:hidden flex-1 overflow-y-auto divide-y divide-slate-100 dark:divide-slate-800/50">
        {treeRoots.length === 0 ? (
          <div className="p-12 text-center text-slate-500 font-bold tracking-wider uppercase text-xs">
            {t("departments.messages.no_departments")}
          </div>
        ) : (
          treeRoots.map(root => renderMobileNode(root, 0))
        )}
      </div>
    </div>
  );
}
