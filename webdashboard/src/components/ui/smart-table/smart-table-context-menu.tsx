import React from "react";

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { cn } from "@/lib/utils";

import type { ContextMenuItem } from "../smart-table";

interface Props<T> {
  contextMenu: { x: number; y: number; row: T } | null;
  contextMenuItems?: (row: T) => ContextMenuItem[];
  setContextMenu: (v: null) => void;
}

export default function SmartTableContextMenu<T>({ contextMenu, contextMenuItems, setContextMenu }: Props<T>) {
  if (!contextMenu) return null;

  return (
    <DropdownMenu
      onOpenChange={(open: boolean) => {
        if (!open) setContextMenu(null);
      }}
      open
    >
      <DropdownMenuTrigger asChild>
        <div
          style={{
            position: "fixed",
            left: contextMenu.x,
            top: contextMenu.y,
            width: 1,
            height: 1,
          }}
        />
      </DropdownMenuTrigger>
      <DropdownMenuContent className="w-48 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 shadow-xl rounded-2xl p-1.5">
        {contextMenuItems?.(contextMenu.row).map((item: ContextMenuItem, idx: number) => (
          <DropdownMenuItem
            className={cn(
              "flex items-center gap-2 px-3 py-2 text-xs font-bold uppercase tracking-tight rounded-xl cursor-pointer hover:bg-slate-100 dark:hover:bg-slate-800",
              item.disabled && "opacity-50 pointer-events-none",
            )}
            disabled={item.disabled}
            key={idx}
            onClick={() => {
              item.onClick();
              setContextMenu(null);
            }}
          >
            {item.icon}
            <span>{item.label}</span>
          </DropdownMenuItem>
        ))}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
