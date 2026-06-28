import { useNavigate, Link } from "@tanstack/react-router";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import { AuthLayout } from "@/components/ui/auth-layout";
import Button from "@/components/ui/button";
import { LABEL_BASE } from "@/components/ui/form-field";
import { Typography } from "@/components/ui/typography";
import {
  register,
  setAccessToken,
  setRefreshToken,
  setCurrentUserId,
  getCurrentUserId,
  AuthResponse,
} from "@/lib/api";
import { ROUTES } from "@/lib/routes";

interface RegisterFormData {
  confirm_password: string;
  email: string;
  full_name: string;
  password: string;
}

const INITIAL_FORM: RegisterFormData = {
  email: "",
  full_name: "",
  password: "",
  confirm_password: "",
};

export default function RegisterPage() {
  const { t } = useTranslation();
  const userId = getCurrentUserId();
  const navigate = useNavigate();
  const [error, setError] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [formData, setFormData] = useState<RegisterFormData>(INITIAL_FORM);

  if (userId) {
    void navigate({ to: ROUTES.HOME, replace: true });
    return null;
  }

  const handleSubmit = async (e: React.SyntheticEvent) => {
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
      window.location.href = ROUTES.ONBOARDING;
    } catch (error_: unknown) {
      const message = error_ instanceof Error ? error_.message : String(error_);
      setError(message || t("auth.registration_failed"));
    } finally {
      setIsLoading(false);
    }
  };

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setFormData((previous) => ({ ...previous, [e.target.name]: e.target.value }));
  };

  const fields: {
    id: keyof RegisterFormData;
    labelKey: string;
    type: string;
    placeholder: string;
    autoComplete?: string;
  }[] = [
    {
      id: "full_name",
      labelKey: "auth.display_name",
      type: "text",
      placeholder: "Nguyen Van A",
      autoComplete: "name",
    },
    {
      id: "email",
      labelKey: "auth.email",
      type: "email",
      placeholder: "email@company.com",
      autoComplete: "email",
    },
    {
      id: "password",
      labelKey: "auth.password",
      type: "password",
      placeholder: "••••••••",
      autoComplete: "new-password",
    },
    {
      id: "confirm_password",
      labelKey: "auth.confirm_password",
      type: "password",
      placeholder: "••••••••",
      autoComplete: "new-password",
    },
  ];

  return (
    <AuthLayout>
      {/* Logo + Title */}
      <div className="mb-8">
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
            {t("auth.register_title")}
          </h2>
          <p className="text-slate-400 dark:text-slate-500 text-sm mt-1">
            {t("auth.already_have_account")}{" "}
            <Link
              className="font-semibold text-primary-600 dark:text-primary-400 hover:text-primary-700 dark:hover:text-primary-300 transition-colors"
              to={ROUTES.LOGIN}
            >
              {t("auth.login.title")}
            </Link>
          </p>
        </div>
      </div>

      {/* Error alert */}
      {error ? (
        <div
          aria-live="polite"
          className="mb-5 bg-rose-50 dark:bg-rose-950/50 border border-rose-200 dark:border-rose-800 text-rose-700 dark:text-rose-300 p-4 rounded-2xl text-sm font-medium animate-in slide-in-from-top-3 duration-300"
          role="alert"
        >
          {error}
        </div>
      ) : null}

      {/* Register form */}
      <form className="space-y-4" noValidate onSubmit={handleSubmit}>
        {fields.map((field) => (
          <div className="space-y-2" key={field.id}>
            <label className={LABEL_BASE} htmlFor={field.id}>
              {t(field.labelKey)}
            </label>
            <input
              autoComplete={field.autoComplete}
              className="block w-full px-5 py-3.5 bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 text-slate-700 dark:text-slate-200 rounded-2xl focus:outline-none focus:ring-2 focus:ring-primary-500/30 focus:border-primary-400 focus:bg-white dark:focus:bg-slate-900 transition-all text-sm font-medium placeholder:text-slate-300 dark:placeholder:text-slate-600 shadow-sm"
              id={field.id}
              name={field.id}
              onChange={handleChange}
              placeholder={field.placeholder}
              required
              type={field.type}
              value={formData[field.id]}
            />
          </div>
        ))}

        <div className="pt-2">
          <Button
            className="w-full"
            disabled={isLoading}
            loading={isLoading}
            size="lg"
            type="submit"
            variant="secondary"
          >
            {isLoading ? t("auth.creating_account") : t("auth.register_btn")}
          </Button>
        </div>
      </form>
    </AuthLayout>
  );
}
