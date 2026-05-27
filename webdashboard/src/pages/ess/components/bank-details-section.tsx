import { Building2, CreditCard, MapPin, Save } from "lucide-react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { LABEL_BASE } from "@/components/ui/form-field";

export interface BankDetailsFormData {
  bank_account_number: string;
  bank_branch: string;
  bank_name: string;
}

interface BankDetailsSectionProps {
  formData: BankDetailsFormData;
  handleSubmit: (e: React.SyntheticEvent) => Promise<void>;
  isUpdating: boolean;
  setFormData: (data: BankDetailsFormData) => void;
}

export function BankDetailsSection({
  formData,
  handleSubmit,
  isUpdating,
  setFormData,
}: BankDetailsSectionProps) {
  const { t } = useTranslation();

  return (
    <form
      className="max-w-5xl bg-white dark:bg-slate-900 rounded-[3rem] border border-slate-100 dark:border-slate-800 shadow-sm p-12 space-y-12 animate-in fade-in duration-500"
      onSubmit={(e) => {
        void handleSubmit(e);
      }}
    >
      <div className="grid grid-cols-1 md:grid-cols-3 gap-10">
        {[
          {
            icon: Building2,
            key: "bank_name",
            label: "bank_name",
            placeholder: "Vietcombank",
          },
          {
            icon: CreditCard,
            key: "bank_account_number",
            label: "account_number",
            placeholder: "xxxx xxxx xxxx",
          },
          {
            icon: MapPin,
            key: "bank_branch",
            label: "bank_branch",
            placeholder: t("ess.branch_placeholder"),
          },
        ].map((field) => (
          <div className="space-y-3 px-1" key={field.key}>
            <label className={LABEL_BASE}>
              <field.icon className="inline mr-2" size={12} /> {t(`ess.${field.label}`)}
            </label>
            <input
              className="w-full px-6 py-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-bold text-sm tracking-tight shadow-sm font-mono"
              onChange={(e) => {
                setFormData({ ...formData, [field.key]: e.target.value });
              }}
              placeholder={field.placeholder}
              type="text"
              value={(formData as unknown as Record<string, string>)[field.key] || ""}
            />
          </div>
        ))}
      </div>
      <div className="pt-8 border-t border-slate-50 dark:border-slate-800 flex justify-end">
        <Button disabled={isUpdating}
          icon={<Save size={18} />}
          loading={isUpdating}
          type="submit"
          variant="primary"
        >
          {isUpdating ? t("common.saving").toUpperCase() : t("common.save").toUpperCase()}
        </Button>
      </div>
    </form>
  );
}
