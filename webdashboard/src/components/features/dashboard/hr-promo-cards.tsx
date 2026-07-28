import { Link } from "@tanstack/react-router";
import { Users2, Banknote, ArrowRight } from "lucide-react";

import { type TFunction } from "i18next";

import CollapsibleSection from "@/components/features/dashboard/collapsible-section";
import PromoCard from "@/components/ui/promo-card";
import { ROUTES } from "@/lib/routes";

interface HrPromoCardsProps {
  isCollapsed: boolean;
  onToggle: (id: string) => void;
  t: TFunction;
}

export function HrPromoCards({ isCollapsed, onToggle, t }: HrPromoCardsProps) {
  return (
    <CollapsibleSection
      id="actions"
      isCollapsed={isCollapsed}
      onToggle={onToggle}
      title={t("dashboard.sections.actions")}
    >
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
        <PromoCard
          buttonText={t("dashboard.hr.hr_management_btn").toUpperCase()}
          description={t("dashboard.hr.hr_management_desc")}
          eyebrow={t("dashboard.hr.hr_management")}
          linkElement={
            <Link
              className="bg-white dark:bg-slate-900 text-slate-700 dark:text-slate-200 px-8 sm:px-10 py-3 sm:py-4 rounded-2xl font-bold text-xs uppercase tracking-widest transition-all inline-flex items-center gap-4 hover:gap-6 shadow-2xl hover:shadow-primary-500/20 active:scale-95"
              to={ROUTES.EMPLOYEES}
            >
              {t("dashboard.hr.hr_management_btn").toUpperCase()} <ArrowRight size={20} />
            </Link>
          }
          title={t("dashboard.hr.hr_management_title")}
          variant="dark"
          watermarkIcon={Users2}
        />
        <PromoCard
          buttonText={t("dashboard.hr.payroll_btn").toUpperCase()}
          description={t("dashboard.hr.payroll_desc")}
          eyebrow={t("dashboard.hr.payroll")}
          linkElement={
            <Link
              className="bg-slate-900 text-white px-8 sm:px-10 py-3 sm:py-4 rounded-2xl font-bold text-xs uppercase tracking-widest transition-all inline-flex items-center gap-4 hover:gap-6 shadow-2xl shadow-black/20 active:scale-95"
              search={{ tab: "recorded", mode: "this_month" }}
              to={ROUTES.ATTENDANCE_LOGS}
            >
              {t("dashboard.hr.payroll_btn").toUpperCase()} <ArrowRight size={20} />
            </Link>
          }
          title={t("dashboard.hr.payroll_title")}
          variant="primary"
          watermarkIcon={Banknote}
        />
      </div>
    </CollapsibleSection>
  );
}
