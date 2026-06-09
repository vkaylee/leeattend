import { useNavigate } from "@tanstack/react-router";
import { Plus, AlertCircle } from "lucide-react";
import React from "react";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";

import { AssignmentRulesTable } from "./assignment-rules/components/assignment-rules-table";
import { useAssignmentRules } from "./assignment-rules/hooks/use-assignment-rules";

export default function AssignmentRulesPage() {
  const navigate = useNavigate();
  const {
    t,
    rules,
    rulesLoading,
    patterns,
    departments,
    employers,
    locations,
    handleDelete,
    ConfirmationDialog,
  } = useAssignmentRules();

  return (
    <div className="space-y-10 animate-in fade-in duration-700 pb-20">
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-8 px-2">
        <div className="space-y-1">
          <Typography className="text-4xl font-bold tracking-tighter text-slate-700 dark:text-slate-200 uppercase leading-none" data-testid="assignment-rules-title" variant="h1">
            {t("rules.title")}
          </Typography>
          <Typography className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] opacity-60" variant="p">
            {t("rules.subtitle")}
          </Typography>
        </div>
        <Button
          icon={<Plus size={18} />}
          onClick={() => {
            void navigate({ to: "/assignment-rules/new" });
          }}
          variant="primary"
        >
          {t("rules.create_btn").toUpperCase()}
        </Button>
      </div>

      <AssignmentRulesTable
        departments={departments}
        employers={employers}
        isLoading={rulesLoading}
        locations={locations}
        onDelete={handleDelete}
        onEdit={(rule) => {
          void navigate({ to: `/assignment-rules/${rule.id}/edit` });
        }}
        patterns={patterns}
        rules={rules}
      />

      <div className="p-8 bg-blue-50 dark:bg-blue-950/50 border border-blue-100 dark:border-blue-900 rounded-[2.5rem] flex gap-6 shadow-inner animate-in fade-in slide-in-from-bottom-4 duration-1000">
        <div className="w-12 h-12 rounded-2xl bg-white dark:bg-slate-800 flex items-center justify-center text-blue-600 dark:text-blue-400 shadow-sm dark:shadow-black/10 border border-blue-100 dark:border-blue-900 shrink-0">
          <AlertCircle size={24} />
        </div>
        <div className="text-xs text-blue-900 dark:text-blue-200 leading-relaxed font-medium pt-1">
          <Typography className="font-bold uppercase tracking-[0.2em] mb-2 text-blue-950 dark:text-blue-100 opacity-80" variant="p">
            {t("common.did_you_know")}
          </Typography>
          <Typography className="opacity-70 font-bold uppercase tracking-tight" variant="p">
            {t("rules.info_priority")}
          </Typography>
        </div>
      </div>

      <ConfirmationDialog />
    </div>
  );
}
