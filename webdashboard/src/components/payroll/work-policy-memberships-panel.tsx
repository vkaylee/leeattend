import { ArrowRightLeft, Plus, Users } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import type { WorkPolicyGroupSummary, WorkPolicyMembership } from "@/lib/api/payroll";

interface RelationshipOption {
  id: string;
  label: string;
}

interface WorkPolicyMembershipsPanelProps {
  allGroups: WorkPolicyGroupSummary[];
  canManage: boolean;
  currentGroupId: string;
  isAssigning: boolean;
  isLoading: boolean;
  isTransferring: boolean;
  memberships: WorkPolicyMembership[];
  onAssign: (payload: { effective_from: string; effective_to?: string | null; employment_relationship_id: string }) => void;
  onTransfer: (payload: { effective_date: string; employment_relationship_id: string; target_group_id: string }) => void;
  relationshipOptions: RelationshipOption[];
}

export function WorkPolicyMembershipsPanel({
  allGroups,
  canManage,
  currentGroupId,
  isAssigning,
  isLoading,
  isTransferring,
  memberships,
  onAssign,
  onTransfer,
  relationshipOptions,
}: WorkPolicyMembershipsPanelProps) {
  const { t } = useTranslation();
  const today = new Date().toISOString().slice(0, 10);

  const [relId, setRelId] = useState("");
  const [effFrom, setEffFrom] = useState(today);
  const [effTo, setEffTo] = useState("");

  const [transferRelId, setTransferRelId] = useState<string | null>(null);
  const [transferTargetGroupId, setTransferTargetGroupId] = useState("");
  const [transferDate, setTransferDate] = useState(today);

  const labelByRelId = new Map(relationshipOptions.map((o) => [o.id, o.label]));
  const otherGroups = allGroups.filter((g) => g.id !== currentGroupId);

  return (
    <div className="space-y-4 rounded-2xl border border-slate-200 bg-white p-6 dark:border-slate-800 dark:bg-slate-900" data-testid="work-policy-memberships-panel">
      <div className="flex items-center gap-2">
        <Users className="h-4 w-4 text-primary-600" />
        <Typography className="text-sm font-bold text-slate-800 dark:text-slate-100" variant="h3">
          {t("payroll.work_policies.memberships_title", "Employment Relationship Memberships")}
        </Typography>
      </div>

      {canManage ? (
        <form
          className="grid grid-cols-1 gap-3 rounded-xl border border-slate-100 bg-slate-50 p-4 dark:border-slate-800 dark:bg-slate-800/40 sm:grid-cols-4"
          onSubmit={(e) => {
            e.preventDefault();
            if (!relId || !effFrom) return;
            onAssign({ employment_relationship_id: relId, effective_from: effFrom, effective_to: effTo || null });
            setRelId("");
            setEffTo("");
          }}
        >
          <select
            aria-label={t("payroll.work_policies.relationship", "Employment Relationship")}
            className="rounded-xl border border-slate-200 bg-white px-3 py-2 text-xs dark:border-slate-700 dark:bg-slate-950"
            data-testid="wp-membership-rel-select"
            onChange={(e) => { setRelId(e.target.value); }}
            value={relId}
          >
            <option value="">{t("payroll.work_policies.select_relationship", "Select employee relationship...")}</option>
            {relationshipOptions.map((o) => (
              <option key={o.id} value={o.id}>{o.label}</option>
            ))}
          </select>
          <input
            aria-label={t("payroll.work_policies.effective_from", "Effective From")}
            className="rounded-xl border border-slate-200 bg-white px-3 py-2 text-xs dark:border-slate-700 dark:bg-slate-950"
            onChange={(e) => { setEffFrom(e.target.value); }}
            required
            type="date"
            value={effFrom}
          />
          <input
            aria-label={t("payroll.work_policies.effective_to", "Effective To (Optional)")}
            className="rounded-xl border border-slate-200 bg-white px-3 py-2 text-xs dark:border-slate-700 dark:bg-slate-950"
            onChange={(e) => { setEffTo(e.target.value); }}
            type="date"
            value={effTo}
          />
          <Button data-testid="wp-assign-membership-btn" disabled={!relId || isAssigning} size="sm" type="submit" variant="primary">
            <Plus className="mr-1 h-3.5 w-3.5" />
            {t("payroll.work_policies.assign_member", "Assign")}
          </Button>
        </form>
      ) : null}

      {isLoading ? (
        <p className="text-xs text-slate-400">{t("common.loading", "Loading...")}</p>
      ) : memberships.length === 0 ? (
        <p className="text-xs italic text-slate-400">{t("payroll.work_policies.no_memberships", "No relationships assigned to this group yet.")}</p>
      ) : (
        <div className="divide-y divide-slate-100 rounded-xl border border-slate-100 dark:divide-slate-800 dark:border-slate-800">
          {memberships.map((m) => (
            <div className="flex flex-wrap items-center justify-between gap-3 px-4 py-3 text-xs" key={m.id}>
              <div>
                <div className="font-semibold text-slate-800 dark:text-slate-100">
                  {labelByRelId.get(m.employment_relationship_id) ?? m.employment_relationship_id}
                </div>
                <div className="font-mono text-[11px] text-slate-500">
                  {m.effective_from} → {m.effective_to ?? t("common.present", "Present")}
                </div>
              </div>
              {canManage && otherGroups.length > 0 ? (
                <Button
                  data-testid={`wp-open-transfer-${m.employment_relationship_id}`}
                  onClick={() => {
                    setTransferRelId(m.employment_relationship_id);
                    setTransferTargetGroupId(otherGroups[0]?.id ?? "");
                  }}
                  size="sm"
                  variant="outline"
                >
                  <ArrowRightLeft className="mr-1 h-3.5 w-3.5" />
                  {t("payroll.work_policies.transfer", "Transfer")}
                </Button>
              ) : null}
            </div>
          ))}
        </div>
      )}

      {transferRelId ? (
        <div className="space-y-3 rounded-xl border border-primary-200 bg-primary-50/50 p-4 dark:border-primary-800 dark:bg-primary-950/30" data-testid="wp-transfer-modal">
          <Typography className="text-xs font-bold text-primary-700 dark:text-primary-300" variant="h4">
            {t("payroll.work_policies.atomic_transfer_title", "Atomic Group Transfer")}
          </Typography>
          <div className="grid grid-cols-1 gap-3 sm:grid-cols-3">
            <select
              aria-label={t("payroll.work_policies.target_group", "Target Group")}
              className="rounded-xl border border-slate-200 bg-white px-3 py-2 text-xs dark:border-slate-700 dark:bg-slate-950"
              data-testid="wp-transfer-target-group"
              onChange={(e) => { setTransferTargetGroupId(e.target.value); }}
              value={transferTargetGroupId}
            >
              {otherGroups.map((g) => (
                <option key={g.id} value={g.id}>{g.code} — {g.name}</option>
              ))}
            </select>
            <input
              aria-label={t("payroll.work_policies.effective_date", "Transfer Date")}
              className="rounded-xl border border-slate-200 bg-white px-3 py-2 text-xs dark:border-slate-700 dark:bg-slate-950"
              data-testid="wp-transfer-date"
              onChange={(e) => { setTransferDate(e.target.value); }}
              type="date"
              value={transferDate}
            />
            <div className="flex items-center justify-end gap-2">
              <Button onClick={() => { setTransferRelId(null); }} size="sm" variant="ghost">
                {t("common.cancel", "Cancel")}
              </Button>
              <Button
                data-testid="wp-confirm-transfer-btn"
                disabled={!transferTargetGroupId || !transferDate || isTransferring}
                onClick={() => {
                  onTransfer({
                    employment_relationship_id: transferRelId,
                    target_group_id: transferTargetGroupId,
                    effective_date: transferDate,
                  });
                  setTransferRelId(null);
                }}
                size="sm"
                variant="primary"
              >
                {t("payroll.work_policies.confirm_transfer", "Confirm Transfer")}
              </Button>
            </div>
          </div>
        </div>
      ) : null}
    </div>
  );
}
