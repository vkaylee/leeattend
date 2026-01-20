import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Users, UserPlus } from "lucide-react";
import MemberTable from "@/components/members/MemberTable";
import InviteMemberDialog from "@/components/members/InviteMemberDialog";

export default function MembersPage() {
  const { t } = useTranslation();
  const [isInviteOpen, setIsInviteOpen] = useState(false);

  return (
    <div className="space-y-8 animate-in fade-in duration-700">
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-6 px-2">
        <div className="flex items-center gap-4">
          <div className="w-12 h-12 rounded-[1.25rem] bg-primary-50 flex items-center justify-center text-primary-600 shadow-sm border border-primary-100/50">
            <Users size={24} />
          </div>
          <div>
            <h1 className="text-3xl font-bold text-slate-900 tracking-tight leading-tight">
              {t("members.title")}
            </h1>
            <p className="text-slate-500 mt-1 font-medium">
              {t("members.subtitle")}
            </p>
          </div>
        </div>

        <button
          onClick={() => setIsInviteOpen(true)}
          data-testid="invite-member-trigger"
          className="flex items-center gap-2 bg-primary-600 text-white px-6 py-2.5 rounded-2xl font-black text-[10px] uppercase tracking-widest hover:bg-primary-700 transition-all shadow-lg shadow-primary-100 active:scale-95"
        >
          <UserPlus size={16} />
          {t("members.invite.title")}
        </button>
      </div>

      <div className="bg-white border border-slate-200 rounded-[2.5rem] shadow-sm overflow-hidden animate-in slide-in-from-bottom-4 duration-500 delay-150">
        <MemberTable />
      </div>

      <InviteMemberDialog
        isOpen={isInviteOpen}
        onClose={() => setIsInviteOpen(false)}
      />
    </div>
  );
}
