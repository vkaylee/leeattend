import { useQuery } from "@tanstack/react-query";
import { Sparkles, Plus } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import { fetchIncentivePolicies } from "@/lib/api/payroll";
import { cn } from "@/lib/utils";

import { BonusPolicyCard } from "./bonus-policy-card";
import { BonusPolicyForm } from "./bonus-policy-form";
import { useBonusPolicyMutations } from "./hooks/use-bonus-policy-mutations";

interface BonusPolicyManagerProps {
}

export default function BonusPolicyManager({}: BonusPolicyManagerProps) {
  const { t } = useTranslation();
  const [isCreating, setIsCreating] = useState(false);

  const { data: policies = [], isLoading } = useQuery({ queryKey: ["incentive-policies"], queryFn: () => fetchIncentivePolicies(), });

  const { createMutation, deleteMutation, cloneMutation } = useBonusPolicyMutations(() => { setIsCreating(false); });

  return (
    <div className="space-y-10">
      <div className="flex items-center justify-between">
        <div>
          <Typography className="text-xl font-bold text-slate-700 dark:text-slate-200 uppercase tracking-tight flex items-center gap-3" variant="h2">
            <Sparkles className="text-primary-600 dark:text-primary-400" size={24} />
            {t("payroll.incentives.title", { defaultValue: "INCENTIVE & BONUS POLICIES" })}
          </Typography>
          <Typography className="text-sm text-slate-500 dark:text-slate-400 font-medium mt-1 uppercase tracking-widest text-[10px] opacity-70" variant="p">
            Manage Diligence, Performance and Safety bonuses
          </Typography>
        </div>
        <Button
          className={cn(
            "flex items-center gap-2 px-6 py-3 rounded-2xl font-bold text-[10px] uppercase tracking-widest transition-all shadow-lg active:scale-95",
            isCreating ? "bg-slate-100 dark:bg-slate-700 text-slate-500 dark:text-slate-400" : "bg-primary-600 text-white hover:bg-primary-700"
          )}
          onClick={() => { setIsCreating(!isCreating); }}
          variant="ghost"
        >
          {isCreating ? t("common.cancel") : <><Plus size={16} /> {t("payroll.incentives.add_btn", { defaultValue: "NEW POLICY" })}</>}
        </Button>
      </div>

      {isCreating ? (
        <BonusPolicyForm
          isPending={createMutation.isPending}
          onCancel={() => { setIsCreating(false); }}
          onSave={(data) => { createMutation.mutate(data); }}
        />
      ) : null}

      <div className="grid grid-cols-1 gap-6">
        {policies.map(policy => (
          <BonusPolicyCard
            key={policy.id}
            onClone={(id) => { cloneMutation.mutate(id); }}
            onDelete={(id) => { deleteMutation.mutate(id); }}
            policy={policy}
          />
        ))}

        {!isLoading && policies.length === 0 && !isCreating ? (
          <div className="py-24 text-center border-2 border-dashed border-slate-100 dark:border-slate-800 rounded-3xl sm:rounded-[3rem] bg-slate-50 dark:bg-slate-800/30 flex flex-col items-center">
            <div className="w-20 h-20 bg-white dark:bg-slate-900 rounded-full flex items-center justify-center shadow-sm mb-4 border border-slate-100 dark:border-slate-800">
              <Sparkles className="text-slate-200 dark:text-slate-700" size={32} />
            </div>
            <Typography className="text-lg font-bold text-slate-700 dark:text-slate-200" variant="h3">
              {t("payroll.incentives.no_policies", { defaultValue: "NO INCENTIVE POLICIES" })}
            </Typography>
            <Typography className="text-slate-500 dark:text-slate-400 text-sm mt-1 uppercase font-bold tracking-widest opacity-50" variant="p">
              Configure your first bonus rule to reward diligent staff
            </Typography>
          </div>
        ) : null}
      </div>
    </div>
  );
}
