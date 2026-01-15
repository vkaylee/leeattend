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

  // Redirect if already logged in
  if (userId) {
    return <Navigate to="/" replace />;
  }

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

      // Auto login
      setCurrentUserId(response.user.id);
      setAccessToken(response.access_token);
      setRefreshToken(response.refresh_token);

      // Redirect to onboarding or home
      window.location.href = "/onboarding";
    } catch (err: any) {
      console.error("Registration failed", err);
      setError(err.message || t("auth.registration_failed"));
    } finally {
      setIsLoading(false);
    }
  };

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setFormData((prev) => ({ ...prev, [e.target.name]: e.target.value }));
  };

  return (
    <div className="min-h-screen flex items-center justify-center bg-slate-50">
      <div className="max-w-md w-full space-y-8 p-8 bg-white rounded-xl shadow-lg">
        <div className="text-center">
          <h2 className="mt-6 text-3xl font-bold text-gray-900">
            {t("auth.register_title")}
          </h2>
          <p className="mt-2 text-sm text-gray-600">
            {t("auth.slogan")}
          </p>
        </div>

        {error && (
          <div className="bg-red-50 text-red-600 p-3 rounded text-sm text-center">
            {error}
          </div>
        )}

        <form className="mt-8 space-y-6" onSubmit={handleSubmit}>
          <div className="space-y-4">
            <div>
              <label
                htmlFor="full_name"
                className="block text-sm font-medium text-gray-700"
              >
                {t("auth.display_name")}
              </label>
              <input
                id="full_name"
                name="full_name"
                type="text"
                required
                className="mt-1 appearance-none block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm placeholder-gray-400 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
                value={formData.full_name}
                onChange={handleChange}
              />
            </div>

            <div>
              <label
                htmlFor="email"
                className="block text-sm font-medium text-gray-700"
              >
                {t("auth.email")}
              </label>
              <input
                id="email"
                name="email"
                type="email"
                autoComplete="email"
                required
                className="mt-1 appearance-none block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm placeholder-gray-400 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
                value={formData.email}
                onChange={handleChange}
              />
            </div>

            <div>
              <label
                htmlFor="password"
                className="block text-sm font-medium text-gray-700"
              >
                {t("auth.password")}
              </label>
              <input
                id="password"
                name="password"
                type="password"
                autoComplete="new-password"
                required
                className="mt-1 appearance-none block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm placeholder-gray-400 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
                value={formData.password}
                onChange={handleChange}
              />
            </div>

            <div>
              <label
                htmlFor="confirm_password"
                className="block text-sm font-medium text-gray-700"
              >
                {t("auth.confirm_password")}
              </label>
              <input
                id="confirm_password"
                name="confirm_password"
                type="password"
                autoComplete="new-password"
                required
                className="mt-1 appearance-none block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm placeholder-gray-400 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
                value={formData.confirm_password}
                onChange={handleChange}
              />
            </div>
          </div>

          <div>
            <button
              type="submit"
              disabled={isLoading}
              className="group relative w-full flex justify-center py-3 px-4 border border-transparent text-sm font-medium rounded-md text-white bg-primary-600 hover:bg-primary-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 transition-colors disabled:opacity-50"
            >
              {isLoading ? t("auth.creating_account") : t("auth.register_btn")}
            </button>
          </div>

          <div className="text-center text-sm">
            <span className="text-gray-500">{t("auth.already_have_account")} </span>
            <Link
              to="/login"
              className="font-medium text-primary-600 hover:text-indigo-500"
            >
              {t("auth.login")}
            </Link>
          </div>
        </form>
      </div>
    </div>
  );
}
