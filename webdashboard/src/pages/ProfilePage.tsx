import { useState, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { useToast } from "../components/Toast";
import {
  User as UserIcon,
  Mail,
  UserCircle,
  Save,
  Calendar,
  Lock,
  ShieldCheck,
  Palette,
  Undo2,
} from "lucide-react";
import { useProfile } from "@/hooks/use-profile";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { useTheme } from "@/hooks/use-theme";
import { useTranslation } from "react-i18next";
import { Globe } from "lucide-react";
import { Combobox } from "@/components/ui/Combobox";

export default function ProfilePage() {
  const { t } = useTranslation();
  const { user, isLoading, updateProfile, isUpdating } = useProfile();
  const { formatDate } = useDateTimeFormatter();
  const { primaryColor, isOverridden, setUserTheme } = useTheme();

  const [formData, setFormData] = useState({
    full_name: "",
    avatar_url: "",
    bio: "",
    preferred_language: "",
  });

  useEffect(() => {
    if (user) {
      setFormData({
        full_name: user.full_name || "",
        avatar_url: user.avatar_url || "",
        bio: user.bio || "",
        preferred_language: user.preferred_language || "",
      });
    }
  }, [user]);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    try {
      await updateProfile(formData);
      toast.success(t("common.success"));
    } catch (e) {
      toast.error(t("common.error"));
    }
  }

  if (isLoading)
    return (
      <div className="p-8 text-center text-slate-500 font-bold">
        {t("common.loading")}
      </div>
    );

  return (
    <div className="p-8 animate-in fade-in duration-500">
      <div className="max-w-2xl mx-auto">
        <div className="mb-8">
          <h1 className="text-3xl font-bold text-slate-900 tracking-tight">
            {t("common.profile")}
          </h1>
          <p className="text-slate-500 mt-1">
            {t("profile.subtitle")}
          </p>
        </div>

        <div className="bg-white rounded-3xl border border-slate-100 shadow-sm overflow-hidden">
          <div className="h-32 bg-gradient-to-r from-blue-600 to-indigo-600"></div>
          <div className="px-8 pb-8">
            <div className="relative -mt-12 mb-6">
              {formData.avatar_url ? (
                <img
                  src={formData.avatar_url}
                  alt="Avatar"
                  className="w-24 h-24 rounded-2xl border-4 border-white shadow-lg object-cover"
                />
              ) : (
                <div className="w-24 h-24 rounded-2xl border-4 border-white shadow-lg bg-slate-100 flex items-center justify-center text-slate-400">
                  <UserCircle className="w-12 h-12" />
                </div>
              )}
            </div>

            <form onSubmit={handleSubmit} className="space-y-6">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                <div className="space-y-2">
                  <label className="text-xs font-bold text-slate-400 uppercase tracking-widest ml-1">
                    {t("common.email")}
                  </label>
                  <div className="flex items-center gap-3 px-4 py-3 bg-slate-50 border border-slate-100 rounded-xl text-slate-500">
                    <Mail className="w-4 h-4" />
                    <span className="text-sm font-medium">
                      {user?.email}
                    </span>
                  </div>
                </div>

                <div className="space-y-2">
                  <label className="text-xs font-bold text-slate-400 uppercase tracking-widest ml-1">
                    {t("profile.join_date")}
                  </label>
                  <div className="flex items-center gap-3 px-4 py-3 bg-slate-50 border border-slate-100 rounded-xl text-slate-500">
                    <Calendar className="w-4 h-4" />
                    <span className="text-sm font-medium">
                      {formatDate(user?.created_at || "")}
                    </span>
                  </div>
                </div>

                <div className="space-y-2">
                  <label className="text-xs font-bold text-slate-400 uppercase tracking-widest ml-1">
                    {t("employees.table.name")}
                  </label>
                  <div className="relative">
                    <input
                      required
                      className="w-full bg-white border border-slate-200 rounded-xl px-4 py-3 focus:ring-2 focus:ring-primary-500 outline-none transition-all pr-10"
                      value={formData.full_name}
                      onChange={(e) =>
                        setFormData({ ...formData, full_name: e.target.value })
                      }
                    />
                    <UserIcon className="absolute right-3 top-3.5 w-4 h-4 text-slate-300" />
                  </div>
                </div>

                <div className="space-y-2">
                  <label className="text-xs font-bold text-slate-400 uppercase tracking-widest ml-1">
                    {t("profile.preferred_language")}
                  </label>
                  <Combobox
                    options={[
                      { value: "", label: t("profile.follow_workspace"), icon: <Globe className="w-4 h-4" /> },
                      { value: "vi", label: t("common.languages.vi"), icon: <Globe className="w-4 h-4" /> },
                      { value: "en", label: t("common.languages.en"), icon: <Globe className="w-4 h-4" /> },
                      { value: "zh", label: t("common.languages.zh"), icon: <Globe className="w-4 h-4" /> },
                    ]}
                    value={formData.preferred_language}
                    onChange={(val) => setFormData({ ...formData, preferred_language: val })}
                    className="bg-white border-slate-200"
                  />
                </div>
              </div>

              <div className="space-y-2">
                <label className="text-xs font-bold text-slate-400 uppercase tracking-widest ml-1">
                  {t("profile.bio")}
                </label>
                <textarea
                  className="w-full bg-white border border-slate-200 rounded-xl px-4 py-3 focus:ring-2 focus:ring-primary-500 outline-none transition-all"
                  rows={3}
                  value={formData.bio}
                  onChange={(e) =>
                    setFormData({ ...formData, bio: e.target.value })
                  }
                />
              </div>

              <div className="pt-4 border-t border-slate-50 flex justify-end">
                <button
                  type="submit"
                  disabled={isUpdating}
                  className="bg-primary-600 text-white px-8 py-3 rounded-xl font-bold hover:bg-primary-700 transition-all shadow-lg shadow-primary-100 flex items-center gap-2 disabled:opacity-50"
                >
                  <Save className="w-5 h-5" />
                  {isUpdating ? t("common.loading") : t("common.save")}
                </button>
              </div>
            </form>
          </div>
        </div>

        {/* Personalization Section */}
        <div className="mt-8 bg-white rounded-3xl border border-slate-100 shadow-sm overflow-hidden p-8">
          <div className="flex items-center justify-between mb-6">
            <div className="flex items-center gap-3">
              <div className="w-10 h-10 rounded-xl bg-pink-50 flex items-center justify-center text-pink-600">
                <Palette className="w-6 h-6" />
              </div>
              <div>
                <h2 className="text-xl font-bold text-slate-900">{t("profile.personalization.title")}</h2>
                <p className="text-sm text-slate-500">
                  {t("profile.personalization.subtitle")}
                </p>
              </div>
            </div>

            {isOverridden && (
              <button 
                onClick={() => setUserTheme(null)}
                className="flex items-center gap-2 text-[10px] font-black uppercase tracking-widest text-slate-400 hover:text-primary-600 transition-colors"
              >
                <Undo2 className="w-3 h-3" />
                {t("profile.personalization.use_workspace_default")}
              </button>
            )}
          </div>

          <div className="space-y-4">
            <div className="flex flex-wrap gap-3 items-center">
              {[
                { name: t("common.colors.violet"), color: '#7c3aed' },
                { name: t("common.colors.indigo"), color: '#4f46e5' },
                { name: t("common.colors.blue"), color: '#2563eb' },
                { name: t("common.colors.emerald"), color: '#059669' },
                { name: t("common.colors.amber"), color: '#d97706' },
                { name: t("common.colors.rose"), color: '#e11d48' },
                { name: t("common.colors.slate"), color: '#475569' },
                { name: t("common.colors.black"), color: '#0f172a' },
              ].map((preset) => (
                <button
                  key={preset.color}
                  onClick={() => setUserTheme(preset.color)}
                  className={`w-10 h-10 rounded-xl transition-all border-2 flex items-center justify-center ${
                    primaryColor === preset.color ? 'border-slate-900 scale-110 shadow-lg' : 'border-transparent hover:scale-105'
                  }`}
                  style={{ backgroundColor: preset.color }}
                  title={preset.name}
                >
                  {primaryColor === preset.color && <div className="w-2 h-2 bg-white rounded-full" />}
                </button>
              ))}
              
              <div className="h-8 w-[1px] bg-slate-100 mx-1" />
              
              <div className="flex items-center gap-2 bg-slate-50 p-1 rounded-xl border border-slate-100">
                <input 
                  type="color" 
                  value={primaryColor} 
                  onChange={(e) => setUserTheme(e.target.value)}
                  className="w-8 h-8 rounded-lg cursor-pointer bg-transparent border-none"
                />
              </div>
            </div>
            <p className="text-[10px] text-slate-400 italic">
              {t("profile.personalization.note")}
            </p>
          </div>
        </div>

        {/* Security Section */}
        <div className="mt-8 bg-white rounded-3xl border border-slate-100 shadow-sm overflow-hidden">
          <div className="p-8">
            <div className="flex items-center gap-3 mb-6">
              <div className="w-10 h-10 rounded-xl bg-orange-50 flex items-center justify-center text-orange-600">
                <ShieldCheck className="w-6 h-6" />
              </div>
              <div>
                <h2 className="text-xl font-bold text-slate-900">{t("profile.security.title")}</h2>
                <p className="text-sm text-slate-500">
                  {t("profile.security.subtitle")}
                </p>
              </div>
            </div>

            <ChangePasswordForm />
          </div>
        </div>
      </div>
    </div>
  );
}

function ChangePasswordForm() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const toast = useToast();
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [formData, setFormData] = useState({
    old_password: "",
    new_password: "",
    confirm_password: "",
  });

  async function handlePasswordSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (formData.new_password !== formData.confirm_password) {
      toast.warning(t("profile.security.messages.confirm_mismatch"));
      return;
    }

    setIsSubmitting(true);
    try {
      await changePassword({
        old_password: formData.old_password,
        new_password: formData.new_password,
      });
      toast.success(t("profile.security.messages.password_changed"));
      setTimeout(() => {
        clearAuthSession();
        navigate("/login");
      }, 2000);
    } catch (e: any) {
      toast.error(e.message || t("profile.security.messages.change_error"));
    } finally {
      setIsSubmitting(false);
    }
  }

  return (
    <form onSubmit={handlePasswordSubmit} className="space-y-4 max-w-md">
      <div className="space-y-2">
        <label className="text-xs font-bold text-slate-400 uppercase tracking-widest ml-1">
          {t("profile.security.current_password")}
        </label>
        <div className="relative">
          <input
            type="password"
            required
            className="w-full bg-white border border-slate-200 rounded-xl px-4 py-3 focus:ring-2 focus:ring-orange-500 outline-none transition-all pr-10"
            value={formData.old_password}
            onChange={(e) =>
              setFormData({ ...formData, old_password: e.target.value })
            }
          />
          <Lock className="absolute right-3 top-3.5 w-4 h-4 text-slate-300" />
        </div>
      </div>

      <div className="space-y-2">
        <label className="text-xs font-bold text-slate-400 uppercase tracking-widest ml-1">
          {t("profile.security.new_password")}
        </label>
        <div className="relative">
          <input
            type="password"
            required
            className="w-full bg-white border border-slate-200 rounded-xl px-4 py-3 focus:ring-2 focus:ring-orange-500 outline-none transition-all pr-10"
            value={formData.new_password}
            onChange={(e) =>
              setFormData({ ...formData, new_password: e.target.value })
            }
          />
          <Lock className="absolute right-3 top-3.5 w-4 h-4 text-slate-300" />
        </div>
      </div>

      <div className="space-y-2">
        <label className="text-xs font-bold text-slate-400 uppercase tracking-widest ml-1">
          {t("profile.security.confirm_password")}
        </label>
        <div className="relative">
          <input
            type="password"
            required
            className="w-full bg-white border border-slate-200 rounded-xl px-4 py-3 focus:ring-2 focus:ring-orange-500 outline-none transition-all pr-10"
            value={formData.confirm_password}
            onChange={(e) =>
              setFormData({ ...formData, confirm_password: e.target.value })
            }
          />
          <Lock className="absolute right-3 top-3.5 w-4 h-4 text-slate-300" />
        </div>
      </div>

      <div className="pt-2">
        <button
          type="submit"
          disabled={isSubmitting}
          className="bg-slate-900 text-white px-6 py-3 rounded-xl font-bold hover:bg-slate-800 transition-all flex items-center gap-2 disabled:opacity-50"
        >
          {isSubmitting ? t("common.processing") : t("profile.security.submit_btn")}
        </button>
      </div>
    </form>
  );
}