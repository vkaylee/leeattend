import { useNavigate, Navigate, Link } from "react-router-dom";
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

export default function LoginPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);

  const userId = getCurrentUserId();

  if (userId) return <Navigate to="/" replace />;

  const performLogin = async (loginEmail: string, loginPass: string) => {
    setIsSubmitting(true); setError(null);
    try {
      await apiLogin(loginEmail, loginPass);
      navigate("/", { replace: true });
      window.location.reload();
    } catch (err: any) {
      setError(err.message || t("auth.login_error"));
    } finally { setIsSubmitting(false); }
  };

  const handleLogin = async (e: React.FormEvent) => { e.preventDefault(); await performLogin(email, password); };
  const handleFastLogin = async () => { const devEmail = "admin@leeattend.com"; const devPass = "admin123"; setEmail(devEmail); setPassword(devPass); await performLogin(devEmail, devPass); };

  return (
    <div className="min-h-screen flex items-center justify-center bg-[#f8fafc] p-6 relative overflow-hidden animate-in fade-in duration-1000">
      <div className="absolute top-[-10%] left-[-10%] w-[50%] h-[50%] bg-primary-100/30 rounded-full blur-[140px] animate-pulse duration-[12s]" />
      <div className="absolute bottom-[-10%] right-[-10%] w-[50%] h-[50%] bg-indigo-100/30 rounded-full blur-[140px] animate-pulse duration-[10s]" />
      
      <div className="max-w-md w-full space-y-12 p-12 bg-white rounded-[3.5rem] shadow-premium-2xl relative z-10 animate-in zoom-in-95 duration-1000 border border-white/60">
        <div className="text-center">
          <div className="bg-slate-900 w-24 h-24 rounded-[2.5rem] flex items-center justify-center mx-auto mb-10 shadow-2xl shadow-slate-400 transform hover:rotate-6 hover:scale-110 transition-all duration-700 group/logo">
             <span className="text-white text-5xl font-bold italic tracking-tighter group-hover:text-primary-400 transition-colors">L</span>
          </div>
          <h2 className="text-5xl font-bold text-slate-700 tracking-tighter uppercase leading-none">Attend</h2>
          <p className="mt-4 text-slate-500 font-bold uppercase tracking-[0.3em] text-[10px] opacity-60 leading-relaxed">
            {t("auth.slogan").toUpperCase()}
          </p>
        </div>

        {error && (
          <div role="alert" className="bg-rose-50 border-2 border-rose-100 text-rose-600 p-5 rounded-[2rem] text-[11px] font-bold uppercase tracking-widest text-center animate-in slide-in-from-top-4 shadow-inner">
            {error}
          </div>
        )}

        <div className="space-y-10">
          <button
            onClick={handleFastLogin}
            {...debugProps({ devEmail: "admin@leeattend.com", devPass: "admin123" })}
            data-testid="fast-login-btn"
            className="group relative w-full flex justify-center py-4 px-4 border-2 border-primary-100 text-[10px] font-bold rounded-2xl text-primary-600 bg-primary-50/20 hover:bg-primary-50 hover:scale-[1.02] active:scale-95 transition-all uppercase tracking-[0.2em] shadow-sm hover:shadow-lg"
          >
            {t("auth.fast_login").toUpperCase()}
          </button>

          <div className="relative">
            <div className="absolute inset-0 flex items-center"><div className="w-full border-t border-slate-100" /></div>
            <div className="relative flex justify-center text-[9px] font-bold uppercase tracking-[0.4em] text-slate-300"><span className="px-6 bg-white">{t("auth.or_use_account")}</span></div>
          </div>

          <form className="space-y-8" onSubmit={handleLogin}>
            <div className="space-y-6">
              <div className="space-y-3 px-1">
                <label className="text-[10px] font-bold text-slate-500 uppercase tracking-widest ml-1 opacity-70">{t("auth.email")}</label>
                <div className="relative group">
                  <input id="email-address" data-testid="login-email-input" name="email" type="email" autoComplete="email" required value={email} onChange={(e) => setEmail(e.target.value)} className="block w-full px-6 py-4.5 bg-slate-50 border border-slate-100 text-slate-700 rounded-[1.5rem] focus:outline-none focus:ring-4 focus:ring-primary-500/10 focus:border-primary-300 focus:bg-white transition-all font-bold placeholder:text-slate-300 text-sm uppercase tracking-tight shadow-inner" placeholder="admin@leeattend.com" />
                </div>
              </div>
              <div className="space-y-3 px-1">
                <label className="text-[10px] font-bold text-slate-500 uppercase tracking-widest ml-1 opacity-70">{t("auth.password")}</label>
                <div className="relative group">
                  <input id="password" data-testid="login-password-input" name="password" type="password" autoComplete="current-password" required value={password} onChange={(e) => setPassword(e.target.value)} className="block w-full px-6 py-4.5 bg-slate-50 border border-slate-100 text-slate-700 rounded-[1.5rem] focus:outline-none focus:ring-4 focus:ring-primary-500/10 focus:border-primary-300 focus:bg-white transition-all font-bold placeholder:text-slate-300 text-sm tracking-[0.3em] shadow-inner" placeholder="••••••••" />
                </div>
              </div>
            </div>

            <button
              type="submit"
              disabled={isSubmitting}
              data-testid="login-submit-btn"
              className="w-full flex justify-center py-5 px-4 border border-transparent text-xs font-bold rounded-2xl text-white bg-slate-900 hover:bg-primary-700 transition-all disabled:opacity-50 uppercase tracking-[0.2em] shadow-2xl shadow-slate-400 active:scale-95"
            >
              {isSubmitting ? t("auth.processing").toUpperCase() : t("auth.login_btn").toUpperCase()}
            </button>

            <div className="text-center pt-2">
              <span className="text-slate-500 text-[10px] font-bold uppercase tracking-widest opacity-60">{t("auth.no_account")} </span>
              <Link to="/register" className="font-bold text-primary-600 hover:text-primary-700 text-[10px] uppercase tracking-widest transition-colors ml-1">{t("auth.register_now").toUpperCase()}</Link>
            </div>
          </form>
        </div>
      </div>
    </div>
  );
}
