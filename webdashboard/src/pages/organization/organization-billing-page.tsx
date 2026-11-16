import { CreditCard, CheckCircle2 } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { PageHeader } from "@/components/ui/page-header";
import { Typography } from "@/components/ui/typography";

export default function OrganizationBillingPage() {
  const { t } = useTranslation();
  return (
    <div className="space-y-6 animate-in fade-in duration-300">
      <PageHeader
        subtitle={t("billing.subtitle", { defaultValue: "Quản lý gói dịch vụ và hóa đơn của tổ chức" })}
        title={t("billing.title", { defaultValue: "Gói & Thanh toán" })}
      />

      <div className="bg-white dark:bg-slate-900/90 rounded-2xl border border-slate-200/80 dark:border-slate-800/80 p-8 sm:p-12 shadow-2xs flex flex-col items-center justify-center text-center max-w-xl mx-auto my-6">
        <div className="w-14 h-14 rounded-2xl bg-emerald-50 dark:bg-emerald-950/70 border border-emerald-100/50 dark:border-emerald-900/50 flex items-center justify-center text-emerald-600 dark:text-emerald-400 shadow-2xs mb-4">
          <CreditCard size={28} />
        </div>
        <Typography className="text-xl font-bold text-slate-800 dark:text-slate-100 mb-2" variant="h3">
          {t("billing.comingSoon", { defaultValue: "Tính năng Thanh toán" })}
        </Typography>
        <Typography className="text-xs font-medium text-slate-500 dark:text-slate-400 max-w-md leading-relaxed mb-6" variant="p">
          {t("billing.comingSoonDesc", { defaultValue: "Tính năng quản lý gói thành viên, hạn mức nhân sự và xuất hóa đơn VAT tự động sẽ có mặt trong phiên bản tiếp theo." })}
        </Typography>

        <div className="w-full grid grid-cols-1 sm:grid-cols-2 gap-3 text-left">
          <div className="flex items-center gap-2 p-3 rounded-xl bg-slate-50 dark:bg-slate-800/60 border border-slate-100 dark:border-slate-800 text-xs font-medium text-slate-700 dark:text-slate-300">
            <CheckCircle2 className="w-4 h-4 text-emerald-500 shrink-0" />
            <span>Thanh toán định kỳ linh hoạt</span>
          </div>
          <div className="flex items-center gap-2 p-3 rounded-xl bg-slate-50 dark:bg-slate-800/60 border border-slate-100 dark:border-slate-800 text-xs font-medium text-slate-700 dark:text-slate-300">
            <CheckCircle2 className="w-4 h-4 text-emerald-500 shrink-0" />
            <span>Hóa đơn điện tử VAT</span>
          </div>
        </div>
      </div>
    </div>
  );
}
