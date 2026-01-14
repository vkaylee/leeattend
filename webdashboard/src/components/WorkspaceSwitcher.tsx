import { useState, useEffect } from "react";
import { Briefcase, Plus } from "lucide-react";
import {
  Workspace,
  getCurrentWorkspaceId,
  setCurrentWorkspaceId,
} from "@/lib/api";
import CreateWorkspaceDialog from "./CreateWorkspaceDialog";
import { useWorkspaces } from "@/hooks/use-workspace";
import { Combobox } from "./ui/Combobox";

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

  function handleSwitch(wsId: string) {
    if (wsId === "ADD_NEW") {
      setIsCreateOpen(true);
      return;
    }
    const ws = workspaces.find(w => w.id === wsId);
    if (ws) {
      setActiveWorkspace(ws);
      setCurrentWorkspaceId(ws.id);
      // Reload page to refresh all data with new workspace header
      window.location.reload();
    }
  }

  if (isLoading && workspaces.length === 0) {
    return (
      <div className="w-full h-14 bg-slate-50 animate-pulse rounded-2xl" />
    );
  }

    const options = [

      ...workspaces.map((ws) => ({

        value: ws.id,

        label: ws.name,

        icon: (

          <div className="bg-violet-50 p-1.5 rounded-lg border border-violet-100">

            <Briefcase className="w-3.5 h-3.5 text-violet-600" />

          </div>

        ),

        description: ws.id.substring(0, 8)

      })),

      {

        value: "ADD_NEW",

        label: "Tạo Workspace",

        icon: (

          <div className="bg-slate-50 p-1.5 rounded-lg border border-slate-100">

            <Plus className="w-3.5 h-3.5 text-slate-500" />

          </div>

        ),

        description: "Thêm văn phòng/chi nhánh"

      }

    ];

  

    return (

      <div className="relative">

        <Combobox

          options={options}

          value={activeWorkspace?.id}

          onChange={handleSwitch}

          placeholder="Chọn Workspace"

          searchPlaceholder="Tìm kiếm..."

          className="h-10 bg-slate-50/50 border-slate-100 hover:bg-white hover:border-violet-200 transition-all rounded-xl text-sm font-semibold text-slate-700"

        />

  

        <CreateWorkspaceDialog

          isOpen={isCreateOpen}

          onClose={() => setIsCreateOpen(false)}

        />

      </div>

    );

  }

  