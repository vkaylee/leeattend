import { useState, useEffect } from "react";
import { ChevronDown, Briefcase, Plus, Check } from "lucide-react";
import {
  Workspace,
  getCurrentWorkspaceId,
  setCurrentWorkspaceId,
} from "@/lib/api";
import CreateWorkspaceDialog from "./CreateWorkspaceDialog";
import { useWorkspaces } from "@/hooks/use-workspace";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "./ui/dropdown-menu";

export default function WorkspaceSwitcher() {
  const { workspaces, isLoading } = useWorkspaces();
  const [activeWorkspace, setActiveWorkspace] = useState<Workspace | null>(
    null,
  );
  const [isCreateOpen, setIsCreateOpen] = useState(false);

  useEffect(() => {
    if (!isLoading && workspaces.length > 0) {
      const currentId = getCurrentWorkspaceId();
      const current =
        workspaces.find((ws) => ws.id === currentId) || workspaces[0];

      if (current) {
        setActiveWorkspace(current);
        if (!currentId) setCurrentWorkspaceId(current.id);
      }
    }
  }, [workspaces, isLoading]);

  function handleSwitch(ws: Workspace) {
    setActiveWorkspace(ws);
    setCurrentWorkspaceId(ws.id);
    // Reload page to refresh all data with new workspace header
    window.location.reload();
  }

  if (isLoading && workspaces.length === 0) {
    return (
      <div className="w-full h-12 bg-slate-50 animate-pulse rounded-2xl" />
    );
  }

  return (
    <div className="relative">
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <button className="w-full flex items-center justify-between gap-3 px-4 py-3 bg-slate-50 hover:bg-slate-100 rounded-2xl transition-all group border border-slate-100 outline-none focus:ring-2 focus:ring-blue-500/20">
            <div className="flex items-center gap-3 overflow-hidden">
              <div className="bg-white p-1.5 rounded-lg shadow-sm">
                <Briefcase className="w-4 h-4 text-blue-600" />
              </div>
              <span className="font-bold text-sm text-slate-700 truncate">
                {activeWorkspace?.name || "Chọn Workspace"}
              </span>
            </div>
            <ChevronDown className="w-4 h-4 text-slate-400 group-data-[state=open]:rotate-180 transition-transform" />
          </button>
        </DropdownMenuTrigger>

        <DropdownMenuContent className="w-[var(--radix-dropdown-menu-trigger-width)]">
          <DropdownMenuLabel>Workspaces của bạn</DropdownMenuLabel>
          <div className="max-h-60 overflow-y-auto">
            {workspaces.map((ws) => (
              <DropdownMenuItem
                key={ws.id}
                onSelect={() => handleSwitch(ws)}
                className="justify-between"
              >
                <span
                  className={`font-semibold ${ws.id === activeWorkspace?.id ? "text-blue-600" : "text-slate-600"}`}
                >
                  {ws.name}
                </span>
                {ws.id === activeWorkspace?.id && (
                  <Check className="w-4 h-4 text-blue-600" />
                )}
              </DropdownMenuItem>
            ))}
          </div>
          <DropdownMenuSeparator />
          <DropdownMenuItem
            onSelect={() => setIsCreateOpen(true)}
            className="text-slate-500 focus:text-blue-600 focus:bg-blue-50 font-bold gap-2"
          >
            <Plus className="w-4 h-4" />
            Thêm Workspace
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>

      <CreateWorkspaceDialog
        isOpen={isCreateOpen}
        onClose={() => setIsCreateOpen(false)}
      />
    </div>
  );
}
