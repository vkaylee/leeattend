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
    e.preventDefault();
    setError(null);

    try {
      await addMember({ email, role });
      setEmail("");
      setRole("member");
      onClose();
    } catch (e) {
      console.error(e);
      setError(t("members.invite.error_message"));
    }
  }

  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      title={t("members.invite.title")}
      subtitle={t("members.invite.subtitle")}
      icon={<UserPlus className="w-6 h-6" />}
    >
      <form onSubmit={handleSubmit} className="space-y-6">
        {error && (
          <div className="p-3 text-sm text-red-600 bg-red-50 rounded-lg">
            {error}
          </div>
        )}

        <div className="space-y-2">
          <label className="text-xs font-bold text-slate-500 uppercase tracking-widest ml-1">
            {t("members.invite.email_label")}
          </label>
          <div className="relative">
            <div className="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
              <Mail className="h-5 w-5 text-slate-400" />
            </div>
            <input
              type="email"
              required
              className="w-full pl-10 pr-4 py-3 bg-slate-50 border border-slate-200 rounded-xl focus:ring-2 focus:ring-primary-500 outline-none transition-all text-sm"
              placeholder="dongnghiep@congty.com"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
            />
          </div>
        </div>

        <div className="space-y-2">
          <label className="text-xs font-bold text-slate-500 uppercase tracking-widest ml-1">
            {t("members.invite.role_label")}
          </label>
          <select
            className="w-full px-4 py-3 bg-slate-50 border border-slate-200 rounded-xl focus:ring-2 focus:ring-primary-500 outline-none transition-all appearance-none text-sm font-semibold"
            value={role}
            onChange={(e) => setRole(e.target.value)}
          >
            <option value="member">{t("members.roles.member")}</option>
            <option value="admin">{t("members.roles.admin")}</option>
          </select>
        </div>

        <div className="pt-2">
          <button
            type="submit"
            disabled={isAdding}
            data-testid="submit-invite-member-btn"
            className="w-full flex items-center justify-center gap-2 bg-slate-900 text-white py-3 rounded-xl font-bold hover:bg-slate-800 transition-all shadow-lg shadow-slate-100 text-sm disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {isAdding ? (
              <Loader2 className="w-4 h-4 animate-spin" />
            ) : (
              <UserPlus className="w-4 h-4" />
            )}
            {t("members.invite.submit_btn")}
          </button>
        </div>
      </form>
    </Modal>
  );
}
