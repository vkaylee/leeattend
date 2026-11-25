import { Copy, Layers, Plus } from "lucide-react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { Typography } from "@/components/ui/typography";
import type { WorkPolicyGroupSummary } from "@/lib/api/payroll";
import { cn } from "@/lib/utils";

interface WorkPolicyGroupListProps {
  canManage: boolean;
  groups: WorkPolicyGroupSummary[];
  isError: boolean;
  isLoading: boolean;
  onCloneLatest: (group: WorkPolicyGroupSummary) => void;
  onCreateGroup: () => void;
  onSelectGroup: (groupId: string) => void;
  selectedGroupId: string;
}

export function WorkPolicyGroupList({
  canManage,
  groups,
  isError,
  isLoading,
  onCloneLatest,
  onCreateGroup,
  onSelectGroup,
  selectedGroupId,
}: WorkPolicyGroupListProps) {
  const { t } = useTranslation();

  if (isLoading) {
    return (
      <div className="space-y-3" data-testid="work-policy-group-list-loading">
        <Skeleton className="h-10 w-full rounded-xl" />
        <Skeleton className="h-24 w-full rounded-2xl" />
        <Skeleton className="h-24 w-full rounded-2xl" />
      </div>
    );
  }

  if (isError) {
    return (
      <div className="rounded-2xl border border-rose-200 bg-rose-50 p-4 text-sm text-rose-700 dark:border-rose-800 dark:bg-rose-950/30 dark:text-rose-200" role="alert">
        {t("payroll.work_policies.load_error", "Failed to load work policy groups.")}
      </div>
    );
  }

  return (
    <div className="space-y-4" data-testid="work-policy-group-list">
      <div className="flex items-center justify-between gap-2">
        <div className="flex items-center gap-2">
          <Layers className="h-4 w-4 text-primary-600" />
          <Typography className="text-sm font-bold text-slate-800 dark:text-slate-100" variant="h3">
            {t("payroll.work_policies.groups_title", "Work Policy Groups")}
          </Typography>
        </div>
        {canManage ? (
          <Button data-testid="create-work-policy-group-btn" onClick={onCreateGroup} size="sm" variant="primary">
            <Plus className="mr-1 h-3.5 w-3.5" />
            {t("payroll.work_policies.create_group", "New Group")}
          </Button>
        ) : null}
      </div>

      {groups.length === 0 ? (
        <div className="rounded-2xl border border-dashed border-slate-200 bg-slate-50 p-6 text-center text-xs text-slate-500 dark:border-slate-800 dark:bg-slate-900/40 dark:text-slate-400">
          {t("payroll.work_policies.empty_groups", "No work policy groups yet. Create one to start configuring versioned attendance and payroll rules.")}
        </div>
      ) : (
        <div className="space-y-2.5">
          {groups.map((group) => {
            const isSelected = group.id === selectedGroupId;
            return (
              <div
                className={cn(
                  "rounded-2xl border p-4 transition-all",
                  isSelected
                    ? "border-primary-500 bg-primary-50/40 shadow-xs dark:border-primary-500 dark:bg-primary-950/30"
                    : "border-slate-200 bg-white hover:border-slate-300 dark:border-slate-800 dark:bg-slate-900 dark:hover:border-slate-700",
                )}
                data-testid={`work-policy-group-card-${group.code}`}
                key={group.id}
              >
                <div className="flex items-start justify-between gap-2">
                  <Button
                    className="h-auto flex-1 justify-start p-0 text-left font-normal"
                    onClick={() => { onSelectGroup(group.id); }}
                    variant="ghost"
                  >
                    <div>
                      <div className="flex items-center gap-2">
                        <span className="font-mono text-xs font-bold text-primary-600 dark:text-primary-400">{group.code}</span>
                        <span className="text-sm font-bold text-slate-800 dark:text-slate-100">{group.name}</span>
                      </div>
                      {group.description ? (
                        <p className="mt-1 line-clamp-2 text-xs text-slate-500 dark:text-slate-400">{group.description}</p>
                      ) : null}
                    </div>
                  </Button>
                  {canManage && (group.current_published || group.latest_draft) ? (
                    <Button
                      aria-label={t("payroll.work_policies.clone_version", "Clone to new draft")}
                      onClick={() => { onCloneLatest(group); }}
                      size="sm"
                      title={t("payroll.work_policies.clone_version", "Clone to new draft")}
                      variant="ghost"
                    >
                      <Copy className="h-3.5 w-3.5" />
                    </Button>
                  ) : null}
                </div>
                <div className="mt-3 flex flex-wrap items-center gap-2 text-[11px]">
                  {group.current_published ? (
                    <span className="rounded-full bg-emerald-50 px-2.5 py-0.5 font-semibold text-emerald-700 dark:bg-emerald-950/60 dark:text-emerald-300">
                      {t("payroll.work_policies.published_badge", "Published")} v{group.current_published.version_no}
                    </span>
                  ) : (
                    <span className="rounded-full bg-slate-100 px-2.5 py-0.5 font-medium text-slate-500 dark:bg-slate-800 dark:text-slate-400">
                      {t("payroll.work_policies.unpublished_badge", "Unpublished")}
                    </span>
                  )}
                  {group.latest_draft ? (
                    <span className="rounded-full bg-amber-50 px-2.5 py-0.5 font-semibold text-amber-700 dark:bg-amber-950/60 dark:text-amber-300">
                      {t("payroll.work_policies.draft_badge", "Draft")} v{group.latest_draft.version_no}
                    </span>
                  ) : null}
                  <span className="ml-auto text-slate-500 dark:text-slate-400">
                    {group.active_membership_count} {t("payroll.work_policies.members_count", "assigned")}
                  </span>
                </div>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}
