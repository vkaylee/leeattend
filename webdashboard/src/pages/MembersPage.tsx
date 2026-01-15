import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Users, UserPlus } from "lucide-react";
import MemberTable from "@/components/members/MemberTable";
import InviteMemberDialog from "@/components/members/InviteMemberDialog";

export default function MembersPage() {
  const { t } = useTranslation();
  const [isInviteOpen, setIsInviteOpen] = useState(false);

  return (
    <div className="space-y-10 animate-enter">
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h1 className="text-3xl font-extrabold text-slate-900 tracking-tight">
            {t("members.title")}
          </h1>
          <p className="text-slate-500 mt-2 text-lg">
            {t("members.subtitle")}
          </p>
        </div>

        <button
          onClick={() => setIsInviteOpen(true)}
          className="flex items-center gap-2 bg-primary-600 text-white px-6 py-3 rounded-2xl font-bold hover:bg-primary-700 hover:scale-105 transition-all shadow-xl shadow-primary-200 active:scale-95"
        >
          <UserPlus className="w-5 h-5" />
          {t("members.invite.title")}
        </button>
      </div>

      <div className="bg-white border border-slate-50 rounded-[2rem] shadow-sm overflow-hidden">
        <MemberTable />
      </div>

      <InviteMemberDialog
        isOpen={isInviteOpen}
        onClose={() => setIsInviteOpen(false)}
      />
    </div>
  );
}
