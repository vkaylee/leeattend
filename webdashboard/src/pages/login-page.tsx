import { useNavigate, Link, useLocation } from "@tanstack/react-router";
import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { LABEL_BASE } from "@/components/ui/form-field";
import {
  getCurrentUserId,
  login as apiLogin,
} from "@/lib/api";
import { ROUTES } from "@/lib/routes";

export default function LoginPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const location = useLocation();
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);

  const from = (location.state as { from?: string } | null)?.from ?? ROUTES.HOME;

  const userId = getCurrentUserId();
 

  useEffect(() => {
    if (userId) {
      void navigate({ to: from, replace: true });
    }
  }, [userId, navigate, from]);

  if (userId) {
    return null;
  }

  const performLogin = async (loginEmail: string, loginPass: string) => {
    setIsSubmitting(true); setError(null);
    try {
      await apiLogin(loginEmail, loginPass);
      // Điều hướng về trang trước đó hoặc Dashboard
      void navigate({ to: from, replace: true });
    } catch (error_: unknown) {
      setError(error_ instanceof Error ? error_.message : t("auth.login_error"));
    } finally { setIsSubmitting(false); }
 
  };

  const handleLogin = async (e: React.SyntheticEvent) => { e.preventDefault(); await performLogin(email, password); };
  const _handleFastLogin = undefined; // Removed: no seeded admin credentials

  return (
    <div className="min-h-screen flex items-center justify-center bg-slate-50 dark:bg-slate-950 p-6 relative overflow-hidden animate-in fade-in duration-1000">
      <div className="absolute top-[-10%] left-[-10%] w-[50%] h-[50%] bg-primary-100 dark:bg-primary-900/30 rounded-full blur-[140px] animate-pulse duration-[12s]" />
      <div className="absolute bottom-[-10%] right-[-10%] w-[50%] h-[50%] bg-blue-100/30 dark:bg-blue-900/20 rounded-full blur-[140px] animate-pulse duration-[10s]" />
      
      <div className="max-w-md w-full space-y-12 p-12 bg-white dark:bg-slate-900 rounded-[3.5rem] shadow-premium-2xl relative z-10 animate-in zoom-in-95 duration-1000 border border-white/60">
        <div className="text-center">
          <div className="bg-slate-900 w-24 h-24 rounded-[2.5rem] flex items-center justify-center mx-auto mb-10 shadow-2xl shadow-slate-400 dark:shadow-black/40 transform hover:rotate-6 hover:scale-110 transition-all duration-700 group/logo">
             <span className="text-white text-5xl font-bold italic tracking-tighter group-hover:text-primary-400 transition-colors pl-3">L</span>
          </div>
          <h2 className="text-5xl font-bold text-slate-700 dark:text-slate-200 tracking-tighter uppercase leading-none px-4">Attend</h2>
          <p className="mt-4 text-slate-500 dark:text-slate-400 font-bold uppercase tracking-[0.3em] text-[10px] opacity-60 leading-relaxed">
            {t("auth.slogan").toUpperCase()}
          </p>
        </div>

        {error ? (
          <div className="bg-rose-50 dark:bg-rose-950 border-2 border-rose-100 dark:border-rose-800 text-rose-600 dark:text-rose-300 p-5 rounded-[2rem] text-[11px] font-bold uppercase tracking-widest text-center animate-in slide-in-from-top-4 shadow-inner" role="alert">
            {error}
          </div>
        ) : null}

        <div className="space-y-10">


          <form className="space-y-8" onSubmit={handleLogin}>
            <div className="space-y-6">
              <div className="space-y-3 px-1">
                <label className={LABEL_BASE}>{t("auth.email")}</label>
                <div className="relative group">
                  <input autoComplete="email" className="block w-full px-6 py-4.5 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 text-slate-700 dark:text-slate-200 rounded-[1.5rem] focus:outline-none focus:ring-4 focus:ring-primary-500/10 focus:border-primary-300 focus:bg-white dark:focus:bg-slate-900 transition-all font-bold placeholder:text-slate-300 dark:placeholder:text-slate-600 dark:text-slate-300 text-sm tracking-tight shadow-inner" data-testid="login-email-input" id="email-address" name="email" onChange={(e) => { setEmail(e.target.value); }} placeholder="email@company.com" required type="email" value={email} />
                </div>
              </div>
              <div className="space-y-3 px-1">
                <label className={LABEL_BASE}>{t("auth.password")}</label>
                <div className="relative group">
                  <input autoComplete="current-password" className="block w-full px-6 py-4.5 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 text-slate-700 dark:text-slate-200 rounded-[1.5rem] focus:outline-none focus:ring-4 focus:ring-primary-500/10 focus:border-primary-300 focus:bg-white dark:focus:bg-slate-900 transition-all font-bold placeholder:text-slate-300 dark:placeholder:text-slate-600 dark:text-slate-300 text-sm tracking-[0.3em] shadow-inner" data-testid="login-password-input" id="password" name="password" onChange={(e) => { setPassword(e.target.value); }} placeholder="••••••••" required type="password" value={password} />
                </div>
              </div>
            </div>

            <Button
              className="w-full"
              data-testid="login-submit-btn"
              disabled={isSubmitting}
              loading={isSubmitting}
              size="lg"
              type="submit"
              variant="secondary"
            >
              {isSubmitting ? t("auth.processing").toUpperCase() : t("auth.login_btn").toUpperCase()}
            </Button>

            <div className="text-center pt-2">
              <span className="text-slate-500 dark:text-slate-400 text-[10px] font-bold uppercase tracking-widest opacity-60">{t("auth.no_account")} </span>
              <Link className="font-bold text-primary-600 dark:text-primary-400 hover:text-primary-700 dark:text-primary-300 text-[10px] uppercase tracking-widest transition-colors ml-1" to={ROUTES.REGISTER}>{t("auth.register_now").toUpperCase()}</Link>
            </div>
          </form>
        </div>
      </div>
    </div>
  );
}
