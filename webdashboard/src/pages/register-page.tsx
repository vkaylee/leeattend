import { useNavigate, Link } from "@tanstack/react-router";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { LABEL_BASE } from "@/components/ui/form-field";
import {
  register,
  setAccessToken,
  setRefreshToken,
  setCurrentUserId,
  getCurrentUserId,
  AuthResponse,
} from "@/lib/api";
import { ROUTES } from "@/lib/routes";

export default function RegisterPage() {
  const { t } = useTranslation();
  const userId = getCurrentUserId();
  const navigate = useNavigate();
  const [error, setError] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [formData, setFormData] = useState({ email: "", full_name: "", password: "", confirm_password: "" });

  if (userId) {
    void navigate({ to: ROUTES.HOME, replace: true });
    return null;
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault(); setError(null);
    if (formData.password !== formData.confirm_password) { setError(t("auth.password_mismatch")); return; }
    if (formData.password.length < 6) { setError(t("auth.password_too_short")); return; }
    setIsLoading(true);
    try {
      const response: AuthResponse = await register({ email: formData.email, password: formData.password, full_name: formData.full_name });
      setCurrentUserId(response.user.id); setAccessToken(response.access_token); setRefreshToken(response.refresh_token);
      window.location.href = ROUTES.ONBOARDING;
    } catch (error_: unknown) {
      const message = error_ instanceof Error ? error_.message : String(error_);
      setError(message || t("auth.registration_failed"));
    } finally { setIsLoading(false); }
  };

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => { setFormData((previous) => ({ ...previous, [e.target.name]: e.target.value })); };

  return (
    <div className="min-h-screen flex items-center justify-center bg-slate-50 dark:bg-slate-950 p-6 relative overflow-hidden animate-in fade-in duration-1000">
      <div className="absolute top-[-10%] right-[-10%] w-[50%] h-[50%] bg-primary-100 dark:bg-primary-900/30 rounded-full blur-[140px] animate-pulse duration-[12s]" />
      <div className="absolute bottom-[-10%] left-[-10%] w-[50%] h-[50%] bg-indigo-100/30 dark:bg-indigo-900/20 rounded-full blur-[140px] animate-pulse duration-[10s]" />
      
      <div className="max-w-md w-full space-y-10 p-12 bg-white dark:bg-slate-900 rounded-[3.5rem] shadow-premium-2xl relative z-10 animate-in zoom-in-95 duration-1000 border border-white/60">
        <div className="text-center">
          <div className="bg-slate-900 w-20 h-20 rounded-[2rem] flex items-center justify-center mx-auto mb-8 shadow-2xl shadow-slate-400 dark:shadow-black/40 transform hover:rotate-6 hover:scale-110 transition-all duration-700 group/logo">
             <span className="text-white text-4xl font-bold italic tracking-tighter group-hover:text-primary-400 transition-colors">L</span>
          </div>
          <h2 className="text-4xl font-bold text-slate-700 dark:text-slate-200 tracking-tighter uppercase leading-none">{t("auth.register_title")}</h2>
          <p className="mt-4 text-slate-500 dark:text-slate-400 font-bold uppercase tracking-[0.3em] text-[9px] opacity-60 leading-relaxed">
            {t("auth.slogan").toUpperCase()}
          </p>
        </div>

        {!!error && (
          <div className="bg-rose-50 dark:bg-rose-950 border-2 border-rose-100 dark:border-rose-800 text-rose-600 dark:text-rose-300 p-5 rounded-[2rem] text-[11px] font-bold uppercase tracking-widest text-center animate-in slide-in-from-top-4 shadow-inner" role="alert">
            {error}
          </div>
        )}

        <form className="space-y-8" onSubmit={handleSubmit}>
          <div className="space-y-5">
            {[
              { id: "full_name", label: "display_name", type: "text", placeholder: "Nguyen Van A" },
              { id: "email", label: "email", type: "email", placeholder: "email@congty.com", auto: "email" },
              { id: "password", label: "password", type: "password", placeholder: "••••••••", auto: "new-password" },
              { id: "confirm_password", label: "confirm_password", type: "password", placeholder: "••••••••", auto: "new-password" }
            ].map((field) => (
              <div className="space-y-2 px-1" key={field.id}>
                <label className={LABEL_BASE} htmlFor={field.id}>{t(`auth.${field.label ?? ""}`)}</label>
                <input autoComplete={field.auto} className="block w-full px-6 py-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 text-slate-700 dark:text-slate-200 rounded-[1.5rem] focus:outline-none focus:ring-4 focus:ring-primary-500/10 focus:border-primary-300 focus:bg-white dark:focus:bg-slate-900 transition-all font-bold placeholder:text-slate-300 dark:placeholder:text-slate-600 dark:text-slate-300 text-sm tracking-tight shadow-inner" id={field.id} name={field.id} onChange={handleChange} placeholder={field.placeholder} required type={field.type} value={formData[field.id as keyof typeof formData]} />
              </div>
            ))}
          </div>

          <div className="pt-2">
            <Button className="w-full" disabled={isLoading} loading={isLoading} size="lg" type="submit" variant="secondary">
              {isLoading ? t("auth.creating_account").toUpperCase() : t("auth.register_btn").toUpperCase()}
            </Button>
          </div>

          <div className="text-center pt-2">
            <span className="text-slate-500 dark:text-slate-400 text-[10px] font-bold uppercase tracking-widest opacity-60">{t("auth.already_have_account")} </span>
            <Link className="font-bold text-primary-600 dark:text-primary-400 hover:text-primary-700 dark:text-primary-300 text-[10px] uppercase tracking-widest transition-colors ml-1" to={ROUTES.LOGIN}>{t("auth.login.title").toUpperCase()}</Link>
          </div>
        </form>
      </div>
    </div>
  );
}
