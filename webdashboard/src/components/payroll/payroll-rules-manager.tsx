import { useQuery, useQueryClient } from "@tanstack/react-query";
import { Plus, FileText } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import Modal from "@/components/ui/modal";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import { fetchPayrollRules } from "@/lib/api/payroll";
import { createQueryKey } from "@/lib/query-keys";

import { PayrollRuleForm } from "./payroll-rule-form";
import { RuleRow } from "./payroll-rule-list";


export default function PayrollRulesManager() {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const toast = useToast();
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [expandedRuleId, setExpandedRuleId] = useState<string | null>(null);

  const { data: rules, isLoading } = useQuery({ queryKey: createQueryKey("payroll-rules"), queryFn: () => fetchPayrollRules() });

  return (
    <div className="animate-in fade-in duration-700 space-y-6">
      <div className="flex items-center justify-between pb-6 border-b border-slate-100 dark:border-slate-800">
        <div className="flex items-center gap-4">
          <div className="w-12 h-12 rounded-[1.25rem] bg-rose-50 dark:bg-rose-950 flex items-center justify-center text-rose-600 dark:text-rose-300 shadow-inner border border-rose-100 dark:border-rose-800/50"><FileText size={24} /></div>
          <div>
            <Typography className="text-xl font-bold text-slate-800 dark:text-slate-100 tracking-tight uppercase leading-none" variant="h2">{t("payroll.statutory_rules.title")}</Typography>
            <Typography className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest mt-1.5 opacity-70" variant="p">{t("payroll.statutory_rules.subtitle")}</Typography>
          </div>
        </div>
        <Button className="flex items-center gap-3 bg-primary-600 text-white px-8 py-3 rounded-2xl font-bold text-xs uppercase tracking-widest hover:bg-primary-700 transition-all shadow-xl shadow-primary-100 dark:shadow-black/30 active:scale-95" onClick={() => { setIsModalOpen(true); }} variant="ghost"><Plus size={18} /> {t("common.add").toUpperCase()}</Button>
      </div>

      <div className="divide-y divide-slate-50 dark:divide-slate-800 px-2">
        {isLoading ? (
          <div className="p-20 text-center" data-testid="loader"><div className="w-10 h-10 border-4 border-primary-600 border-t-transparent rounded-full animate-spin mx-auto"></div></div>
        ) : rules?.length === 0 ? (
          <div className="p-32 text-center bg-slate-50 dark:bg-slate-800/30 m-8 rounded-[3rem] border-2 border-dashed border-slate-100 dark:border-slate-800 flex flex-col items-center">
            <div className="w-20 h-20 bg-white dark:bg-slate-900 rounded-full flex items-center justify-center shadow-sm mb-4 border border-slate-100 dark:border-slate-800"><FileText className="text-slate-200 dark:text-slate-700" size={32} /></div>
            <Typography className="text-[10px] font-bold text-slate-300 dark:text-slate-600 uppercase tracking-widest" variant="p">{t("payroll.statutory_rules.no_rules")}</Typography>
          </div>
        ) : (
          rules?.map((rule) => (
            <RuleRow expandedRuleId={expandedRuleId}
            key={rule.id}
            rule={rule}
            setExpandedRuleId={setExpandedRuleId}  />
          ))
        )}
      </div>

      <Modal icon={<FileText size={24} />} isOpen={isModalOpen} maxWidth="lg" onClose={() => { setIsModalOpen(false); }} subtitle={t("payroll.statutory_rules.create_rule_desc")} title={t("payroll.statutory_rules.add_rule")}>
        <PayrollRuleForm
        onCancel={() => { setIsModalOpen(false); }}
        onSuccess={() => {
          toast.success(t("common.success"));
          void queryClient.invalidateQueries({ queryKey: createQueryKey("payroll-rules") });
          setIsModalOpen(false);
        }}  />
      </Modal>
    </div>
  );
}
