import { Globe, Mail, Save, User as UserIcon, UserCircle } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { Combobox } from "@/components/ui/combobox";
import { LABEL_BASE } from "@/components/ui/form-field";
import { Typography } from "@/components/ui/typography";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { User } from "@/lib/api";

interface ProfileTabProps {
  formData: {
    avatar_url: string;
    bio: string;
    full_name: string;
    preferred_language: string;
  };
  handleSubmit: (e: React.SyntheticEvent) => void;
  isUpdating: boolean;
  setFormData: (data: { avatar_url: string; bio: string; full_name: string; preferred_language: string }) => void;
  user: User | undefined;
}

export const ProfileTab: React.FC<ProfileTabProps> = ({
  formData,
  handleSubmit,
  isUpdating,
  setFormData,
  user,
}) => {
  const { t } = useTranslation();
  const { formatDate } = useDateTimeFormatter();

  return (
    <div className="max-w-5xl bg-white dark:bg-slate-900 rounded-[3rem] border border-slate-100 dark:border-slate-800 shadow-sm overflow-hidden group/hero">
      <div className="h-40 bg-slate-900 relative overflow-hidden">
        <div className="absolute inset-0 bg-gradient-to-br from-primary-600/40 via-blue-600/20 to-transparent" />
        <div className="absolute top-0 left-0 w-full h-full opacity-10 pointer-events-none bg-[radial-gradient(circle_at_50%_50%,rgba(255,255,255,0.1)_0,transparent_50%)]" />
      </div>
      <div className="px-12 pb-10">
        <div className="relative -mt-16 mb-8 flex items-end gap-8">
          <div className="w-32 h-32 rounded-[2.5rem] border-[10px] border-white shadow-2xl bg-slate-100 dark:bg-slate-700 flex items-center justify-center text-slate-300 dark:text-slate-600 overflow-hidden rotate-3 group-hover/hero:rotate-0 transition-transform duration-700">
            {formData.avatar_url ? (
              <img alt="avatar" className="w-full h-full object-cover" src={formData.avatar_url} />
            ) : (
              <UserCircle size={64} />
            )}
          </div>
          <div className="mb-2">
            <Typography className="text-3xl font-bold text-slate-700 dark:text-slate-200 tracking-tighter uppercase leading-none mb-2" variant="h2">
              {user?.full_name ?? user?.email}
            </Typography>
            <Typography className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest opacity-70" variant="p">
              {t("profile.join_date")}: {formatDate(user?.created_at ?? "")}
            </Typography>
          </div>
        </div>

        <form className="space-y-10 pt-4" onSubmit={handleSubmit}>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-10">
            <div className="space-y-3 px-1">
              <label className={LABEL_BASE}>{t("common.email")}</label>
              <div className="flex items-center gap-4 px-6 py-4 bg-slate-100 dark:bg-slate-700/50 border border-slate-200 dark:border-slate-700 rounded-2xl text-slate-500 dark:text-slate-400 font-bold text-sm shadow-inner uppercase tracking-tight opacity-70 cursor-not-allowed">
                <Mail className="opacity-50" size={18} /> {user?.email}
              </div>
            </div>

            <div className="space-y-3 px-1">
              <label className={LABEL_BASE}>{t("employees.table.name")}</label>
              <div className="relative group">
                <input 
                  className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-6 py-4 focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all pr-14 font-bold text-sm tracking-tight shadow-sm" 
                  onChange={(e) => { setFormData({ ...formData, full_name: e.target.value }); }} 
                  required 
                  value={formData.full_name} 
                />
                <UserIcon className="absolute right-5 top-1/2 -translate-y-1/2 text-slate-300 dark:text-slate-600 group-focus-within:text-primary-500 dark:group-focus-within:text-primary-400 transition-colors" size={20} />
              </div>
            </div>

            <div className="space-y-3 px-1">
              <label className={LABEL_BASE}>{t("profile.preferred_language")}</label>
              <Combobox 
                className="bg-slate-50 dark:bg-slate-800 border-slate-100 dark:border-slate-800 h-14" 
                onChange={(value) => { setFormData({ ...formData, preferred_language: value }); }} 
                options={[
                  { value: "", label: t("profile.follow_workspace").toUpperCase(), icon: <Globe size={14} /> }, 
                  { value: "vi", label: t("common.languages.vi").toUpperCase(), icon: <Globe size={14} /> }, 
                  { value: "en", label: t("common.languages.en").toUpperCase(), icon: <Globe size={14} /> }, 
                  { value: "zh", label: t("common.languages.zh").toUpperCase(), icon: <Globe size={14} /> }
                ]} 
                value={formData.preferred_language} 
              />
            </div>
          </div>

          <div className="space-y-3 px-1">
            <label className={LABEL_BASE}>{t("profile.bio")}</label>
            <textarea 
              className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-6 py-4 focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-medium text-sm min-h-[140px] resize-none shadow-sm" 
              onChange={(e) => { setFormData({ ...formData, bio: e.target.value }); }} 
              placeholder={t("profile.bio_placeholder")} 
              rows={3} 
              value={formData.bio} 
            />
          </div>

          <div className="pt-8 border-t border-slate-50 dark:border-slate-800 flex justify-end">
            <Button icon={<Save size={18} />} loading={isUpdating} size="lg" type="submit">
              {isUpdating ? t("common.loading").toUpperCase() : t("common.save").toUpperCase()}
            </Button>
          </div>
        </form>
      </div>
    </div>
  );
};
