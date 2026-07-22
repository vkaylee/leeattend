import { Globe, Layers, Plus, Trash2, Target, Route } from "lucide-react";
import React, { useState } from "react";

import Button from "@/components/ui/button";
import { Combobox } from "@/components/ui/combobox";
import EmptyState from "@/components/ui/empty-state";
import FormField from "@/components/ui/form-field";
import Modal from "@/components/ui/modal";
import ResponsiveTable from "@/components/ui/responsive-table";
import { Typography } from "@/components/ui/typography";
import { EContractRoutingRule } from "@/lib/api/econtracts";

interface EContractsRulesTabProps {
  isCreatingRule: boolean;
  isDeletingRule: boolean;
  isLoading: boolean;
  onCreateRule: (rule: Omit<EContractRoutingRule, "id" | "created_at">) => void;
  onDeleteRule: (ruleId: string) => void;
  rules: EContractRoutingRule[];
  selectedEmployerId: string;
}

export default function EContractsRulesTab({
  isCreatingRule,
  isDeletingRule,
  isLoading,
  onCreateRule,
  onDeleteRule,
  rules,
  selectedEmployerId,
}: EContractsRulesTabProps) {
  const [isRuleModalOpen, setIsRuleModalOpen] = useState(false);

  const employerRules = rules.filter((r) => r.employer_id === selectedEmployerId);

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center px-4">
        <div>
          <Typography
            className="text-sm font-bold uppercase tracking-widest text-slate-800 dark:text-slate-100 flex items-center gap-2"
            variant="h3"
          >
            <Globe className="text-primary-500" size={16} /> Signing Provider Routing Precedence Rules
          </Typography>
          <Typography
            className="text-[10px] text-slate-400 font-semibold uppercase tracking-wider mt-0.5"
            variant="p"
          >
            Route envelopes to specific signature providers based on country & verification criteria
          </Typography>
        </div>
        <Button onClick={() => { setIsRuleModalOpen(true); }} variant="outline">
          <Plus className="mr-1.5 h-4 w-4" /> ADD ROUTING RULE
        </Button>
      </div>

      <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-[2.5rem] overflow-hidden shadow-sm">
        {isLoading ? (
          <div className="p-8 text-center text-xs text-slate-500 animate-pulse">Loading routing rules...</div>
        ) : employerRules.length === 0 ? (
          <EmptyState
            action={
              <Button onClick={() => { setIsRuleModalOpen(true); }} variant="secondary">
                Create First Rule
              </Button>
            }
            description="No custom routing rules established for this Legal Entity. All envelopes default to Mock Provider."
            icon={<Layers size={40} />}
            title="No Precedence Rules Found"
          />
        ) : (
          <ResponsiveTable<EContractRoutingRule>
            data={employerRules.sort((a, b) => b.priority - a.priority)}
            getCardKey={(rule) => rule.id}
            isLoading={false}
            renderCard={(rule) => (
              <div className="rounded-2xl border border-slate-100 bg-white p-5 shadow-sm dark:border-slate-800 dark:bg-slate-900">
                {/* Header: Country + Provider */}
                <div className="flex items-start justify-between gap-3">
                  <div className="min-w-0 flex-1">
                    <div className="text-sm font-bold text-slate-800 dark:text-slate-200">
                      {rule.signer_country} → {rule.target_provider_type.toUpperCase()}
                    </div>
                    <div className="mt-1 text-[10px] font-bold uppercase tracking-wider text-slate-500 dark:text-slate-400">
                      {rule.signing_method}
                    </div>
                  </div>
                  <span className="shrink-0 rounded-lg bg-slate-100 px-2.5 py-1 text-[10px] font-bold text-slate-700 shadow-sm dark:bg-slate-800 dark:text-slate-300">
                    P{rule.priority}
                  </span>
                </div>

                {/* Detail grid */}
                <div className="mt-4 grid grid-cols-2 gap-3 border-t border-slate-100 pt-4 dark:border-slate-800">
                  <div className="flex items-center gap-2">
                    <Route className="h-3.5 w-3.5 text-slate-400" />
                    <span className="text-xs font-medium uppercase tracking-wider text-slate-600 dark:text-slate-400">
                      {rule.signer_country}
                    </span>
                  </div>
                  <div className="flex items-center gap-2">
                    <Target className="h-3.5 w-3.5 text-slate-400" />
                    <span className="text-xs font-medium uppercase tracking-wider text-slate-600 dark:text-slate-400">
                      {rule.target_provider_type}
                    </span>
                  </div>
                </div>

                {/* Actions */}
                <div className="mt-4 flex items-center justify-end border-t border-slate-100 pt-4 dark:border-slate-800">
                  <Button
                    aria-label="Delete rule"
                    className="text-slate-400 hover:text-rose-600 hover:bg-rose-50 dark:hover:bg-rose-955/40 p-2 rounded-xl h-auto w-auto"
                    loading={isDeletingRule}
                    onClick={() => { onDeleteRule(rule.id); }}
                    size="sm"
                    variant="ghost"
                  >
                    <Trash2 size={14} />
                  </Button>
                </div>
              </div>
            )}
            renderDesktopTable={() => (
              <table className="w-full text-left border-collapse">
                <thead>
                  <tr className="bg-slate-50 dark:bg-slate-800/10">
                    <th className="px-8 py-5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest border-b border-slate-50 dark:border-slate-800">
                      Signer Country
                    </th>
                    <th className="px-8 py-5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest border-b border-slate-50 dark:border-slate-800">
                      Signing Auth
                    </th>
                    <th className="px-8 py-5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest border-b border-slate-50 dark:border-slate-800">
                      Routed Provider
                    </th>
                    <th className="px-8 py-5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest border-b border-slate-50 dark:border-slate-800 text-center">
                      Priority
                    </th>
                    <th className="px-8 py-5 border-b border-slate-50 dark:border-slate-800"></th>
                  </tr>
                </thead>
                <tbody>
                  {employerRules
                    .sort((a, b) => b.priority - a.priority)
                    .map((rule) => (
                      <tr
                        className="border-b border-slate-50 dark:border-slate-800/50 hover:bg-slate-50/50 dark:hover:bg-slate-800/10 transition-colors"
                        key={rule.id}
                      >
                        <td className="px-8 py-6 text-xs font-mono font-bold text-slate-700 dark:text-slate-200">
                          {rule.signer_country}
                        </td>
                        <td className="px-8 py-6 text-xs font-bold text-slate-800 dark:text-slate-200 uppercase tracking-wider">
                          {rule.signing_method}
                        </td>
                        <td className="px-8 py-6 text-xs font-bold text-slate-800 dark:text-slate-200 uppercase tracking-wider">
                          {rule.target_provider_type}
                        </td>
                        <td className="px-8 py-6 text-center text-xs font-bold text-slate-800 dark:text-slate-200">
                          <span className="px-2.5 py-1 bg-slate-100 dark:bg-slate-800 rounded-lg shadow-sm">
                            {rule.priority}
                          </span>
                        </td>
                        <td className="px-8 py-6 text-right">
                          <Button
                            aria-label="Delete rule"
                            className="text-slate-400 hover:text-rose-600 hover:bg-rose-50 dark:hover:bg-rose-955/40 p-2 rounded-xl transition-colors h-auto w-auto shadow-none border-none animate-none"
                            loading={isDeletingRule}
                            onClick={() => { onDeleteRule(rule.id); }}
                            size="sm"
                            variant="ghost"
                          >
                            <Trash2 size={14} />
                          </Button>
                        </td>
                      </tr>
                    ))}
                </tbody>
              </table>
            )}
          />
        )}
      </div>

      <AddRuleModal
        isCreatingRule={isCreatingRule}
        isOpen={isRuleModalOpen}
        onClose={() => {
          setIsRuleModalOpen(false);
        }}
        onSubmit={onCreateRule}
        selectedEmployerId={selectedEmployerId}
      />
    </div>
  );
}

interface AddRuleModalProps {
  isCreatingRule: boolean;
  isOpen: boolean;
  onClose: () => void;
  onSubmit: (rule: Omit<EContractRoutingRule, "id" | "created_at">) => void;
  selectedEmployerId: string;
}

function AddRuleModal({
  isCreatingRule,
  isOpen,
  onClose,
  onSubmit,
  selectedEmployerId,
}: AddRuleModalProps) {
  const [newRuleForm, setNewRuleForm] = useState<Partial<EContractRoutingRule>>({
    signer_country: "VN",
    signing_method: "smartca",
    target_provider_type: "smartca",
    priority: 1,
  });

  const handleCreateRuleSubmit = (e: React.SyntheticEvent) => {
    e.preventDefault();
    if (!selectedEmployerId) return;
    onSubmit({
      ...newRuleForm,
      employer_id: selectedEmployerId,
    } as Omit<EContractRoutingRule, "id" | "created_at">);
    onClose();
  };

  return (
    <Modal
      icon={<Plus size={24} />}
      isOpen={isOpen}
      onClose={onClose}
      subtitle="Add specific routing configuration priority rules"
      title="Add Routing Rule"
    >
      <form className="space-y-6 pt-4" noValidate onSubmit={handleCreateRuleSubmit}>
        <div className="grid grid-cols-2 gap-4">
          <FormField
            id="rule-country-input"
            label="Signer Country Code (e.g. VN, US)"
            onChange={(e) => {
              setNewRuleForm((prev) => ({ ...prev, signer_country: e.target.value.toUpperCase() }));
            }}
            required
            value={newRuleForm.signer_country ?? ""}
          />
          <div className="space-y-2">
            <label
              className="text-[11px] font-bold text-slate-500 uppercase tracking-widest block cursor-pointer"
              htmlFor="rule-auth-select"
            >
              Preferred Signing Auth
            </label>
            <Combobox
              id="rule-auth-select"
              onChange={(val) => {
                setNewRuleForm((prev) => ({
                  ...prev,
                  signing_method: val as EContractRoutingRule["signing_method"],
                }));
              }}
              options={[
                { value: "smartca", label: "SmartCA Remote OTP" },
                { value: "sms", label: "SMS OTP verification" },
                { value: "email", label: "Email OTP verification" },
                { value: "hsm", label: "HSM Enterprise Sign" },
              ]}
              placeholder="Select Signing Auth"
              value={newRuleForm.signing_method ?? "smartca"}
            />
          </div>
        </div>

        <div className="grid grid-cols-2 gap-4">
          <div className="space-y-2">
            <label
              className="text-[11px] font-bold text-slate-500 uppercase tracking-widest block cursor-pointer"
              htmlFor="rule-provider-select"
            >
              Target Provider Route
            </label>
            <Combobox
              id="rule-provider-select"
              onChange={(val) => {
                setNewRuleForm((prev) => ({ ...prev, target_provider_type: val }));
              }}
              options={[
                { value: "smartca", label: "VN SmartCA" },
                { value: "hsm", label: "Corporate HSM" },
                { value: "fpt", label: "FPT CeCA" },
                { value: "docusign", label: "DocuSign" },
                { value: "mock", label: "Mock Sandbox" },
              ]}
              placeholder="Select Provider Route"
              value={newRuleForm.target_provider_type ?? "smartca"}
            />
          </div>
          <FormField
            id="rule-priority-input"
            label="Priority Weight (Integer)"
            onChange={(e) => {
              setNewRuleForm((prev) => ({ ...prev, priority: parseInt(e.target.value) || 1 }));
            }}
            required
            type="number"
            value={String(newRuleForm.priority ?? 1)}
          />
        </div>

        <div className="flex items-center justify-end gap-3 pt-6 border-t border-slate-100 dark:border-slate-800">
          <Button onClick={onClose} type="button" variant="outline">
            CANCEL
          </Button>
          <Button loading={isCreatingRule} type="submit" variant="secondary">
            ADD RULE
          </Button>
        </div>
      </form>
    </Modal>
  );
}
