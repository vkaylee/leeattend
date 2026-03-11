import { useState, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { useToast } from "../components/ui/Toast";
import {
  User as UserIcon,
  Mail,
  UserCircle,
  Save,
  Lock,
  ShieldCheck,
  Palette,
  Undo2,
  Globe,
  RefreshCw,
} from "lucide-react";
import { useProfile } from "@/hooks/use-profile";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { useTheme } from "@/hooks/use-theme";
import { useTranslation } from "react-i18next";
import { Combobox } from "@/components/ui/Combobox";
import { changePassword, clearAuthSession } from "@/lib/api";
import { cn } from "@/lib/utils";
import Button from "@/components/ui/Button";
import { LABEL_BASE } from "@/components/ui/FormField";

type TabType = "profile" | "personalization" | "security";

export default function ProfilePage() {
  const { t } = useTranslation();
  const { user, isLoading, updateProfile, isUpdating } = useProfile();
  const { formatDate } = useDateTimeFormatter();
  const { primaryColor, isOverridden, setUserTheme } = useTheme();
  const toast = useToast();
  const [activeTab, setActiveTab] = useState<TabType>("profile");

  const [formData, setFormData] = useState({
    full_name: "", avatar_url: "", bio: "", preferred_language: "",
  });

  useEffect(() => {
    if (user) {
      // eslint-disable-next-line react-hooks/set-state-in-effect
      setFormData({
        full_name: user.full_name || "", avatar_url: user.avatar_url || "",
        bio: user.bio || "", preferred_language: user.preferred_language || "",
      });
    }
  }, [user]);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    try {
      await updateProfile(formData);
      toast.success(t("common.success"));
    } catch (e) { console.error(e); }
  }

  if (isLoading) return <div className="p-20 text-center"><div className="w-10 h-10 border-4 border-primary-600 border-t-transparent rounded-full animate-spin mx-auto"></div></div>;

  return (
    <div className="space-y-10 animate-in fade-in duration-700 pb-20">
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-8 px-2">
        <div className="space-y-1">
          <h1 className="text-4xl font-bold tracking-tighter text-slate-700 dark:text-slate-200 uppercase leading-none">{t("common.profile")}</h1>
          <p className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] opacity-60">{t("profile.subtitle")}</p>
        </div>

        <div className="flex bg-slate-100 dark:bg-slate-700/50 p-1.5 rounded-[22px] w-fit border border-slate-200 dark:border-slate-700/50 shadow-inner">
          <button onClick={() => setActiveTab("profile")} className={cn("flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all", activeTab === 'profile' ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800" : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 dark:text-slate-700")}><UserCircle size={14} /> {t("common.profile")}</button>
          <button onClick={() => setActiveTab("personalization")} className={cn("flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all", activeTab === 'personalization' ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800" : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 dark:text-slate-700")}><Palette size={14} /> {t("profile.personalization.title")}</button>
          <button onClick={() => setActiveTab("security")} className={cn("flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all", activeTab === 'security' ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800" : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 dark:text-slate-700")}><ShieldCheck size={14} /> {t("profile.security.title")}</button>
        </div>
      </div>

      <div className="animate-in slide-in-from-bottom-8 duration-700">
        {activeTab === "profile" && (
          <div className="max-w-5xl bg-white dark:bg-slate-900 rounded-[3rem] border border-slate-100 dark:border-slate-800 shadow-sm overflow-hidden group/hero">
            <div className="h-40 bg-slate-900 relative overflow-hidden">
               <div className="absolute inset-0 bg-gradient-to-br from-primary-600/40 via-indigo-600/20 to-transparent" />
               <div className="absolute top-0 left-0 w-full h-full opacity-10 pointer-events-none bg-[radial-gradient(circle_at_50%_50%,rgba(255,255,255,0.1)_0,transparent_50%)]" />
            </div>
            <div className="px-12 pb-10">
              <div className="relative -mt-16 mb-8 flex items-end gap-8">
                <div className="w-32 h-32 rounded-[2.5rem] border-[10px] border-white shadow-2xl bg-slate-100 dark:bg-slate-700 flex items-center justify-center text-slate-300 dark:text-slate-600 overflow-hidden rotate-3 group-hover/hero:rotate-0 transition-transform duration-700">
                  {formData.avatar_url ? <img src={formData.avatar_url} className="w-full h-full object-cover" alt="avatar" /> : <UserCircle size={64} />}
                </div>
                <div className="mb-2">
                  <h2 className="text-3xl font-bold text-slate-700 dark:text-slate-200 tracking-tighter uppercase leading-none mb-2">{user?.full_name || user?.email}</h2>
                  <p className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest opacity-70">{t("profile.join_date")}: {formatDate(user?.created_at || "")}</p>
                </div>
              </div>

              <form onSubmit={handleSubmit} className="space-y-10 pt-4">
                <div className="grid grid-cols-1 md:grid-cols-2 gap-10">
                  <div className="space-y-3 px-1">
                    <label className={LABEL_BASE}>{t("common.email")}</label>
                    <div className="flex items-center gap-4 px-6 py-4 bg-slate-100 dark:bg-slate-700/50 border border-slate-200 dark:border-slate-700 rounded-2xl text-slate-500 dark:text-slate-400 font-bold text-sm shadow-inner uppercase tracking-tight opacity-70 cursor-not-allowed">
                      <Mail size={18} className="opacity-50" /> {user?.email}
                    </div>
                  </div>

                  <div className="space-y-3 px-1">
                    <label className={LABEL_BASE}>{t("employees.table.name")}</label>
                    <div className="relative group">
                      <input required className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-6 py-4 focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all pr-14 font-bold text-sm uppercase tracking-tight shadow-sm" value={formData.full_name} onChange={(e) => setFormData({ ...formData, full_name: e.target.value })} />
                      <UserIcon className="absolute right-5 top-1/2 -translate-y-1/2 text-slate-300 dark:text-slate-600 group-focus-within:text-primary-500 dark:group-focus-within:text-primary-400 transition-colors" size={20} />
                    </div>
                  </div>

                  <div className="space-y-3 px-1">
                    <label className={LABEL_BASE}>{t("profile.preferred_language")}</label>
                    <Combobox options={[{ value: "", label: t("profile.follow_workspace").toUpperCase(), icon: <Globe size={14} /> }, { value: "vi", label: t("common.languages.vi").toUpperCase(), icon: <Globe size={14} /> }, { value: "en", label: t("common.languages.en").toUpperCase(), icon: <Globe size={14} /> }, { value: "zh", label: t("common.languages.zh").toUpperCase(), icon: <Globe size={14} /> }]} value={formData.preferred_language} onChange={(val) => setFormData({ ...formData, preferred_language: val })} className="bg-slate-50 dark:bg-slate-800 border-slate-100 dark:border-slate-800 h-14" />
                  </div>
                </div>

                <div className="space-y-3 px-1">
                  <label className={LABEL_BASE}>{t("profile.bio")}</label>
                  <textarea className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-6 py-4 focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-medium text-sm min-h-[140px] resize-none shadow-sm" rows={3} value={formData.bio} onChange={(e) => setFormData({ ...formData, bio: e.target.value })} placeholder={t("profile.bio_placeholder")} />
                </div>

                <div className="pt-8 border-t border-slate-50 dark:border-slate-800 flex justify-end">
                  <Button size="lg" type="submit" loading={isUpdating} icon={<Save size={18} />}>{isUpdating ? t("common.loading").toUpperCase() : t("common.save").toUpperCase()}</Button>
                </div>
              </form>
            </div>
          </div>
        )}

        {activeTab === "personalization" && (
          <div className="max-w-5xl bg-white dark:bg-slate-900 rounded-[3rem] border border-slate-100 dark:border-slate-800 shadow-sm p-12 space-y-12 animate-in fade-in duration-500">
            <div className="flex flex-col md:flex-row md:items-center justify-between gap-6 px-2">
              <div className="flex items-center gap-5">
                <div className="w-14 h-14 rounded-2xl bg-pink-50 flex items-center justify-center text-pink-600 shadow-inner border border-pink-100/50"><Palette size={28} /></div>
                <div>
                  <h2 className="text-2xl font-bold text-slate-700 dark:text-slate-200 tracking-tighter uppercase leading-none mb-1">{t("profile.personalization.title")}</h2>
                  <p className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest opacity-70">{t("profile.personalization.subtitle")}</p>
                </div>
              </div>
              {isOverridden && (
                <button onClick={() => setUserTheme(null)} className="flex items-center gap-3 text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400 hover:text-primary-600 dark:text-primary-400 transition-all bg-slate-50 dark:bg-slate-800 px-6 py-2.5 rounded-xl border border-slate-100 dark:border-slate-800 shadow-sm active:scale-95"><Undo2 size={14} /> {t("profile.personalization.use_workspace_default").toUpperCase()}</button>
              )}
            </div>

            <div className="space-y-10">
              <div className="space-y-6">
                <label className={LABEL_BASE}>{t("common.workspace_settings.primary_color")}</label>
                <div className="flex flex-wrap gap-5 items-center">
                  {[
                    { name: t("common.colors.violet"), color: '#7c3aed' }, { name: t("common.colors.indigo"), color: '#4f46e5' }, { name: t("common.colors.blue"), color: '#2563eb' },
                    { name: t("common.colors.emerald"), color: '#059669' }, { name: t("common.colors.amber"), color: '#d97706' }, { name: t("common.colors.rose"), color: '#e11d48' },
                    { name: t("common.colors.slate"), color: '#475569' }, { name: t("common.colors.black"), color: '#0f172a' },
                  ].map((preset) => (
                    <button key={preset.color} onClick={() => setUserTheme(preset.color)} className={cn("w-14 h-14 rounded-2xl transition-all border-4 flex items-center justify-center shadow-lg group active:scale-90", primaryColor === preset.color ? 'border-slate-900 scale-110' : 'border-white hover:scale-105 hover:border-slate-100 dark:border-slate-800')} style={{ backgroundColor: preset.color }} title={preset.name}>
                      {primaryColor === preset.color && <div className="w-3 h-3 bg-white dark:bg-slate-900 rounded-full shadow-2xl animate-in zoom-in duration-300" />}
                    </button>
                  ))}
                  <div className="h-12 w-px bg-slate-100 dark:bg-slate-700 mx-2" />
                  <div className="flex items-center gap-4 bg-slate-50 dark:bg-slate-800 p-2.5 rounded-[1.5rem] border border-slate-100 dark:border-slate-800 shadow-inner group">
                    <input type="color" value={primaryColor} onChange={(e) => setUserTheme(e.target.value)} className="w-12 h-12 rounded-xl cursor-pointer bg-transparent border-none p-0 overflow-hidden shadow-sm" />
                    <input type="text" value={primaryColor} onChange={(e) => setUserTheme(e.target.value)} className="bg-transparent border-none text-sm font-mono font-bold text-slate-600 dark:text-slate-300 w-24 outline-none uppercase tracking-tighter" />
                  </div>
                </div>
              </div>
              <div className="p-10 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-[2.5rem] shadow-inner"><p className="text-[11px] font-bold text-slate-500 dark:text-slate-400 leading-relaxed italic uppercase tracking-tight opacity-80">{t("profile.personalization.note")}</p></div>
            </div>
          </div>
        )}

        {activeTab === "security" && (
          <div className="max-w-5xl bg-white dark:bg-slate-900 rounded-[3rem] border border-slate-100 dark:border-slate-800 shadow-sm p-12 space-y-12 animate-in fade-in duration-500">
            <div className="flex items-center gap-5">
              <div className="w-14 h-14 rounded-2xl bg-orange-50 dark:bg-orange-950 flex items-center justify-center text-orange-600 dark:text-orange-300 shadow-inner border border-orange-100/50"><ShieldCheck size={28} /></div>
              <div>
                <h2 className="text-2xl font-bold text-slate-700 dark:text-slate-200 tracking-tighter uppercase leading-none mb-1">{t("profile.security.title")}</h2>
                <p className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest opacity-70">{t("profile.security.subtitle")}</p>
              </div>
            </div>
            <div className="bg-slate-50 dark:bg-slate-800/50 p-10 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 shadow-inner"><ChangePasswordForm /></div>
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
      toast.warning(t("profile.security.messages.confirm_mismatch")); return;
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
    <form onSubmit={handlePasswordSubmit} className="space-y-8 max-w-lg">
      {[
        { id: "current", label: "current_password", key: "old_password" },
        { id: "new", label: "new_password", key: "new_password" },
        { id: "confirm", label: "confirm_password", key: "confirm_password" }
      ].map((field) => (
        <div key={field.id} className="space-y-3 px-1">
          <label className={LABEL_BASE}>{t(`profile.security.${field.label}`)}</label>
          <div className="relative group">
            <input type="password" required data-testid={`${field.id}-password-input`} className="w-full bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-2xl px-6 py-4 focus:ring-4 focus:ring-orange-500/10 focus:border-orange-300 outline-none transition-all pr-14 font-bold text-sm tracking-tight shadow-sm" value={(formData as any)[field.key]} onChange={(e) => setFormData({ ...formData, [field.key]: e.target.value })} />
            <Lock className="absolute right-5 top-1/2 -translate-y-1/2 text-slate-300 dark:text-slate-600 group-focus-within:text-orange-500 transition-colors" size={20} />
          </div>
        </div>
      ))}
      <div className="pt-6 border-t border-slate-100 dark:border-slate-800 flex justify-end">
        <Button variant="secondary" size="lg" type="submit" loading={isSubmitting} data-testid="update-password-btn">
          {t("profile.security.submit_btn").toUpperCase()}
        </Button>
      </div>
    </form>
  );
}
