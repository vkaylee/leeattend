import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Users, UserPlus } from "lucide-react";
import MemberTable from "@/components/members/MemberTable";
import InviteMemberDialog from "@/components/members/InviteMemberDialog";
import PermissionGate from "@/components/common/PermissionGate";
import Button from "@/components/ui/Button";

export default function MembersPage() {
  const { t } = useTranslation();
  const [isInviteOpen, setIsInviteOpen] = useState(false);

  return (
    <div className="space-y-10 animate-in fade-in duration-700 pb-20">
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-8 px-2">
        <div className="flex items-center gap-5">
          <div className="w-14 h-14 rounded-2xl bg-primary-50 dark:bg-primary-950 flex items-center justify-center text-primary-600 dark:text-primary-400 shadow-inner border border-primary-100 dark:border-primary-800/50">
            <Users size={32} />
          </div>
          <div>
            <h1 className="text-4xl font-bold text-slate-700 dark:text-slate-200 tracking-tighter uppercase leading-none">
              {t("members.title")}
            </h1>
            <p className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] mt-2 opacity-60">
              {t("members.subtitle")}
            </p>
          </div>
        </div>

        <PermissionGate permission="members:invite">
          <Button
            onClick={() => setIsInviteOpen(true)}
            data-testid="invite-member-trigger"
            icon={<UserPlus size={18} />}
          >
            {t("members.invite.title").toUpperCase()}
          </Button>
        </PermissionGate>
      </div>

      <div className="animate-in slide-in-from-bottom-8 duration-700">
        <MemberTable />
      </div>

      <InviteMemberDialog
        isOpen={isInviteOpen}
        onClose={() => setIsInviteOpen(false)}
      />
    </div>
  );
}
