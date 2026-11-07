import { Users, UserPlus } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import PermissionGate from "@/components/common/permission-gate";
import InviteMemberDialog from "@/components/members/invite-member-dialog";
import MemberTable from "@/components/members/member-table";
import Button from "@/components/ui/button";
import { SettingsHeader } from "@/components/ui/settings/settings-header";

export default function MembersPage() {
  const { t } = useTranslation();
  const [isInviteOpen, setIsInviteOpen] = useState(false);

  return (
    <div className="space-y-8 animate-in fade-in duration-500">
      <SettingsHeader
        actions={
          <PermissionGate permission="members:invite">
            <Button
              className="rounded-xl px-4 py-2 text-xs font-semibold shadow-xs normal-case tracking-normal"
              data-testid="invite-member-trigger"
              icon={<UserPlus size={16} />}
              onClick={() => { setIsInviteOpen(true); }}
              variant="primary"
            >
              {t("members.invite.title")}
            </Button>
          </PermissionGate>
        }
        data-testid="members-title"
        description={t("members.subtitle")}
        icon={<Users size={20} />}
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
