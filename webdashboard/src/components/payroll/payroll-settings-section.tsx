import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import { useMyPermissions } from "@/hooks/use-permissions";
import { fetchEmployeeRelationships, fetchEmployees } from "@/lib/api/employees";
import {
  createWorkPolicyGroup,
  createWorkPolicyMembership,
  createWorkPolicyVersion,
  fetchWorkPolicyGroups,
  fetchWorkPolicyGroupVersions,
  fetchWorkPolicyMemberships,
  fetchWorkPolicyReferenceOptions,
  patchWorkPolicyVersion,
  publishWorkPolicyVersion,
  simulateWorkPolicyVersionBatch,
  transferWorkPolicyMembership,
  validateWorkPolicyVersion,
  type WorkPolicyBatchSimulationResponse,
  type WorkPolicyGroupSummary,
  type WorkPolicyValidationResponse,
  type WorkPolicyVersionFields,
} from "@/lib/api/payroll";
import { payrollKeys } from "@/lib/query-keys";

import { WorkPolicyGroupList } from "./work-policy-group-list";
import { WorkPolicyMembershipsPanel } from "./work-policy-memberships-panel";
import { WorkPolicySimulationPanel } from "./work-policy-simulation-panel";
import { WorkPolicyVersionEditor } from "./work-policy-version-editor";

interface PayrollSettingsSectionProps {
  onDirtyChange?: (isDirty: boolean) => void;
}

export default function PayrollSettingsSection({ onDirtyChange }: PayrollSettingsSectionProps) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const toast = useToast();
  const { hasPermission } = useMyPermissions();

  const canManage = hasPermission("payroll:manage");
  const canSimulate = hasPermission("payroll:sensitive:read");

  const [selectedGroupId, setSelectedGroupId] = useState("");
  const [selectedVersionId, setSelectedVersionId] = useState("");
  const [isCreatingGroup, setIsCreatingGroup] = useState(false);
  const [newGroupCode, setNewGroupCode] = useState("");
  const [newGroupName, setNewGroupName] = useState("");
  const [newGroupDesc, setNewGroupDesc] = useState("");
  const [conflictMessage, setConflictMessage] = useState<string | null>(null);
  const [validationResult, setValidationResult] = useState<WorkPolicyValidationResponse | null>(null);
  const [simResult, setSimResult] = useState<WorkPolicyBatchSimulationResponse | null>(null);

  const { data: groups = [], isLoading: isGroupsLoading, isError: isGroupsError } = useQuery({
    queryKey: payrollKeys.workPolicyGroups(),
    queryFn: () => fetchWorkPolicyGroups(),
  });

  const activeGroupId = selectedGroupId || groups[0]?.id || "";

  const { data: versions = [] } = useQuery({
    queryKey: payrollKeys.workPolicyVersions(activeGroupId),
    queryFn: () => fetchWorkPolicyGroupVersions(activeGroupId),
    enabled: Boolean(activeGroupId),
  });

  const activeVersion = useMemo(() => {
    if (versions.length === 0) return null;
    return versions.find((v) => v.id === selectedVersionId) ?? versions[0];
  }, [versions, selectedVersionId]);

  const { data: memberships = [], isLoading: isMembershipsLoading } = useQuery({
    queryKey: payrollKeys.workPolicyMemberships(activeGroupId),
    queryFn: () => fetchWorkPolicyMemberships(activeGroupId),
    enabled: Boolean(activeGroupId),
  });

  const { data: references = { overtimeVersions: [], patterns: [], salaryComponents: [] } } = useQuery({
    queryKey: payrollKeys.workPolicyReferences(),
    queryFn: () => fetchWorkPolicyReferenceOptions(),
  });

  const { data: relationshipOptions = [] } = useQuery({
    queryKey: ["work-policy-employee-relationships"],
    queryFn: async () => {
      const resp = await fetchEmployees(1, 100);
      const items = resp?.data ?? [];
      const resolved = await Promise.all(
        items.map(async (emp) => {
          try {
            const rels = await fetchEmployeeRelationships(emp.id);
            const active = rels.find((r) => r.status === "active") ?? rels[0];
            if (!active) return null;
            return { id: active.id, label: `${emp.employee_id_number} — ${emp.full_name}` };
          } catch {
            return null;
          }
        }),
      );
      return resolved.filter((o): o is { id: string; label: string } => o !== null);
    },
  });

  const relationshipLabels = useMemo(() => new Map(relationshipOptions.map((o) => [o.id, o.label])), [relationshipOptions]);

  const invalidateAll = () => {
    void queryClient.invalidateQueries({ queryKey: payrollKeys.workPolicyGroups() });
    if (activeGroupId) {
      void queryClient.invalidateQueries({ queryKey: payrollKeys.workPolicyVersions(activeGroupId) });
      void queryClient.invalidateQueries({ queryKey: payrollKeys.workPolicyMemberships(activeGroupId) });
    }
  };

  const createGroupMutation = useMutation({
    mutationFn: () =>
      createWorkPolicyGroup({
        code: newGroupCode.trim().toUpperCase(),
        name: newGroupName.trim(),
        description: newGroupDesc.trim() || null,
        effective_from: new Date().toISOString().slice(0, 10),
        minutes_per_credit: 480,
        work_unit_minutes: 1,
        rounding_policy: "none",
        early_arrival_strategy: "clamp",
        late_arrival_grace_minutes: 0,
        early_ot_enabled: false,
        early_ot_qualification_grace_minutes: 0,
        break_auto_deduct: true,
        break_is_overtime: false,
        ot_aggregation_mode: "aggregate",
        pay_method: "monthly",
        monthly_denominator: "fixed",
        monthly_standard_days: "22",
        paid_leave_credit_mode: "standard_day",
        overtime_policy_version_id: references.overtimeVersions[0]?.id ?? "",
      }),
    onSuccess: (created) => {
      setIsCreatingGroup(false);
      setNewGroupCode("");
      setNewGroupName("");
      setNewGroupDesc("");
      setSelectedGroupId(created.group_id);
      setSelectedVersionId(created.id);
      invalidateAll();
      toast.success(t("common.success"));
    },
    onError: (err: Error) => { toast.error(err.message); },
  });

  const cloneVersionMutation = useMutation({
    mutationFn: (group: WorkPolicyGroupSummary) => {
      const sourceId = group.latest_draft?.id ?? group.current_published?.id;
      return createWorkPolicyVersion(group.id, { clone_from_version_id: sourceId });
    },
    onSuccess: (created) => {
      setSelectedGroupId(created.group_id);
      setSelectedVersionId(created.id);
      setValidationResult(null);
      setConflictMessage(null);
      invalidateAll();
      toast.success(t("common.success"));
    },
    onError: (err: Error) => { toast.error(err.message); },
  });

  const patchVersionMutation = useMutation({
    mutationFn: (fields: WorkPolicyVersionFields) =>
      patchWorkPolicyVersion(activeGroupId, activeVersion!.id, {
        ...fields,
        expected_row_version: activeVersion!.row_version,
      }),
    onSuccess: () => {
      setConflictMessage(null);
      setValidationResult(null);
      onDirtyChange?.(false);
      invalidateAll();
      toast.success(t("common.success"));
    },
    onError: (err: Error & { status?: number }) => {
      if (err.status === 409 || err.message.toLowerCase().includes("conflict")) {
        setConflictMessage(t("payroll.work_policies.conflict_409", "This draft was modified concurrently. Reload the latest version before saving."));
      } else {
        toast.error(err.message);
      }
    },
  });

  const validateMutation = useMutation({
    mutationFn: () => validateWorkPolicyVersion(activeGroupId, activeVersion!.id),
    onSuccess: (res) => { setValidationResult(res); },
    onError: (err: Error) => { toast.error(err.message); },
  });

  const publishMutation = useMutation({
    mutationFn: (autoClosePrevious: boolean) => {
      const idemKey = `pub-${activeVersion!.id}-v${activeVersion!.row_version}`;
      return publishWorkPolicyVersion(
        activeGroupId,
        activeVersion!.id,
        { expected_row_version: activeVersion!.row_version, auto_close_previous: autoClosePrevious, idempotency_key: idemKey },
        idemKey,
      );
    },
    onSuccess: () => {
      setConflictMessage(null);
      setValidationResult(null);
      invalidateAll();
      toast.success(t("payroll.work_policies.published_success", "Work policy version published."));
    },
    onError: (err: Error & { status?: number }) => {
      if (err.status === 409 || err.message.toLowerCase().includes("conflict")) {
        setConflictMessage(t("payroll.work_policies.publish_conflict", "Publish blocked due to version conflict, date overlap, or locked payroll period."));
      } else {
        toast.error(err.message);
      }
    },
  });

  const assignMembershipMutation = useMutation({
    mutationFn: (payload: { effective_from: string; effective_to?: string | null; employment_relationship_id: string }) =>
      createWorkPolicyMembership(activeGroupId, payload),
    onSuccess: () => { invalidateAll(); toast.success(t("common.success")); },
    onError: (err: Error) => { toast.error(err.message); },
  });

  const transferMembershipMutation = useMutation({
    mutationFn: (payload: { effective_date: string; employment_relationship_id: string; target_group_id: string }) =>
      transferWorkPolicyMembership(payload),
    onSuccess: () => { invalidateAll(); toast.success(t("common.success")); },
    onError: (err: Error) => { toast.error(err.message); },
  });

  const simulateMutation = useMutation({
    mutationFn: ({ month, relationshipIds, year }: { month: number; relationshipIds: string[]; year: number }) =>
      simulateWorkPolicyVersionBatch(activeGroupId, activeVersion!.id, {
        year,
        month,
        employment_relationship_ids: relationshipIds,
      }),
    onSuccess: (res) => { setSimResult(res); },
    onError: (err: Error) => { toast.error(err.message); },
  });

  return (
    <div className="space-y-6" data-testid="payroll-settings-section">
      {isCreatingGroup ? (
        <form
          className="space-y-4 rounded-2xl border border-primary-200 bg-primary-50/30 p-5 dark:border-primary-800 dark:bg-primary-950/20"
          data-testid="create-work-policy-group-form"
          onSubmit={(e) => {
            e.preventDefault();
            if (!newGroupCode.trim() || !newGroupName.trim()) return;
            createGroupMutation.mutate();
          }}
        >
          <Typography className="text-sm font-bold text-slate-800 dark:text-slate-100" variant="h3">
            {t("payroll.work_policies.create_group_title", "Create Work Policy Group (Draft v1)")}
          </Typography>
          <div className="grid grid-cols-1 gap-3 sm:grid-cols-3">
            <input
              aria-label={t("payroll.work_policies.group_code", "Group Code")}
              className="rounded-xl border border-slate-200 bg-white px-3 py-2 text-sm dark:border-slate-700 dark:bg-slate-950"
              data-testid="new-wp-code"
              onChange={(e) => { setNewGroupCode(e.target.value); }}
              placeholder="e.g. SHIFT-PROD"
              required
              value={newGroupCode}
            />
            <input
              aria-label={t("payroll.work_policies.group_name", "Group Name")}
              className="rounded-xl border border-slate-200 bg-white px-3 py-2 text-sm dark:border-slate-700 dark:bg-slate-950"
              data-testid="new-wp-name"
              onChange={(e) => { setNewGroupName(e.target.value); }}
              placeholder="Production Shift Policy"
              required
              value={newGroupName}
            />
            <input
              aria-label={t("common.description", "Description")}
              className="rounded-xl border border-slate-200 bg-white px-3 py-2 text-sm dark:border-slate-700 dark:bg-slate-950"
              onChange={(e) => { setNewGroupDesc(e.target.value); }}
              placeholder={t("common.description", "Description")}
              value={newGroupDesc}
            />
          </div>
          <div className="flex justify-end gap-2">
            <Button onClick={() => { setIsCreatingGroup(false); }} size="sm" variant="ghost">
              {t("common.cancel", "Cancel")}
            </Button>
            <Button
              data-testid="submit-new-wp-group-btn"
              disabled={createGroupMutation.isPending || references.overtimeVersions.length === 0}
              size="sm"
              type="submit"
              variant="primary"
            >
              {t("common.create", "Create")}
            </Button>
          </div>
        </form>
      ) : null}

      <div className="grid grid-cols-1 gap-6 lg:grid-cols-12">
        <div className="lg:col-span-4">
          <WorkPolicyGroupList
            canManage={canManage}
            groups={groups}
            isError={isGroupsError}
            isLoading={isGroupsLoading}
            onCloneLatest={(group) => { cloneVersionMutation.mutate(group); }}
            onCreateGroup={() => { setIsCreatingGroup(true); }}
            onSelectGroup={(id) => {
              setSelectedGroupId(id);
              setSelectedVersionId("");
              setValidationResult(null);
              setSimResult(null);
              setConflictMessage(null);
            }}
            selectedGroupId={activeGroupId}
          />
        </div>

        <div className="space-y-6 lg:col-span-8">
          {versions.length > 0 ? (
            <div className="flex flex-wrap items-center gap-2">
              {versions.map((v) => (
                <Button
                  data-testid={`wp-version-tab-v${v.version_no}`}
                  key={v.id}
                  onClick={() => {
                    setSelectedVersionId(v.id);
                    setValidationResult(null);
                    setSimResult(null);
                    setConflictMessage(null);
                  }}
                  size="sm"
                  variant={activeVersion?.id === v.id ? "primary" : "outline"}
                >
                  v{v.version_no} ({v.status})
                </Button>
              ))}
            </div>
          ) : null}

          {activeVersion ? (
            <>
              <WorkPolicyVersionEditor
                canManage={canManage}
                conflictMessage={conflictMessage}
                isPublishing={publishMutation.isPending}
                isSaving={patchVersionMutation.isPending}
                isValidating={validateMutation.isPending}
                onDirtyChange={onDirtyChange}
                onPublish={(autoClose) => { publishMutation.mutate(autoClose); }}
                onReload={() => {
                  setConflictMessage(null);
                  invalidateAll();
                }}
                onSaveDraft={(fields) => { patchVersionMutation.mutate(fields); }}
                onValidate={() => { validateMutation.mutate(); }}
                references={references}
                validation={validationResult}
                version={activeVersion}
              />

              <WorkPolicySimulationPanel
                canSimulate={canSimulate}
                defaultRelationshipIds={memberships.map((m) => m.employment_relationship_id)}
                isRunning={simulateMutation.isPending}
                onRunSimulation={(year, month, relationshipIds) => {
                  simulateMutation.mutate({ year, month, relationshipIds });
                }}
                relationshipLabels={relationshipLabels}
                result={simResult}
              />

              <WorkPolicyMembershipsPanel
                allGroups={groups}
                canManage={canManage}
                currentGroupId={activeGroupId}
                isAssigning={assignMembershipMutation.isPending}
                isLoading={isMembershipsLoading}
                isTransferring={transferMembershipMutation.isPending}
                memberships={memberships}
                onAssign={(payload) => { assignMembershipMutation.mutate(payload); }}
                onTransfer={(payload) => { transferMembershipMutation.mutate(payload); }}
                relationshipOptions={relationshipOptions}
              />
            </>
          ) : null}
        </div>
      </div>
    </div>
  );
}
