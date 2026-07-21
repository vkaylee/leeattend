import { useNavigate } from "@tanstack/react-router";
import { Lock, ShieldCheck } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { LABEL_BASE } from "@/components/ui/form-field";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import { changePassword, clearAuthSession } from "@/lib/api";
import { ROUTES } from "@/lib/routes";

export const SecurityTab: React.FC = () => {
  const { t } = useTranslation();

  return (
    <div className="max-w-5xl bg-white dark:bg-slate-900 rounded-3xl sm:rounded-[3rem] border border-slate-100 dark:border-slate-800 shadow-sm p-6 sm:p-12 space-y-12 animate-in fade-in duration-500">
      <div className="flex items-center gap-5">
        <div className="w-14 h-14 rounded-2xl bg-orange-50 dark:bg-orange-950 flex items-center justify-center text-orange-600 dark:text-orange-300 shadow-inner border border-orange-100/50">
          <ShieldCheck size={28} />
        </div>
        <div>
          <Typography className="text-2xl font-bold text-slate-700 dark:text-slate-200 tracking-tighter uppercase leading-none mb-1" variant="h2">
            {t("profile.security.title")}
          </Typography>
          <Typography className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest opacity-70" variant="p">
            {t("profile.security.subtitle")}
          </Typography>
        </div>
      </div>
      <div className="bg-slate-50 dark:bg-slate-800/50 p-10 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 shadow-inner">
        <ChangePasswordForm />
      </div>
    </div>
  );
};

function ChangePasswordForm() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const toast = useToast();
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [formData, setFormData] = useState({ old_password: "", new_password: "", confirm_password: "" });

  async function handlePasswordSubmit(e: React.SyntheticEvent) {
    e.preventDefault();
    if (formData.new_password !== formData.confirm_password) {
      toast.warning(t("profile.security.messages.confirm_mismatch"));
      return;
    }
    setIsSubmitting(true);
    try {
      await changePassword({ old_password: formData.old_password, new_password: formData.new_password });
      toast.success(t("profile.security.messages.password_changed"));
      setTimeout(() => { 
        clearAuthSession(); 
        void navigate({ to: ROUTES.LOGIN }); 
      }, 2000);
    } catch (error: unknown) {
      toast.error(error instanceof Error ? error.message : t("profile.security.messages.change_error"));
    } finally {
      setIsSubmitting(false);
    }
  }

  const fields = [
    { id: "current", label: "current_password", key: "old_password" as const },
    { id: "new", label: "new_password", key: "new_password" as const },
    { id: "confirm", label: "confirm_password", key: "confirm_password" as const }
  ];

  return (
    <form className="space-y-8 max-w-lg" onSubmit={handlePasswordSubmit}>
      {fields.map((field) => (
        <div className="space-y-3 px-1" key={field.id}>
          <label className={LABEL_BASE}>{t(`profile.security.${field.label}`)}</label>
          <div className="relative group">
            <input 
              className="w-full bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-2xl px-6 py-4 focus:ring-4 focus:ring-orange-500/10 focus:border-orange-300 outline-none transition-all pr-14 font-bold text-sm tracking-tight shadow-sm" 
              data-testid={`${field.id}-password-input`} 
              onChange={(e) => { setFormData({ ...formData, [field.key]: e.target.value }); }} 
              required 
              type="password" 
              value={formData[field.key]} 
            />
            <Lock className="absolute right-5 top-1/2 -translate-y-1/2 text-slate-300 dark:text-slate-600 group-focus-within:text-orange-500 transition-colors" size={20} />
          </div>
        </div>
      ))}
      <div className="pt-6 border-t border-slate-100 dark:border-slate-800 flex justify-end">
        <Button data-testid="update-password-btn" loading={isSubmitting} size="lg" type="submit" variant="secondary">
          {t("profile.security.submit_btn").toUpperCase()}
        </Button>
      </div>
    </form>
  );
}
