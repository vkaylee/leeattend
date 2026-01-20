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
import { changePassword, clearAuthSession } from "@/lib/api";

type TabType = "profile" | "personalization" | "security";

export default function ProfilePage() {
  const { t } = useTranslation();
  const { user, isLoading, updateProfile, isUpdating } = useProfile();
  const { formatDate } = useDateTimeFormatter();
  const { primaryColor, isOverridden, setUserTheme } = useTheme();
  const toast = useToast();
  const [activeTab, setActiveTab] = useState<TabType>("profile");

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

  if (isLoading) return <div className="p-8 text-center text-slate-500 font-bold italic">{t("common.loading")}</div>;

  return (
    <div className="space-y-8 animate-in fade-in duration-500">
      <div className="flex flex-col md:flex-row md:items-end justify-between gap-6 px-2">
        <div>
          <h1 className="text-3xl font-bold text-slate-900 tracking-tight">{t("common.profile")}</h1>
          <p className="text-slate-500 mt-1 font-medium">{t("profile.subtitle")}</p>
        </div>

        {/* Tab Switcher */}
        <div className="flex bg-slate-100/50 p-1.5 rounded-[22px] w-fit border border-slate-200/50">
          <button onClick={() => setActiveTab("profile")} className={`flex items-center gap-2 px-6 py-2.5 rounded-[18px] text-xs font-black uppercase tracking-widest transition-all ${activeTab === 'profile' ? "bg-white text-primary-600 shadow-sm border border-slate-100" : "text-slate-500 hover:text-slate-700"}`}>
            <UserCircle size={14} /> {t("common.profile")}
          </button>
          <button onClick={() => setActiveTab("personalization")} className={`flex items-center gap-2 px-6 py-2.5 rounded-[18px] text-xs font-black uppercase tracking-widest transition-all ${activeTab === 'personalization' ? "bg-white text-primary-600 shadow-sm border border-slate-100" : "text-slate-500 hover:text-slate-700"}`}>
            <Palette size={14} /> {t("profile.personalization.title")}
          </button>
          <button onClick={() => setActiveTab("security")} className={`flex items-center gap-2 px-6 py-2.5 rounded-[18px] text-xs font-black uppercase tracking-widest transition-all ${activeTab === 'security' ? "bg-white text-primary-600 shadow-sm border border-slate-100" : "text-slate-500 hover:text-slate-700"}`}>
            <ShieldCheck size={14} /> {t("profile.security.title")}
          </button>
        </div>
      </div>

      <div className="animate-in slide-in-from-bottom-4 duration-500">

      <div className="animate-in slide-in-from-bottom-4 duration-500">
        {activeTab === "profile" && (
          <div className="max-w-4xl bg-white rounded-[32px] border border-slate-100 shadow-sm overflow-hidden">
            <div className="h-40 bg-gradient-to-r from-primary-600 to-indigo-600 opacity-90"></div>
            <div className="px-10 pb-10">
              <div className="relative -mt-16 mb-8 flex items-end gap-6">
                <div className="w-32 h-32 rounded-[2.5rem] border-8 border-white shadow-xl bg-slate-100 flex items-center justify-center text-slate-300 overflow-hidden">
                  {formData.avatar_url ? <img src={formData.avatar_url} className="w-full h-full object-cover" alt="avatar" /> : <UserCircle size={64} />}
                </div>
                <div className="mb-4">
                  <h2 className="text-2xl font-black text-slate-900 tracking-tight">{user?.full_name || user?.email}</h2>
                  <p className="text-sm font-bold text-slate-400 uppercase tracking-widest">{t("profile.join_date")}: {formatDate(user?.created_at || "")}</p>
                </div>
              </div>

              <form onSubmit={handleSubmit} className="space-y-8">
                <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
                  <div className="space-y-2">
                    <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("common.email")}</label>
                    <div className="flex items-center gap-3 px-5 py-4 bg-slate-50 border border-slate-100 rounded-2xl text-slate-500 font-bold text-sm shadow-inner">
                      <Mail size={16} /> {user?.email}
                    </div>
                  </div>

                  <div className="space-y-2">
                    <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("employees.table.name")}</label>
                    <div className="relative group">
                      <input required className="w-full bg-slate-50 border border-slate-100 rounded-2xl px-5 py-4 focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all pr-12 font-bold text-sm" value={formData.full_name} onChange={(e) => setFormData({ ...formData, full_name: e.target.value })} />
                      <UserIcon className="absolute right-4 top-4.5 text-slate-300 group-focus-within:text-primary-500 transition-colors" size={18} />
                    </div>
                  </div>

                  <div className="space-y-2">
                    <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("profile.preferred_language")}</label>
                    <Combobox
                      options={[{ value: "", label: t("profile.follow_workspace"), icon: <Globe size={14} /> }, { value: "vi", label: t("common.languages.vi"), icon: <Globe size={14} /> }, { value: "en", label: t("common.languages.en"), icon: <Globe size={14} /> }, { value: "zh", label: t("common.languages.zh"), icon: <Globe size={14} /> }]}
                      value={formData.preferred_language}
                      onChange={(val) => setFormData({ ...formData, preferred_language: val })}
                      className="bg-slate-50 border-slate-100"
                    />
                  </div>
                </div>

                <div className="space-y-2">
                  <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("profile.bio")}</label>
                  <textarea className="w-full bg-slate-50 border border-slate-100 rounded-2xl px-5 py-4 focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all font-medium text-sm min-h-[120px] resize-none" rows={3} value={formData.bio} onChange={(e) => setFormData({ ...formData, bio: e.target.value })} placeholder={t("profile.bio_placeholder", { defaultValue: "Tell us a bit about yourself..." })} />
                </div>

                <div className="pt-6 border-t border-slate-50 flex justify-end">
                  <button type="submit" disabled={isUpdating} className="bg-primary-600 text-white px-10 py-3.5 rounded-2xl font-black text-xs uppercase tracking-widest hover:bg-primary-700 transition-all shadow-lg shadow-primary-100 flex items-center gap-2 active:scale-95 disabled:opacity-50">
                    <Save size={16} /> {isUpdating ? t("common.loading") : t("common.save")}
                  </button>
                </div>
              </form>
            </div>
          </div>
        )}

        {activeTab === "personalization" && (
          <div className="max-w-4xl bg-white rounded-[32px] border border-slate-100 shadow-sm p-10 space-y-10">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-4">
                <div className="w-14 h-14 rounded-2xl bg-pink-50 flex items-center justify-center text-pink-600 shadow-inner border border-pink-100/50"><Palette size={28} /></div>
                <div>
                  <h2 className="text-2xl font-black text-slate-900 tracking-tight">{t("profile.personalization.title")}</h2>
                  <p className="text-sm font-medium text-slate-500">{t("profile.personalization.subtitle")}</p>
                </div>
              </div>
              {isOverridden && (
                <button onClick={() => setUserTheme(null)} className="flex items-center gap-2 text-[10px] font-black uppercase tracking-widest text-slate-400 hover:text-primary-600 transition-colors bg-slate-50 px-4 py-2 rounded-xl border border-slate-100 shadow-sm"><Undo2 size={12} /> {t("profile.personalization.use_workspace_default")}</button>
              )}
            </div>

            <div className="space-y-6">
              <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("common.workspace_settings.primary_color")}</label>
              <div className="flex flex-wrap gap-4 items-center">
                {[
                  { name: t("common.colors.violet"), color: '#7c3aed' }, { name: t("common.colors.indigo"), color: '#4f46e5' }, { name: t("common.colors.blue"), color: '#2563eb' },
                  { name: t("common.colors.emerald"), color: '#059669' }, { name: t("common.colors.amber"), color: '#d97706' }, { name: t("common.colors.rose"), color: '#e11d48' },
                  { name: t("common.colors.slate"), color: '#475569' }, { name: t("common.colors.black"), color: '#0f172a' },
                ].map((preset) => (
                  <button key={preset.color} onClick={() => setUserTheme(preset.color)} className={`w-12 h-12 rounded-2xl transition-all border-2 flex items-center justify-center shadow-sm ${primaryColor === preset.color ? 'border-slate-900 scale-110 shadow-lg' : 'border-transparent hover:scale-105'}`} style={{ backgroundColor: preset.color }} title={preset.name}>
                    {primaryColor === preset.color && <div className="w-2.5 h-2.5 bg-white rounded-full shadow-sm" />}
                  </button>
                ))}
                <div className="h-10 w-[1px] bg-slate-100 mx-2" />
                <div className="flex items-center gap-3 bg-slate-50 p-2 rounded-2xl border border-slate-100 shadow-inner">
                  <input type="color" value={primaryColor} onChange={(e) => setUserTheme(e.target.value)} className="w-10 h-10 rounded-xl cursor-pointer bg-transparent border-none p-0" />
                  <input type="text" value={primaryColor} onChange={(e) => setUserTheme(e.target.value)} className="bg-transparent border-none text-xs font-mono font-black text-slate-600 w-24 outline-none uppercase" />
                </div>
              </div>
              <div className="p-6 bg-slate-50 border border-slate-100 rounded-[2rem] text-xs text-slate-500 font-medium leading-relaxed italic">{t("profile.personalization.note")}</div>
            </div>
          </div>
        )}

        {activeTab === "security" && (
          <div className="max-w-4xl bg-white rounded-[32px] border border-slate-100 shadow-sm p-10 space-y-10">
            <div className="flex items-center gap-4">
              <div className="w-14 h-14 rounded-2xl bg-orange-50 flex items-center justify-center text-orange-600 shadow-inner border border-orange-100/50"><ShieldCheck size={28} /></div>
              <div>
                <h2 className="text-2xl font-black text-slate-900 tracking-tight">{t("profile.security.title")}</h2>
                <p className="text-sm font-medium text-slate-500">{t("profile.security.subtitle")}</p>
              </div>
            </div>
            <ChangePasswordForm />
          </div>
        )}
      </div>
    </div>
  );
}

function ChangePasswordForm() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const toast = useToast();
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [formData, setFormData] = useState({ old_password: "", new_password: "", confirm_password: "" });

  async function handlePasswordSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (formData.new_password !== formData.confirm_password) {
      toast.warning(t("profile.security.messages.confirm_mismatch"));
      return;
    }
    setIsSubmitting(true);
    try {
      await changePassword({ old_password: formData.old_password, new_password: formData.new_password });
      toast.success(t("profile.security.messages.password_changed"));
      setTimeout(() => { clearAuthSession(); navigate("/login"); }, 2000);
    } catch (e: any) {
      toast.error(e.message || t("profile.security.messages.change_error"));
    } finally { setIsSubmitting(false); }
  }

  return (
    <form onSubmit={handlePasswordSubmit} className="space-y-6 max-w-md">
      {[
        { id: "current", label: "current_password", key: "old_password" },
        { id: "new", label: "new_password", key: "new_password" },
        { id: "confirm", label: "confirm_password", key: "confirm_password" }
      ].map((field) => (
        <div key={field.id} className="space-y-2">
          <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t(`profile.security.${field.label}`)}</label>
          <div className="relative group">
            <input type="password" required data-testid={`${field.id}-password-input`} className="w-full bg-slate-50 border border-slate-100 rounded-2xl px-5 py-4 focus:ring-4 focus:ring-orange-500/10 focus:bg-white focus:border-orange-300 outline-none transition-all pr-12 font-bold text-sm" value={(formData as any)[field.key]} onChange={(e) => setFormData({ ...formData, [field.key]: e.target.value })} />
            <Lock className="absolute right-4 top-4 text-slate-300 group-focus-within:text-orange-500 transition-colors" size={18} />
          </div>
        </div>
      ))}
      <div className="pt-4">
        <button type="submit" disabled={isSubmitting} data-testid="update-password-btn" className="bg-slate-900 text-white px-8 py-4 rounded-2xl font-black text-xs uppercase tracking-widest hover:bg-black transition-all flex items-center gap-2 active:scale-95 disabled:opacity-50 shadow-xl shadow-slate-200">
          {isSubmitting ? t("common.processing") : t("profile.security.submit_btn")}
        </button>
      </div>
    </form>
  );
}
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
            data-testid="current-password-input"
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
            data-testid="new-password-input"
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
            data-testid="confirm-password-input"
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
          data-testid="update-password-btn"
          className="bg-slate-900 text-white px-6 py-3 rounded-xl font-bold hover:bg-slate-800 transition-all flex items-center gap-2 disabled:opacity-50"
        >
          {isSubmitting ? t("common.processing") : t("profile.security.submit_btn")}
        </button>
      </div>
    </form>
  );
}