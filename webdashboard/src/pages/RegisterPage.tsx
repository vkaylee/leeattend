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
  const [formData, setFormData] = useState({
    email: "",
    full_name: "",
    password: "",
    confirm_password: "",
  });

  if (userId) return <Navigate to="/" replace />;

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    if (formData.password !== formData.confirm_password) {
      setError(t("auth.password_mismatch"));
      return;
    }
    if (formData.password.length < 6) {
      setError(t("auth.password_too_short"));
      return;
    }
    setIsLoading(true);
    try {
      const response: AuthResponse = await register({
        email: formData.email,
        password: formData.password,
        full_name: formData.full_name,
      });
      setCurrentUserId(response.user.id);
      setAccessToken(response.access_token);
      setRefreshToken(response.refresh_token);
      window.location.href = "/onboarding";
    } catch (err: any) {
      console.error("Registration failed", err);
      setError(err.message || t("auth.registration_failed"));
    } finally { setIsLoading(false); }
  };

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setFormData((prev) => ({ ...prev, [e.target.name]: e.target.value }));
  };

  return (
    <div className="min-h-screen flex items-center justify-center bg-[#f8fafc] p-6 relative overflow-hidden animate-in fade-in duration-1000">
      {/* Decorative Background Elements */}
      <div className="absolute top-[-10%] right-[-10%] w-[40%] h-[40%] bg-primary-100/40 rounded-full blur-[120px] animate-pulse duration-[10s]" />
      <div className="absolute bottom-[-10%] left-[-10%] w-[40%] h-[40%] bg-indigo-100/40 rounded-full blur-[120px] animate-pulse duration-[8s]" />
      
      <div className="max-w-md w-full space-y-8 p-12 bg-white rounded-[3rem] shadow-2xl shadow-slate-200/60 relative z-10 animate-in zoom-in-95 duration-700 border border-slate-100">
        <div className="text-center">
          <div className="bg-primary-600 w-16 h-16 rounded-[1.5rem] flex items-center justify-center mx-auto mb-6 shadow-2xl shadow-primary-200 transform hover:rotate-12 transition-transform duration-500">
             <span className="text-white text-3xl font-black italic">L</span>
          </div>
          <h2 className="text-3xl font-black text-slate-900 tracking-tighter uppercase">
            {t("auth.register_title")}
          </h2>
          <p className="mt-2 text-slate-400 font-bold uppercase tracking-widest text-[9px] opacity-80">
            {t("auth.slogan")}
          </p>
        </div>

        {error && (
          <div role="alert" className="bg-rose-50 border border-rose-100 text-rose-600 p-4 rounded-2xl text-[11px] font-black uppercase tracking-widest text-center animate-in slide-in-from-top-2">
            {error}
          </div>
        )}

        <form className="space-y-6" onSubmit={handleSubmit}>
          <div className="space-y-4">
            {[
              { id: "full_name", label: "display_name", type: "text", placeholder: "Nguyen Van A" },
              { id: "email", label: "email", type: "email", placeholder: "email@congty.com", auto: "email" },
              { id: "password", label: "password", type: "password", placeholder: "••••••••", auto: "new-password" },
              { id: "confirm_password", label: "confirm_password", type: "password", placeholder: "••••••••", auto: "new-password" }
            ].map((field) => (
              <div key={field.id} className="space-y-1.5">
                <label htmlFor={field.id} className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t(`auth.${field.label}`)}</label>
                <input
                  id={field.id}
                  name={field.id}
                  type={field.type}
                  autoComplete={field.auto}
                  required
                  className="block w-full px-5 py-3.5 bg-slate-50 border border-slate-100 text-slate-900 rounded-2xl focus:outline-none focus:ring-4 focus:ring-primary-500/10 focus:border-primary-300 focus:bg-white transition-all font-bold placeholder:text-slate-300 text-sm"
                  placeholder={field.placeholder}
                  value={(formData as any)[field.id]}
                  onChange={handleChange}
                />
              </div>
            ))}
          </div>

          <div className="pt-2">
            <button
              type="submit"
              disabled={isLoading}
              className="w-full flex justify-center py-4 px-4 border border-transparent text-xs font-black rounded-2xl text-white bg-slate-900 hover:bg-black transition-all disabled:opacity-50 uppercase tracking-widest shadow-xl shadow-slate-200 active:scale-95"
            >
              {isLoading ? t("auth.creating_account").toUpperCase() : t("auth.register_btn")}
            </button>
          </div>

          <div className="text-center">
            <span className="text-slate-400 text-xs font-bold uppercase tracking-tight">{t("auth.already_have_account")} </span>
            <Link
              to="/login"
              className="font-black text-primary-600 hover:text-primary-700 hover:underline text-xs uppercase tracking-tight transition-colors"
            >
              {t("auth.login")}
            </Link>
          </div>
        </form>
      </div>
    </div>
  );
}
