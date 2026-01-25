import { useState } from "react";
import { useTranslation } from "react-i18next";
import { useNavigate, Navigate, Link } from "react-router-dom";
import {
  register,
  setAccessToken,
  setRefreshToken,
  setCurrentUserId,
  getCurrentUserId,
  AuthResponse,
} from "@/lib/api";

export default function RegisterPage() {
  const { t } = useTranslation();
  const userId = getCurrentUserId();
  const navigate = useNavigate();
  const [error, setError] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [formData, setFormData] = useState({ email: "", full_name: "", password: "", confirm_password: "" });

  if (userId) return <Navigate to="/" replace />;

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault(); setError(null);
    if (formData.password !== formData.confirm_password) { setError(t("auth.password_mismatch")); return; }
    if (formData.password.length < 6) { setError(t("auth.password_too_short")); return; }
    setIsLoading(true);
    try {
      const response: AuthResponse = await register({ email: formData.email, password: formData.password, full_name: formData.full_name });
      setCurrentUserId(response.user.id); setAccessToken(response.access_token); setRefreshToken(response.refresh_token);
      window.location.href = "/onboarding";
    } catch (err: any) {
      setError(err.message || t("auth.registration_failed"));
    } finally { setIsLoading(false); }
  };

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => { setFormData((prev) => ({ ...prev, [e.target.name]: e.target.value })); };

  return (
    <div className="min-h-screen flex items-center justify-center bg-[#f8fafc] p-6 relative overflow-hidden animate-in fade-in duration-1000">
      <div className="absolute top-[-10%] right-[-10%] w-[50%] h-[50%] bg-primary-100/30 rounded-full blur-[140px] animate-pulse duration-[12s]" />
      <div className="absolute bottom-[-10%] left-[-10%] w-[50%] h-[50%] bg-indigo-100/30 rounded-full blur-[140px] animate-pulse duration-[10s]" />
      
      <div className="max-w-md w-full space-y-10 p-12 bg-white rounded-[3.5rem] shadow-premium-2xl relative z-10 animate-in zoom-in-95 duration-1000 border border-white/60">
        <div className="text-center">
          <div className="bg-slate-900 w-20 h-20 rounded-[2rem] flex items-center justify-center mx-auto mb-8 shadow-2xl shadow-slate-400 transform hover:rotate-6 hover:scale-110 transition-all duration-700 group/logo">
             <span className="text-white text-4xl font-bold italic tracking-tighter group-hover:text-primary-400 transition-colors">L</span>
          </div>
          <h2 className="text-4xl font-bold text-slate-700 tracking-tighter uppercase leading-none">{t("auth.register_title")}</h2>
          <p className="mt-4 text-slate-500 font-bold uppercase tracking-[0.3em] text-[9px] opacity-60 leading-relaxed">
            {t("auth.slogan").toUpperCase()}
          </p>
        </div>

        {error && (
          <div role="alert" className="bg-rose-50 border-2 border-rose-100 text-rose-600 p-5 rounded-[2rem] text-[11px] font-bold uppercase tracking-widest text-center animate-in slide-in-from-top-4 shadow-inner">
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
              <div key={field.id} className="space-y-2 px-1">
                <label htmlFor={field.id} className="text-[10px] font-bold text-slate-500 uppercase tracking-widest ml-1 opacity-70">{t(`auth.${field.label}`)}</label>
                <input id={field.id} name={field.id} type={field.type} autoComplete={field.auto} required className="block w-full px-6 py-4 bg-slate-50 border border-slate-100 text-slate-700 rounded-[1.5rem] focus:outline-none focus:ring-4 focus:ring-primary-500/10 focus:border-primary-300 focus:bg-white transition-all font-bold placeholder:text-slate-300 text-sm uppercase tracking-tight shadow-inner" placeholder={field.placeholder} value={(formData as any)[field.id]} onChange={handleChange} />
              </div>
            ))}
          </div>

          <div className="pt-2">
            <button type="submit" disabled={isLoading} className="w-full flex justify-center py-5 px-4 border border-transparent text-xs font-bold rounded-2xl text-white bg-slate-900 hover:bg-primary-700 transition-all disabled:opacity-50 uppercase tracking-[0.2em] shadow-2xl shadow-slate-400 active:scale-95">
              {isLoading ? t("auth.creating_account").toUpperCase() : t("auth.register_btn").toUpperCase()}
            </button>
          </div>

          <div className="text-center pt-2">
            <span className="text-slate-500 text-[10px] font-bold uppercase tracking-widest opacity-60">{t("auth.already_have_account")} </span>
            <Link to="/login" className="font-bold text-primary-600 hover:text-primary-700 text-[10px] uppercase tracking-widest transition-colors ml-1">{t("auth.login").toUpperCase()}</Link>
          </div>
        </form>
      </div>
    </div>
  );
}
