import { useNavigate, Navigate, Link } from "react-router-dom";
import { ROUTES } from "@/lib/routes";
import { useTranslation } from "react-i18next";
import { useProfile } from "@/hooks/use-profile";
import {
  setCurrentUserId,
  getCurrentUserId,
  setAccessToken,
  setRefreshToken,
  login as apiLogin,
} from "@/lib/api";
import { useState } from "react";
import { debugProps } from "@/lib/utils";
import Button from "@/components/ui/Button";
import { LABEL_BASE } from "@/components/ui/FormField";

export default function LoginPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);

  const userId = getCurrentUserId();

  if (userId) return <Navigate to={ROUTES.HOME} replace />;

  const performLogin = async (loginEmail: string, loginPass: string) => {
    setIsSubmitting(true); setError(null);
    try {
      await apiLogin(loginEmail, loginPass);
      navigate(ROUTES.HOME, { replace: true });
      window.location.reload();
    } catch (err: any) {
      setError(err.message || t("auth.login_error"));
    } finally { setIsSubmitting(false); }
  };

  const handleLogin = async (e: React.FormEvent) => { e.preventDefault(); await performLogin(email, password); };
  const handleFastLogin = async () => { const devEmail = "admin@leeattend.com"; const devPass = "admin123"; setEmail(devEmail); setPassword(devPass); await performLogin(devEmail, devPass); };

  return (
    <div className="min-h-screen flex items-center justify-center bg-slate-50 dark:bg-slate-950 p-6 relative overflow-hidden animate-in fade-in duration-1000">
      <div className="absolute top-[-10%] left-[-10%] w-[50%] h-[50%] bg-primary-100 dark:bg-primary-900/30 rounded-full blur-[140px] animate-pulse duration-[12s]" />
      <div className="absolute bottom-[-10%] right-[-10%] w-[50%] h-[50%] bg-indigo-100/30 dark:bg-indigo-900/20 rounded-full blur-[140px] animate-pulse duration-[10s]" />
      
      <div className="max-w-md w-full space-y-12 p-12 bg-white dark:bg-slate-900 rounded-[3.5rem] shadow-premium-2xl relative z-10 animate-in zoom-in-95 duration-1000 border border-white/60">
        <div className="text-center">
          <div className="bg-slate-900 w-24 h-24 rounded-[2.5rem] flex items-center justify-center mx-auto mb-10 shadow-2xl shadow-slate-400 dark:shadow-black/40 transform hover:rotate-6 hover:scale-110 transition-all duration-700 group/logo">
             <span className="text-white text-5xl font-bold italic tracking-tighter group-hover:text-primary-400 transition-colors">L</span>
          </div>
          <h2 className="text-5xl font-bold text-slate-700 dark:text-slate-200 tracking-tighter uppercase leading-none">Attend</h2>
          <p className="mt-4 text-slate-500 dark:text-slate-400 font-bold uppercase tracking-[0.3em] text-[10px] opacity-60 leading-relaxed">
            {t("auth.slogan").toUpperCase()}
          </p>
        </div>

        {error && (
          <div role="alert" className="bg-rose-50 dark:bg-rose-950 border-2 border-rose-100 dark:border-rose-800 text-rose-600 dark:text-rose-300 p-5 rounded-[2rem] text-[11px] font-bold uppercase tracking-widest text-center animate-in slide-in-from-top-4 shadow-inner">
            {error}
          </div>
        )}

        <div className="space-y-10">
          <button
            onClick={handleFastLogin}
            {...debugProps({ devEmail: "admin@leeattend.com", devPass: "admin123" })}
            data-testid="fast-login-btn"
            className="group relative w-full flex justify-center py-4 px-4 border-2 border-primary-100 dark:border-primary-800 text-[10px] font-bold rounded-2xl text-primary-600 dark:text-primary-400 bg-primary-50 dark:bg-primary-950/20 hover:bg-primary-50 dark:bg-primary-950 hover:scale-[1.02] active:scale-95 transition-all uppercase tracking-[0.2em] shadow-sm hover:shadow-lg"
          >
            {t("auth.fast_login").toUpperCase()}
          </button>

          <div className="relative">
            <div className="absolute inset-0 flex items-center"><div className="w-full border-t border-slate-100 dark:border-slate-800" /></div>
            <div className="relative flex justify-center text-[9px] font-bold uppercase tracking-[0.4em] text-slate-300 dark:text-slate-600"><span className="px-6 bg-white dark:bg-slate-900">{t("auth.or_use_account")}</span></div>
          </div>

          <form className="space-y-8" onSubmit={handleLogin}>
            <div className="space-y-6">
              <div className="space-y-3 px-1">
                <label className={LABEL_BASE}>{t("auth.email")}</label>
                <div className="relative group">
                  <input id="email-address" data-testid="login-email-input" name="email" type="email" autoComplete="email" required value={email} onChange={(e) => setEmail(e.target.value)} className="block w-full px-6 py-4.5 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 text-slate-700 dark:text-slate-200 rounded-[1.5rem] focus:outline-none focus:ring-4 focus:ring-primary-500/10 focus:border-primary-300 focus:bg-white dark:focus:bg-slate-900 transition-all font-bold placeholder:text-slate-300 dark:placeholder:text-slate-600 dark:text-slate-300 text-sm uppercase tracking-tight shadow-inner" placeholder="admin@leeattend.com" />
                </div>
              </div>
              <div className="space-y-3 px-1">
                <label className={LABEL_BASE}>{t("auth.password")}</label>
                <div className="relative group">
                  <input id="password" data-testid="login-password-input" name="password" type="password" autoComplete="current-password" required value={password} onChange={(e) => setPassword(e.target.value)} className="block w-full px-6 py-4.5 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 text-slate-700 dark:text-slate-200 rounded-[1.5rem] focus:outline-none focus:ring-4 focus:ring-primary-500/10 focus:border-primary-300 focus:bg-white dark:focus:bg-slate-900 transition-all font-bold placeholder:text-slate-300 dark:placeholder:text-slate-600 dark:text-slate-300 text-sm tracking-[0.3em] shadow-inner" placeholder="••••••••" />
                </div>
              </div>
            </div>

            <Button
              type="submit"
              disabled={isSubmitting}
              loading={isSubmitting}
              data-testid="login-submit-btn"
              variant="secondary"
              size="lg"
              className="w-full"
            >
              {isSubmitting ? t("auth.processing").toUpperCase() : t("auth.login_btn").toUpperCase()}
            </Button>

            <div className="text-center pt-2">
              <span className="text-slate-500 dark:text-slate-400 text-[10px] font-bold uppercase tracking-widest opacity-60">{t("auth.no_account")} </span>
              <Link to={ROUTES.REGISTER} className="font-bold text-primary-600 dark:text-primary-400 hover:text-primary-700 dark:text-primary-300 text-[10px] uppercase tracking-widest transition-colors ml-1">{t("auth.register_now").toUpperCase()}</Link>
            </div>
          </form>
        </div>
      </div>
    </div>
  );
}
