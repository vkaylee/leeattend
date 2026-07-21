import { Calendar, MapPin, Phone, Save, ShieldCheck } from "lucide-react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { Combobox } from "@/components/ui/combobox";
import { LABEL_BASE } from "@/components/ui/form-field";

export interface PersonalInfoFormData {
  [key: string]: string;
  address: string;
  date_of_birth: string;
  gender: string;
  national_id: string;
  phone: string;
}

interface PersonalInfoSectionProps {
  formData: PersonalInfoFormData;
  handleSubmit: (e: React.SyntheticEvent) => Promise<void>;
  isUpdating: boolean;
  setFormData: (data: PersonalInfoFormData) => void;
}

export function PersonalInfoSection({
  formData,
  handleSubmit,
  isUpdating,
  setFormData,
}: PersonalInfoSectionProps) {
  const { t } = useTranslation();

  return (
    <form
      className="max-w-5xl bg-white dark:bg-slate-900 rounded-3xl sm:rounded-[3rem] border border-slate-100 dark:border-slate-800 shadow-sm p-6 sm:p-12 space-y-12 animate-in fade-in duration-500"
      onSubmit={(e) => {
        void handleSubmit(e);
      }}
    >
      <div className="grid grid-cols-1 md:grid-cols-2 gap-10">
        {[
          {
            icon: Phone,
            key: "phone",
            label: "phone",
            placeholder: "+84 xxx xxx xxx",
            type: "tel",
          },
          {
            icon: Calendar,
            key: "date_of_birth",
            label: "date_of_birth",
            type: "date",
          },
          {
            icon: ShieldCheck,
            key: "national_id",
            label: "national_id",
            placeholder: "xxx xxx xxx xxx",
          },
        ].map((field) => (
          <div className="space-y-3 px-1" key={field.key}>
            <label className={LABEL_BASE}>
              <field.icon className="inline mr-2" size={12} /> {t(`ess.${field.label}`)}
            </label>
            <input
              className="w-full px-6 py-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-bold text-sm tracking-tight shadow-sm"
              onChange={(e) => {
                setFormData({ ...formData, [field.key]: e.target.value });
              }}
              placeholder={field.placeholder}
              type={field.type}
              value={formData[field.key]}
            />
          </div>
        ))}
        <div className="space-y-3 px-1">
          <label className={LABEL_BASE}>{t("ess.gender")}</label>
          <Combobox
            className="w-full px-4 py-2.5 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-bold text-sm uppercase tracking-tight shadow-sm"
            onChange={(value) => {
              setFormData({ ...formData, gender: value });
            }}
            options={[
              { label: t("ess.gender_male").toUpperCase(), value: "male" },
              { label: t("ess.gender_female").toUpperCase(), value: "female" },
              { label: t("ess.gender_other").toUpperCase(), value: "other" },
            ]}
            placeholder={t("common.select").toUpperCase()}
            value={formData.gender}
          />
        </div>
        <div className="space-y-3 md:col-span-2 px-1">
          <label className={LABEL_BASE}>
            <MapPin className="inline mr-2" size={12} /> {t("ess.address")}
          </label>
          <textarea
            className="w-full px-6 py-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-medium text-sm resize-none shadow-sm"
            onChange={(e) => {
              setFormData({ ...formData, address: e.target.value });
            }}
            placeholder={t("ess.address_placeholder")}
            rows={2}
            value={formData.address}
          />
        </div>
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
