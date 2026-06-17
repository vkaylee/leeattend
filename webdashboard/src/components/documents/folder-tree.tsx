import { ChevronRight, ChevronDown, Folder as FolderIcon, MoreVertical, Plus, Edit2, Trash2 } from "lucide-react";
import React, { useMemo, useState } from "react";

import Button from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { cn } from "@/lib/utils";
import { components } from "@/types/api-generated";

type DocumentFolderDto = components["schemas"]["DocumentFolderDto"];

export type FolderNode = DocumentFolderDto & {
  children: FolderNode[];
};

export interface FolderTreeProps {
  folders: DocumentFolderDto[];
  onCreateFolder?: (parentId: string | null) => void;
  onDeleteFolder?: (folderId: string) => void;
  onRenameFolder?: (folder: DocumentFolderDto) => void;
  onSelectFolder: (id: string | null) => void;
  selectedFolderId: string | null;
}

export function FolderTree({
  folders,
  selectedFolderId,
  onSelectFolder,
  onCreateFolder,
  onRenameFolder,
  onDeleteFolder,
}: FolderTreeProps) {
  const tree = useMemo(() => {
    const map = new Map<string, FolderNode>();
    const roots: FolderNode[] = [];

    // Initialize nodes
    folders.forEach((f) => {
      map.set(f.id, { ...f, children: [] });
    });

    // Build tree
    folders.forEach((f) => {
      const node = map.get(f.id)!;
      if (f.parent_folder_id && map.has(f.parent_folder_id)) {
        map.get(f.parent_folder_id)!.children.push(node);
      } else {
        roots.push(node);
      }
    });

    // Sort alphabetically
    const sortNodes = (nodes: FolderNode[]) => {
      nodes.sort((a, b) => a.name.localeCompare(b.name));
      nodes.forEach((n) => { sortNodes(n.children); });
    };
    sortNodes(roots);

    return roots;
  }, [folders]);

  return (
    <div className="flex flex-col gap-2 w-full">
      <div
        className="flex items-center justify-between px-3 py-2 group rounded-md hover:bg-slate-100 dark:hover:bg-slate-800/50 cursor-pointer focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-primary-500"
        onClick={() => { onSelectFolder(null); }}
        onKeyDown={(e) => {
          if (e.key === "Enter" || e.key === " ") {
            e.preventDefault();
            onSelectFolder(null);
          }
        }}
        role="button"
        tabIndex={0}
      >
        <div className="flex items-center gap-2 text-sm font-semibold text-slate-700 dark:text-slate-300">
          <FolderIcon className="w-4 h-4 text-primary-500" />
          <span>All Documents</span>
        </div>
        {!!onCreateFolder && (
          <Button
            aria-label="Create root folder"
            className="h-6 w-6 opacity-0 group-hover:opacity-100 transition-opacity"
            onClick={(e: React.MouseEvent) => {
              e.stopPropagation();
              onCreateFolder(null);
            }}
            size="icon"
            variant="ghost"
          >
            <Plus className="w-3 h-3" />
          </Button>
        )}
      </div>
      
      <div className="pl-2 border-l border-slate-200 dark:border-slate-800 ml-4 flex flex-col gap-1">
        {tree.map((node) => (
          <FolderTreeNode
            key={node.id}
            level={0}
            node={node}
            onCreateFolder={onCreateFolder}
            onDeleteFolder={onDeleteFolder}
            onRenameFolder={onRenameFolder}
            onSelectFolder={onSelectFolder}
            selectedFolderId={selectedFolderId}
          />
        ))}
      </div>
    </div>
  );
}

interface FolderTreeNodeProps {
  level: number;
  node: FolderNode;
  onCreateFolder?: (parentId: string | null) => void;
  onDeleteFolder?: (folderId: string) => void;
  onRenameFolder?: (folder: DocumentFolderDto) => void;
  onSelectFolder: (id: string | null) => void;
  selectedFolderId: string | null;
}

function FolderTreeNode({
  node,
  level,
  selectedFolderId,
  onSelectFolder,
  onCreateFolder,
  onRenameFolder,
  onDeleteFolder,
}: FolderTreeNodeProps) {
  const [isExpanded, setIsExpanded] = useState(false);
  const isSelected = selectedFolderId === node.id;
  const hasChildren = node.children.length > 0;

  // Auto expand if selected or child selected (naive)
  // In a real app, we might want to deeply check if a child is selected

  return (
    <div className="flex flex-col w-full">
      <div
        className={cn(
          "flex items-center justify-between py-1.5 px-2 rounded-md cursor-pointer group text-sm transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-primary-500",
          isSelected
            ? "bg-primary-50 dark:bg-primary-900/20 text-primary-700 dark:text-primary-300 font-medium"
            : "text-slate-600 dark:text-slate-400 hover:bg-slate-50 dark:hover:bg-slate-800/50"
        )}
        onClick={() => { onSelectFolder(node.id); }}
        onKeyDown={(e) => {
          if (e.key === "Enter" || e.key === " ") {
            e.preventDefault();
            onSelectFolder(node.id);
          }
        }}
        role="button"
        tabIndex={0}
      >
        <div className="flex items-center gap-1.5 overflow-hidden">
          {hasChildren ? (
            <Button
              aria-label={isExpanded ? "Collapse folder" : "Expand folder"}
              className="w-4 h-4 flex items-center justify-center cursor-pointer shrink-0 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-primary-500 rounded hover:bg-slate-200 dark:hover:bg-slate-700 h-auto p-0 shadow-none border border-transparent"
              onClick={(e: React.MouseEvent) => {
                e.stopPropagation();
                setIsExpanded(!isExpanded);
              }}
              size="icon"
              type="button"
              variant="ghost"
            >
              {isExpanded ? (
                <ChevronDown className="w-3 h-3 text-slate-400" />
              ) : (
                <ChevronRight className="w-3 h-3 text-slate-400" />
              )}
            </Button>
          ) : (
            <div className="w-4 h-4 flex items-center justify-center shrink-0">
              <span className="w-3 h-3" />
            </div>
          )}
          <FolderIcon
            className={cn(
              "w-4 h-4 shrink-0",
              isSelected ? "text-primary-500" : "text-slate-400 dark:text-slate-500"
            )}
          />
          <span className="truncate">{node.name}</span>
        </div>

        <div className="flex items-center opacity-0 group-hover:opacity-100 transition-opacity">
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button aria-label="Folder actions" className="h-6 w-6" size="icon" variant="ghost">
                <MoreVertical className="w-3 h-3" />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end" className="w-40">
              <DropdownMenuItem
                onClick={(e: React.MouseEvent) => {
                  e.stopPropagation();
                  onCreateFolder?.(node.id);
                }}
              >
                <Plus className="mr-2 h-4 w-4" />
                <span>New Subfolder</span>
              </DropdownMenuItem>
              <DropdownMenuItem
                onClick={(e: React.MouseEvent) => {
                  e.stopPropagation();
                  onRenameFolder?.(node);
                }}
              >
                <Edit2 className="mr-2 h-4 w-4" />
                <span>Rename</span>
              </DropdownMenuItem>
              <DropdownMenuSeparator />
              <DropdownMenuItem
                className="text-red-600 dark:text-red-400"
                onClick={(e: React.MouseEvent) => {
                  e.stopPropagation();
                  onDeleteFolder?.(node.id);
                }}
              >
                <Trash2 className="mr-2 h-4 w-4" />
                <span>Delete</span>
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      </div>

      {isExpanded && hasChildren ? (
        <div className="pl-4 ml-3 border-l border-slate-100 dark:border-slate-800 flex flex-col gap-0.5 mt-0.5">
          {node.children.map((child) => (
            <FolderTreeNode
              key={child.id}
              level={level + 1}
              node={child}
              onCreateFolder={onCreateFolder}
              onDeleteFolder={onDeleteFolder}
              onRenameFolder={onRenameFolder}
              onSelectFolder={onSelectFolder}
              selectedFolderId={selectedFolderId}
            />
          ))}
        </div>
      ) : null}
    </div>
  );
}
