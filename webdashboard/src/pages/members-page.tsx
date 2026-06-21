import { Users, UserPlus } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import PermissionGate from "@/components/common/permission-gate";
import InviteMemberDialog from "@/components/members/invite-member-dialog";
import MemberTable from "@/components/members/member-table";
import Button from "@/components/ui/button";
import PageHeader from "@/components/ui/page-header";

export default function MembersPage() {
  const { t } = useTranslation();
  const [isInviteOpen, setIsInviteOpen] = useState(false);

  return (
    <div className="space-y-6 animate-in fade-in duration-700 pb-20">
      <PageHeader
        actions={
          <PermissionGate permission="members:invite">
            <Button data-testid="invite-member-trigger"
              icon={<UserPlus size={18} />}
              onClick={() => { setIsInviteOpen(true); }}
              variant="ghost"
            >
              {t("members.invite.title").toUpperCase()}
            </Button>
          </PermissionGate>
        }
        data-testid="members-title"
        icon={<Users size={24} />}
        subtitle={t("members.subtitle")}
        title={t("members.title")}
      />

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
