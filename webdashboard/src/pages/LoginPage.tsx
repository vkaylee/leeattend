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
    setIsSubmitting(true);
    setError(null);
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
      {/* Decorative Background Elements */}
      <div className="absolute top-[-10%] left-[-10%] w-[40%] h-[40%] bg-primary-100/40 rounded-full blur-[120px] animate-pulse duration-[10s]" />
      <div className="absolute bottom-[-10%] right-[-10%] w-[40%] h-[40%] bg-indigo-100/40 rounded-full blur-[120px] animate-pulse duration-[8s]" />
      
      <div className="max-w-md w-full space-y-10 p-12 bg-white rounded-[3rem] shadow-2xl shadow-slate-200/60 relative z-10 animate-in zoom-in-95 duration-700 border border-slate-100">
        <div className="text-center">
          <div className="bg-primary-600 w-20 h-20 rounded-[2rem] flex items-center justify-center mx-auto mb-8 shadow-2xl shadow-primary-200 transform hover:rotate-12 transition-transform duration-500">
             <span className="text-white text-4xl font-black italic">L</span>
          </div>
          <h2 className="text-4xl font-black text-slate-900 tracking-tighter uppercase">LeeAttend</h2>
          <p className="mt-3 text-slate-400 font-bold uppercase tracking-widest text-[10px] opacity-80">
            {t("auth.slogan")}
          </p>
        </div>

        {error && (
          <div role="alert" className="bg-rose-50 border border-rose-100 text-rose-600 p-4 rounded-2xl text-[11px] font-black uppercase tracking-widest text-center animate-in slide-in-from-top-2">
            {error}
          </div>
        )}

        <div className="space-y-8">
          <button
            onClick={handleFastLogin}
            {...debugProps({ devEmail: "admin@leeattend.com", devPass: "admin123" })}
            data-testid="fast-login-btn"
            className="group relative w-full flex justify-center py-4 px-4 border-2 border-primary-100 text-xs font-black rounded-2xl text-primary-600 bg-primary-50/30 hover:bg-primary-50 hover:scale-[1.02] active:scale-95 transition-all uppercase tracking-widest"
          >
            {t("auth.fast_login")}
          </button>

          <div className="relative">
            <div className="absolute inset-0 flex items-center"><div className="w-full border-t border-slate-100" /></div>
            <div className="relative flex justify-center text-[9px]"><span className="px-4 bg-white text-slate-300 font-black uppercase tracking-[0.3em]">{t("auth.or_use_account")}</span></div>
          </div>

          <form className="space-y-6" onSubmit={handleLogin}>
            <div className="space-y-5">
              <div className="space-y-2">
                <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("auth.email")}</label>
                <div className="relative group">
                  <input id="email-address" name="email" type="email" autoComplete="email" required value={email} onChange={(e) => setEmail(e.target.value)} className="block w-full px-5 py-4 bg-slate-50 border border-slate-100 text-slate-900 rounded-2xl focus:outline-none focus:ring-4 focus:ring-primary-500/10 focus:border-primary-300 focus:bg-white transition-all font-bold placeholder:text-slate-300 text-sm" placeholder="admin@leeattend.com" />
                </div>
              </div>
              <div className="space-y-2">
                <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("auth.password")}</label>
                <div className="relative group">
                  <input id="password" name="password" type="password" autoComplete="current-password" required value={password} onChange={(e) => setPassword(e.target.value)} className="block w-full px-5 py-4 bg-slate-50 border border-slate-100 text-slate-900 rounded-2xl focus:outline-none focus:ring-4 focus:ring-primary-500/10 focus:border-primary-300 focus:bg-white transition-all font-bold placeholder:text-slate-300 text-sm" placeholder="••••••••" />
                </div>
              </div>
            </div>

            <button
              type="submit"
              disabled={isSubmitting}
              data-testid="login-submit-btn"
              className="w-full flex justify-center py-5 px-4 border border-transparent text-xs font-black rounded-2xl text-white bg-slate-900 hover:bg-black transition-all disabled:opacity-50 uppercase tracking-widest shadow-xl shadow-slate-200 active:scale-95"
            >
              {isSubmitting ? t("auth.processing").toUpperCase() : t("auth.login_btn")}
            </button>

            <div className="text-center pt-2">
              <span className="text-slate-400 text-xs font-bold uppercase tracking-tight">{t("auth.no_account")} </span>
              <Link to="/register" className="font-black text-primary-600 hover:text-primary-700 hover:underline text-xs uppercase tracking-tight transition-colors">{t("auth.register_now")}</Link>
            </div>
          </form>
        </div>
      </div>
    </div>
  );
}
