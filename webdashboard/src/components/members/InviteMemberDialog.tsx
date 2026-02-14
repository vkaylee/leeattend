import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Mail, UserPlus, Loader2 } from "lucide-react";
import { useWorkspaceMembers } from "@/hooks/use-workspace";
import Modal from "../ui/Modal";

interface InviteMemberDialogProps {
  isOpen: boolean;
  onClose: () => void;
}

export default function InviteMemberDialog({
  isOpen,
  onClose,
}: InviteMemberDialogProps) {
  const { t } = useTranslation();
  const { addMember, isAdding } = useWorkspaceMembers();
  const [email, setEmail] = useState("");
  const [role, setRole] = useState("member");
  const [error, setError] = useState<string | null>(null);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault(); setError(null);
    try {
      await addMember({ email, role });
      setEmail(""); setRole("member"); onClose();
    } catch (e) {
      setError(t("members.invite.error_message"));
    }
  }

  return (
    <Modal isOpen={isOpen} onClose={onClose} title={t("members.invite.title")} subtitle={t("members.invite.subtitle")} icon={<UserPlus size={24} />} maxWidth="md">
      <form onSubmit={handleSubmit} className="space-y-8 pt-4 animate-in fade-in duration-500">
        {error && (
          <div className="p-4 text-[11px] font-bold uppercase tracking-widest text-rose-600 dark:text-rose-300 bg-rose-50 dark:bg-rose-950 border border-rose-100 dark:border-rose-800 rounded-2xl animate-in slide-in-from-top-2">
            {error}
          </div>
        )}

        <div className="space-y-3">
          <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{t("members.invite.email_label")}</label>
          <div className="relative group">
            <div className="absolute inset-y-0 left-0 pl-5 flex items-center pointer-events-none text-slate-300 dark:text-slate-600 group-focus-within:text-primary-500 dark:group-focus-within:text-primary-400 transition-colors">
              <Mail size={18} />
            </div>
            <input type="email" required className="w-full pl-14 pr-6 py-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-bold text-sm tracking-tight" placeholder="dongnghiep@congty.com" value={email} onChange={(e) => setEmail(e.target.value)} />
          </div>
        </div>

        <div className="space-y-3">
          <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{t("members.invite.role_label")}</label>
          <select className="w-full px-5 py-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-bold text-sm uppercase tracking-tight shadow-sm appearance-none cursor-pointer" value={role} onChange={(e) => setRole(e.target.value)}>
            <option value="member">{t("members.roles.member").toUpperCase()}</option>
            <option value="admin">{t("members.roles.admin").toUpperCase()}</option>
          </select>
        </div>

        <div className="pt-2">
          <button type="submit" disabled={isAdding} data-testid="submit-invite-member-btn" className="w-full flex items-center justify-center gap-3 bg-slate-900 text-white py-4 rounded-2xl font-bold text-xs uppercase tracking-widest hover:bg-primary-700 transition-all shadow-xl shadow-slate-200 dark:shadow-black/30 active:scale-95 disabled:opacity-50">
            {isAdding ? <Loader2 size={18} className="animate-spin" /> : <UserPlus size={18} />}
            {t("members.invite.submit_btn").toUpperCase()}
          </button>
        </div>
      </form>
    </Modal>
  );
}
