import { Users, UserPlus } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import PermissionGate from "@/components/common/permission-gate";
import InviteMemberDialog from "@/components/members/invite-member-dialog";
import MemberTable from "@/components/members/member-table";
import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";

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
            <Typography className="text-4xl font-bold text-slate-700 dark:text-slate-200 tracking-tighter uppercase leading-none" variant="h1">
              {t("members.title")}
            </Typography>
            <Typography className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] mt-2 opacity-60" variant="p">
              {t("members.subtitle")}
            </Typography>
          </div>
        </div>

        <PermissionGate permission="members:invite">
          <Button
            data-testid="invite-member-trigger"
            icon={<UserPlus size={18} />}
            onClick={() => { setIsInviteOpen(true); }}
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
        onClose={() => { setIsInviteOpen(false); }}
      />
    </div>
  );
}
