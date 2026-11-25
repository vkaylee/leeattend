import { AlertCircle, Calculator, Plus, Settings2 } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import FormField from "@/components/ui/form-field";
import { Skeleton } from "@/components/ui/skeleton";
import { Typography } from "@/components/ui/typography";
import type { OvertimePolicyV2 } from "@/lib/api/payroll";

import { useOvertimePolicies, useOvertimePolicy, useCreateOvertimePolicy } from "./hooks/use-overtime-policies";
import OvertimeRuleEditor from "./overtime-rule-editor";

function OvertimePolicyCard({ policy }: { policy: OvertimePolicyV2 }) {
  const { data: detail, isLoading } = useOvertimePolicy(policy.id);

  return (
    <div className="group rounded-3xl border border-slate-100 bg-white p-6 shadow-sm transition-all hover:shadow-md dark:border-slate-800 dark:bg-slate-900">
      <div className="mb-4 flex items-center justify-between">
        <div className="flex items-center gap-4">
          <div className="flex h-12 w-12 items-center justify-center rounded-2xl bg-slate-50 text-slate-500 transition-colors group-hover:bg-primary-50 group-hover:text-primary-600 dark:bg-slate-800 dark:text-slate-400 dark:group-hover:bg-primary-950 dark:group-hover:text-primary-400">
            <Settings2 size={24} />
          </div>
          <div>
            <Typography className="text-lg font-bold text-slate-700 dark:text-slate-200" variant="h3">{policy.name}</Typography>
            <Typography className="text-sm text-slate-500 dark:text-slate-400" variant="p">{policy.description}</Typography>
          </div>
        </div>
      </div>

      {isLoading ? (
        <Skeleton className="h-20 rounded-2xl bg-slate-50 dark:bg-slate-800" />
      ) : (
        <OvertimeRuleEditor policyId={policy.id} versions={detail?.versions ?? []} />
      )}
    </div>
  );
}

function OvertimePolicyForm({
  onCancel,
  onSave,
  isPending,
}: {
  isPending: boolean;
  onCancel: () => void;
  onSave: (data: { description: string; name: string }) => void;
}) {
  const { t } = useTranslation();
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");

  return (
    <div className="rounded-3xl border border-slate-200 bg-slate-50 p-6 animate-in fade-in slide-in-from-top-4 duration-300 dark:border-slate-700 dark:bg-slate-800">
      <div className="grid grid-cols-1 gap-6 md:grid-cols-2">
        <FormField
          id="overtime-policy-name"
          label={t("common.name")}
          onChange={(e) => { setName(e.target.value); }}
          placeholder={t("payroll.overtime_policy_name_placeholder", { defaultValue: "e.g. Standard Overtime" })}
          type="text"
          value={name}
        />
        <FormField
          id="overtime-policy-description"
          label={t("common.description")}
          onChange={(e) => { setDescription(e.target.value); }}
          placeholder={t("payroll.overtime_policy_desc_placeholder", { defaultValue: "e.g. Night and Weekend rates" })}
          type="text"
          value={description}
        />
      </div>
      <div className="mt-6 flex justify-end gap-3">
        <Button onClick={onCancel} variant="ghost">
          {t("common.cancel", { defaultValue: "Cancel" })}
        </Button>
        <Button
          disabled={!name || isPending}
          onClick={() => { onSave({ name, description }); }}
          variant="primary"
        >
          {t("common.save")}
        </Button>
      </div>
    </div>
  );
}

export default function OvertimePolicyManager() {
  const { t } = useTranslation();
  const [isCreating, setIsCreating] = useState(false);

  const { data: policies = [], isLoading } = useOvertimePolicies();
  const createMutation = useCreateOvertimePolicy({
    onSuccess: () => { setIsCreating(false); },
  });

  if (isLoading) {
    return (
      <div className="space-y-4">
        <Skeleton className="h-12 w-64 rounded-xl" />
        <Skeleton className="h-40 w-full rounded-2xl" />
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <Typography className="flex items-center gap-2 text-xl font-bold text-slate-700 dark:text-slate-200" variant="h2">
            <Calculator className="text-primary-600 dark:text-primary-400" size={20} />
            {t("payroll.overtime_policies")}
          </Typography>
          <Typography className="mt-1 text-sm text-slate-500 dark:text-slate-400" variant="p">
            {t("payroll.overtime_policies_desc")}
          </Typography>
        </div>
        <Button onClick={() => { setIsCreating(!isCreating); }} variant="primary">
          <Plus className="mr-1.5 h-4 w-4" />
          {t("common.create")}
        </Button>
      </div>

      {isCreating ? (
        <OvertimePolicyForm
          isPending={createMutation.isPending}
          onCancel={() => { setIsCreating(false); }}
          onSave={(data) => {
            createMutation.mutate({
              name: data.name,
              description: data.description,
              rules: [{ priority: 50, day_type: "weekday", days_of_week: 127, multiplier: "1.50" }],
            });
          }}
        />
      ) : null}

      <div className="grid grid-cols-1 gap-4">
        {policies.map((policy) => (
          <OvertimePolicyCard key={policy.id} policy={policy} />
        ))}

        {policies.length === 0 && !isCreating ? (
          <div className="flex flex-col items-center justify-center rounded-2xl border-2 border-dashed border-slate-200 bg-slate-50 py-12 dark:border-slate-700 dark:bg-slate-800">
            <AlertCircle className="mb-4 text-slate-300 dark:text-slate-600" size={48} />
            <Typography className="font-medium text-slate-500 dark:text-slate-400" variant="p">
              {t("payroll.no_overtime_policies")}
            </Typography>
            <Button className="mt-4" onClick={() => { setIsCreating(true); }} variant="ghost">
              {t("payroll.create_first_policy")}
            </Button>
          </div>
        ) : null}
      </div>
    </div>
  );
}
