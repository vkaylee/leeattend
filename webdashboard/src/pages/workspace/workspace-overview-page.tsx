import { Loader2 } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import PageHeader from "@/components/ui/page-header";
import { useDashboardStats } from "@/hooks/use-dashboard";
import { useWorkspace, useWorkspaceMembers, useWorkspaceRoles } from "@/hooks/use-workspace";

import { AdminMembersPreview, WorkspaceHeroCard, WorkspaceStatsGrid } from "./components/overview-subcomponents";

export default function WorkspaceOverviewPage() {
  const { t } = useTranslation();
  const { workspace, isLoading: isWsLoading } = useWorkspace();
  const { members, isLoading: isMembersLoading } = useWorkspaceMembers();
  const { roles, isLoading: isRolesLoading } = useWorkspaceRoles();
  const { data: stats, isLoading: isStatsLoading } = useDashboardStats();

  if (isWsLoading || isMembersLoading || isStatsLoading || isRolesLoading) {
    return (
      <div className="flex items-center justify-center min-h-[400px]">
        <Loader2 className="w-8 h-8 animate-spin text-primary-500" />
      </div>
    );
  }

  if (!workspace) return null;

  return (
    <div className="space-y-10 animate-in fade-in duration-500">
      <PageHeader
        subtitle={`ID: ${workspace.id.substring(0, 8).toUpperCase()}`}
        title={t('overview.title', { defaultValue: 'Workspace Overview' })}
      />

      <WorkspaceHeroCard workspace={workspace} />
      <WorkspaceStatsGrid stats={stats} />
      <AdminMembersPreview members={members} roles={roles} />
    </div>
  );
}
