import { useNavigate, Link, useLocation } from "@tanstack/react-router";
import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { AuthLayout } from "@/components/ui/auth-layout";
import { LABEL_BASE } from "@/components/ui/form-field";
import { Typography } from "@/components/ui/typography";
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
    setIsSubmitting(true);
    setError(null);
    try {
      await apiLogin(loginEmail, loginPass);
      void navigate({ to: from, replace: true });
    } catch (error_: unknown) {
      setError(error_ instanceof Error ? error_.message : t("auth.login_error"));
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleLogin = async (e: React.SyntheticEvent) => {
    e.preventDefault();
    await performLogin(email, password);
  };

  return (
    <AuthLayout>
      {/* Logo + Title */}
      <div className="mb-10">
        <div className="flex items-center gap-4 mb-6">
          <div className="bg-slate-900 dark:bg-slate-800 w-14 h-14 rounded-2xl flex items-center justify-center shadow-xl shadow-slate-300/50 dark:shadow-black/40 transform hover:rotate-6 hover:scale-110 transition-all duration-500 group/logo flex-shrink-0">
            <span className="text-white text-2xl font-bold italic tracking-tighter group-hover/logo:text-primary-400 transition-colors pl-0.5">
              L
            </span>
          </div>
          <div>
            <Typography
              className="text-3xl font-bold text-slate-800 dark:text-slate-100 tracking-tight leading-none"
              variant="h1"
            >
              LeeAttend
            </Typography>
            <p className="text-slate-400 dark:text-slate-500 text-[10px] font-bold uppercase tracking-[0.3em] mt-1">
              {t("auth.slogan")}
            </p>
          </div>
        </div>

        <div>
          <h2 className="text-2xl font-bold text-slate-700 dark:text-slate-200 tracking-tight">
            {t("auth.login.title")}
          </h2>
          <p className="text-slate-400 dark:text-slate-500 text-sm mt-1">
            {t("auth.no_account")}{" "}
            <Link
              className="font-semibold text-primary-600 dark:text-primary-400 hover:text-primary-700 dark:hover:text-primary-300 transition-colors"
              to={ROUTES.REGISTER}
            >
              {t("auth.register_now")}
            </Link>
          </p>
        </div>
      </div>

      {/* Error alert */}
      {error ? (
        <div
          className="mb-6 bg-rose-50 dark:bg-rose-950/50 border border-rose-200 dark:border-rose-800 text-rose-700 dark:text-rose-300 p-4 rounded-2xl text-sm font-medium animate-in slide-in-from-top-3 duration-300"
          role="alert"
          aria-live="polite"
        >
          {error}
        </div>
      ) : null}

      {/* Login form */}
      <form className="space-y-5" onSubmit={handleLogin} noValidate>
        <div className="space-y-2">
          <label className={LABEL_BASE} htmlFor="email-address">
            {t("auth.email")}
          </label>
          <input
            autoComplete="email"
            className="block w-full px-5 py-3.5 bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 text-slate-700 dark:text-slate-200 rounded-2xl focus:outline-none focus:ring-2 focus:ring-primary-500/30 focus:border-primary-400 focus:bg-white dark:focus:bg-slate-900 transition-all text-sm font-medium placeholder:text-slate-300 dark:placeholder:text-slate-600 shadow-sm"
            data-testid="login-email-input"
            id="email-address"
            name="email"
            onChange={(e) => { setEmail(e.target.value); }}
            placeholder="email@company.com"
            required
            type="email"
            value={email}
          />
        </div>

        <div className="space-y-2">
          <label className={LABEL_BASE} htmlFor="password">
            {t("auth.password")}
          </label>
          <input
            autoComplete="current-password"
            className="block w-full px-5 py-3.5 bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 text-slate-700 dark:text-slate-200 rounded-2xl focus:outline-none focus:ring-2 focus:ring-primary-500/30 focus:border-primary-400 focus:bg-white dark:focus:bg-slate-900 transition-all text-sm font-medium placeholder:text-slate-300 dark:placeholder:text-slate-600 tracking-[0.3em] shadow-sm"
            data-testid="login-password-input"
            id="password"
            name="password"
            onChange={(e) => { setPassword(e.target.value); }}
            placeholder="••••••••"
            required
            type="password"
            value={password}
          />
        </div>

        <div className="flex justify-end">
          <Link
            className="text-xs font-semibold text-primary-600 dark:text-primary-400 hover:text-primary-700 dark:hover:text-primary-300 transition-colors"
            data-testid="forgot-password-link"
            to={ROUTES.FORGOT_PASSWORD}
          >
            {t("auth.forgot_password")}
          </Link>
        </div>

        <div className="pt-2">
          <Button
            className="w-full"
            data-testid="login-submit-btn"
            disabled={isSubmitting}
            loading={isSubmitting}
            size="lg"
            type="submit"
            variant="secondary"
          >
            {isSubmitting ? t("auth.processing") : t("auth.login_btn")}
          </Button>
        </div>
      </form>
    </AuthLayout>
  );
}
