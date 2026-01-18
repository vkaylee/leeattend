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

  // Redirect if already logged in
  if (userId) {
    return <Navigate to="/" replace />;
  }

  const performLogin = async (loginEmail: string, loginPass: string) => {
    setIsSubmitting(true);
    setError(null);
    try {
      await apiLogin(loginEmail, loginPass);
      navigate("/", { replace: true });
      window.location.reload();
    } catch (err: any) {
      setError(err.message || t("auth.login_error"));
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleLogin = async (e: React.FormEvent) => {
    e.preventDefault();
    await performLogin(email, password);
  };

  const handleFastLogin = async () => {
    const devEmail = "admin@leeattend.com";
    const devPass = "admin123";
    setEmail(devEmail);
    setPassword(devPass);
    await performLogin(devEmail, devPass);
  };

  return (
    <div className="min-h-screen flex items-center justify-center bg-[#fcfcfd] p-4 relative overflow-hidden">
      {/* Decorative Background Elements */}
      <div className="absolute top-0 -left-20 w-96 h-96 bg-primary-100/50 rounded-full blur-3xl" />
      <div className="absolute bottom-0 -right-20 w-96 h-96 bg-indigo-100/50 rounded-full blur-3xl" />
      
      <div className="max-w-md w-full space-y-10 p-12 bg-white rounded-[2.5rem] shadow-2xl shadow-slate-200/50 relative z-10 animate-enter border border-slate-50">
        <div className="text-center">
          <div className="bg-primary-600 w-16 h-16 rounded-[1.5rem] flex items-center justify-center mx-auto mb-6 shadow-xl shadow-primary-200 rotate-3">
             <span className="text-white text-3xl font-black">L</span>
          </div>
          <h2 className="text-4xl font-black text-slate-900 tracking-tight">LeeAttend</h2>
          <p className="mt-3 text-slate-500 font-medium">
            {t("auth.slogan")}
          </p>
        </div>

        {error && (
          <div role="alert" className="bg-rose-50 border border-rose-100 text-rose-600 p-4 rounded-2xl text-sm font-bold text-center animate-in fade-in slide-in-from-top-2">
            {error}
          </div>
        )}

        <div className="space-y-8">
          <button
            onClick={handleFastLogin}
            {...debugProps({ devEmail: "admin@leeattend.com", devPass: "admin123" })}
            data-testid="fast-login-btn"
            className="group relative w-full flex justify-center py-4 px-4 border border-transparent text-sm font-black rounded-2xl text-white bg-primary-600 hover:bg-primary-700 hover:scale-[1.02] active:scale-95 transition-all shadow-xl shadow-primary-200 uppercase tracking-widest"
          >
            {t("auth.fast_login")}
          </button>

          <div className="relative">
            <div className="absolute inset-0 flex items-center">
              <div className="w-full border-t border-slate-100" />
            </div>
            <div className="relative flex justify-center text-[10px]">
              <span className="px-4 bg-white text-slate-400 font-black uppercase tracking-[0.2em]">
                {t("auth.or_use_account")}
              </span>
            </div>
          </div>

          <form className="space-y-6" onSubmit={handleLogin}>
            <div className="space-y-4">
              <div className="space-y-2">
                <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest px-1">{t("auth.email")}</label>
                <input
                  id="email-address"
                  name="email"
                  type="email"
                  autoComplete="email"
                  required
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  className="block w-full px-5 py-4 bg-slate-50 border border-slate-100 text-slate-900 rounded-2xl focus:outline-none focus:ring-4 focus:ring-primary-500/10 focus:border-primary-300 transition-all font-bold placeholder:text-slate-300"
                  placeholder="admin@leeattend.com"
                />
              </div>
              <div className="space-y-2">
                <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest px-1">{t("auth.password")}</label>
                <input
                  id="password"
                  name="password"
                  type="password"
                  autoComplete="current-password"
                  required
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  className="block w-full px-5 py-4 bg-slate-50 border border-slate-100 text-slate-900 rounded-2xl focus:outline-none focus:ring-4 focus:ring-primary-500/10 focus:border-primary-300 transition-all font-bold placeholder:text-slate-300"
                  placeholder="••••••••"
                />
              </div>
            </div>

            <button
              type="submit"
              disabled={isSubmitting}
              data-testid="login-submit-btn"
              className="w-full flex justify-center py-4 px-4 border border-slate-200 text-sm font-black rounded-2xl text-slate-700 bg-white hover:bg-slate-50 hover:border-primary-200 transition-all disabled:opacity-50 uppercase tracking-widest shadow-sm"
            >
              {isSubmitting ? t("auth.processing") : t("auth.login_btn")}
            </button>

            <div className="text-center">
              <span className="text-slate-400 text-sm font-medium">{t("auth.no_account")} </span>
              <Link
                to="/register"
                className="font-bold text-primary-600 hover:text-primary-700 hover:underline text-sm transition-colors"
              >
                {t("auth.register_now")}
              </Link>
            </div>
          </form>
        </div>
      </div>
    </div>
  );
}
