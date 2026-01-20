import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
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
  const { t } = useTranslation();
  const { workspaces, isLoading } = useWorkspaces();
  const [activeWorkspace, setActiveWorkspace] = useState<Workspace | null>(
    null,
  );
  const [isCreateOpen, setIsCreateOpen] = useState(false);

  useEffect(() => {
    if (workspaces.length > 0) {
      const currentId = getCurrentWorkspaceId();
      const current = workspaces.find((ws) => ws.id === currentId) || workspaces[0];

      if (current) {
        const timer = setTimeout(() => {
          setActiveWorkspace(current);
          if (!currentId) setCurrentWorkspaceId(current.id);
        }, 0);
        return () => clearTimeout(timer);
      }
    }
  }, [workspaces]);

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
          <div className="bg-primary-50 p-2 rounded-xl border border-primary-100 shadow-inner">
            <Briefcase className="w-4 h-4 text-primary-600" />
          </div>
        ),
        description: (
          <div className="flex items-center gap-1.5 mt-0.5 opacity-60">
            <span className="text-[9px] font-black uppercase tracking-widest leading-none">ID:</span>
            <span className="text-[9px] font-mono font-bold leading-none">{ws.id.substring(0, 8)}</span>
          </div>
        )
      })),
      {
        value: "ADD_NEW",
        label: t("common.create_workspace"),
        icon: (
          <div className="bg-slate-50 p-2 rounded-xl border border-slate-100 shadow-inner">
            <Plus className="w-4 h-4 text-slate-400" />
          </div>
        ),
        description: (
          <p className="text-[9px] font-black text-slate-400 uppercase tracking-widest mt-0.5 italic">{t("common.add_office_branch")}</p>
        )
      }
    ];

    return (
      <div className="relative">
        <Combobox
          options={options}
          value={activeWorkspace?.id}
          onChange={handleSwitch}
          placeholder={t("common.select_workspace")}
          searchPlaceholder={t("common.search_placeholder")}
          className="h-12 bg-slate-50/50 border-slate-100 hover:bg-white hover:border-primary-200 transition-all rounded-[1.25rem] text-sm font-black text-slate-700 uppercase tracking-tight shadow-inner"
        />

        <CreateWorkspaceDialog
          isOpen={isCreateOpen}
          onClose={() => setIsCreateOpen(false)}
        />
      </div>
    );
  }

  