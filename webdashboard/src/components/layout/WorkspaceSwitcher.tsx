import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { Briefcase, Plus } from "lucide-react";
import {
  Workspace,
  getCurrentWorkspaceId,
  setCurrentWorkspaceId,
} from "@/lib/api";
import CreateWorkspaceDialog from "@/components/features/workspaces/CreateWorkspaceDialog";
import { useWorkspaces } from "@/hooks/use-workspace";
import { Combobox } from "@/components/ui/Combobox";

export default function WorkspaceSwitcher() {
  const { t } = useTranslation();
  const { workspaces, isLoading } = useWorkspaces();
  const [activeWorkspace, setActiveWorkspace] = useState<Workspace | null>(null);
  const [isCreateOpen, setIsCreateOpen] = useState(false);

  useEffect(() => {
    if (workspaces.length > 0) {
      const currentId = getCurrentWorkspaceId();
      const current = workspaces.find((ws) => ws.id === currentId) || workspaces[0];
      if (current) {
        // eslint-disable-next-line react-hooks/set-state-in-effect
        setActiveWorkspace(current);
        if (!currentId) setCurrentWorkspaceId(current.id);
      }
    }
  }, [workspaces]);

  function handleSwitch(wsId: string) {
    if (wsId === "ADD_NEW") { setIsCreateOpen(true); return; }
    const ws = workspaces.find(w => w.id === wsId);
    if (ws) { setActiveWorkspace(ws); setCurrentWorkspaceId(ws.id); window.location.reload(); }
  }

  if (isLoading && workspaces.length === 0) return <div className="w-full h-12 bg-slate-100 dark:bg-slate-700/50 animate-pulse rounded-[1.25rem] border border-slate-200 dark:border-slate-700/50" />;

  const options = [
    ...workspaces.map((ws) => ({
      value: ws.id, label: ws.name,
      icon: (<div className="bg-primary-50 dark:bg-primary-950 p-2 rounded-xl border border-primary-100 dark:border-primary-800 shadow-inner group-hover:bg-primary-600 group-hover:text-white transition-all"><Briefcase size={16} /></div>),
      description: (<div className="flex items-center gap-2 mt-1 opacity-60"><span className="text-[9px] font-bold uppercase tracking-widest">ID:</span><span className="text-[9px] font-mono font-bold">{ws.id.substring(0, 8).toUpperCase()}</span></div>)
    })),
    {
      value: "ADD_NEW", label: t("common.create_workspace"),
      icon: (<div className="bg-slate-100 dark:bg-slate-700 p-2 rounded-xl border border-slate-200 dark:border-slate-700 shadow-inner group-hover:bg-emerald-600 group-hover:text-white transition-all"><Plus size={16} /></div>),
      description: (<p className="text-[9px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest mt-1 opacity-60 italic">{t("common.add_office_branch")}</p>)
    }
  ];

  return (
    <div className="relative animate-in fade-in duration-500">
      <Combobox
        options={options}
        value={activeWorkspace?.id}
        onChange={handleSwitch}
        placeholder={t("common.select_workspace")}
        searchPlaceholder={t("common.search_placeholder")}
        className="h-14 bg-slate-50 dark:bg-slate-800/50 border-slate-100 dark:border-slate-800 hover:bg-white dark:hover:bg-slate-900 hover:border-primary-200 dark:border-primary-800 transition-all rounded-[1.5rem] text-sm font-bold text-slate-700 dark:text-slate-200 uppercase tracking-tight shadow-inner"
      />
      <CreateWorkspaceDialog isOpen={isCreateOpen} onClose={() => setIsCreateOpen(false)} />
    </div>
  );
}

  