import { Link, useSearch } from "@tanstack/react-router";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import { AuthLayout } from "@/components/ui/auth-layout";
import Button from "@/components/ui/button";
import { LABEL_BASE } from "@/components/ui/form-field";
import { Typography } from "@/components/ui/typography";
import { resetPassword } from "@/lib/api";
import { ROUTES } from "@/lib/routes";

enum Step {
  Form = "form",
  Success = "success",
}

export default function ResetPasswordPage() {
  const { t } = useTranslation();
  const { token } = useSearch({ from: "/reset-password" });

  const [newPassword, setNewPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [step, setStep] = useState<Step>(Step.Form);

  const isTokenMissing = !token;

  const handleSubmit = async (e: React.SyntheticEvent) => {
    e.preventDefault();
    setError(null);

    if (!newPassword.trim()) {
      setError(t("common.required_fields"));
      return;
    }

    if (newPassword.length < 6) {
      setError(t("auth.password_too_short"));
      return;
    }

    if (!token) {
      setError(t("auth.reset_token_invalid"));
      return;
    }

    setIsSubmitting(true);
    try {
      await resetPassword(token, newPassword);
      setStep(Step.Success);
    } catch (error_: unknown) {
      setError(
        error_ instanceof Error ? error_.message : t("auth.reset_token_invalid"),
      );
    } finally {
      setIsSubmitting(false);
    }
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
            <Typography className="text-slate-400 dark:text-slate-500 text-[10px] font-bold uppercase tracking-[0.3em] mt-1" variant="p">
              {t("auth.slogan")}
            </Typography>
          </div>
        </div>

        <div>
          <Typography className="text-2xl font-bold text-slate-700 dark:text-slate-200 tracking-tight" variant="h2">
            {t("auth.reset_password_title")}
          </Typography>
          <Typography className="text-slate-400 dark:text-slate-500 text-sm mt-1" variant="p">
            {t("auth.back_to_login")}
            {" "}
            <Link
              className="font-semibold text-primary-600 dark:text-primary-400 hover:text-primary-700 dark:hover:text-primary-300 transition-colors"
              to={ROUTES.LOGIN}
            >
              {t("auth.login.title")}
            </Link>
          </Typography>
        </div>
      </div>

      {/* Error alert */}
      {error ? (
        <div
          aria-live="polite"
          className="mb-6 bg-rose-50 dark:bg-rose-950/50 border border-rose-200 dark:border-rose-800 text-rose-700 dark:text-rose-300 p-4 rounded-2xl text-sm font-medium animate-in slide-in-from-top-3 duration-300"
          role="alert"
        >
          {error}
        </div>
      ) : null}

      {isTokenMissing ? (
        /* Missing or invalid token state */
        <div className="space-y-5">
          <div className="bg-amber-50 dark:bg-amber-950/30 border border-amber-200 dark:border-amber-800 rounded-2xl p-5 text-center animate-in fade-in zoom-in-95 duration-300">
            <div className="w-12 h-12 bg-amber-100 dark:bg-amber-900/50 rounded-full flex items-center justify-center mx-auto mb-3">
              <svg className="w-6 h-6 text-amber-600 dark:text-amber-400" fill="none" stroke="currentColor" strokeWidth={2} viewBox="0 0 24 24">
                <path d="M12 9v3.75m9-.75a9 9 0 11-18 0 9 9 0 0118 0zm-9 3.75h.008v.008H12v-.008z" strokeLinecap="round" strokeLinejoin="round" />
              </svg>
            </div>
            <Typography className="text-amber-800 dark:text-amber-200 font-semibold text-sm" variant="p">
              {t("auth.reset_token_invalid")}
            </Typography>
          </div>

          <div className="text-center">
            <Link
              className="text-sm font-semibold text-primary-600 dark:text-primary-400 hover:text-primary-700 dark:hover:text-primary-300 transition-colors"
              to={ROUTES.FORGOT_PASSWORD}
            >
              {t("auth.forgot_password")}
            </Link>
          </div>
        </div>
      ) : step === Step.Form ? (
        /* New password form */
        <form className="space-y-5" noValidate onSubmit={handleSubmit}>
          <div className="space-y-2">
            <label className={LABEL_BASE} htmlFor="new-password">
              {t("auth.new_password")}
            </label>
            <input
              autoComplete="new-password"
              className="block w-full px-5 py-3.5 bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 text-slate-700 dark:text-slate-200 rounded-2xl focus:outline-none focus:ring-2 focus:ring-primary-500/30 focus:border-primary-400 focus:bg-white dark:focus:bg-slate-900 transition-all text-sm font-medium placeholder:text-slate-300 dark:placeholder:text-slate-600 tracking-[0.3em] shadow-sm"
              data-testid="reset-password-input"
              id="new-password"
              name="new_password"
              onChange={(e) => { setNewPassword(e.target.value); }}
              placeholder="••••••••"
              required
              type="password"
              value={newPassword}
            />
          </div>

          <div className="pt-2">
            <Button
              className="w-full"
              data-testid="reset-password-submit-btn"
              disabled={isSubmitting}
              loading={isSubmitting}
              size="lg"
              type="submit"
              variant="secondary"
            >
              {isSubmitting ? t("common.processing") : t("auth.reset_password_btn")}
            </Button>
          </div>

          <div className="text-center">
            <Link
              className="text-sm font-semibold text-primary-600 dark:text-primary-400 hover:text-primary-700 dark:hover:text-primary-300 transition-colors"
              to={ROUTES.LOGIN}
            >
              &larr; {t("auth.back_to_login")}
            </Link>
          </div>
        </form>
      ) : (
        /* Success state */
        <div className="space-y-5">
          <div className="bg-emerald-50 dark:bg-emerald-950/30 border border-emerald-200 dark:border-emerald-800 rounded-2xl p-5 text-center animate-in fade-in zoom-in-95 duration-300">
            <div className="w-12 h-12 bg-emerald-100 dark:bg-emerald-900/50 rounded-full flex items-center justify-center mx-auto mb-3">
              <svg className="w-6 h-6 text-emerald-600 dark:text-emerald-400" fill="none" stroke="currentColor" strokeWidth={2.5} viewBox="0 0 24 24">
                <path d="M9 12.75L11.25 15 15 9.75M21 12a9 9 0 11-18 0 9 9 0 0118 0z" strokeLinecap="round" strokeLinejoin="round" />
              </svg>
            </div>
            <Typography className="text-emerald-800 dark:text-emerald-200 font-semibold text-sm" variant="p">
              {t("auth.reset_success")}
            </Typography>
          </div>

          <div className="text-center">
            <Link
              className="font-semibold text-primary-600 dark:text-primary-400 hover:text-primary-700 dark:hover:text-primary-300 transition-colors"
              to={ROUTES.LOGIN}
            >
              {t("auth.login.title")}
            </Link>
          </div>
        </div>
      )}
    </AuthLayout>
  );
}
