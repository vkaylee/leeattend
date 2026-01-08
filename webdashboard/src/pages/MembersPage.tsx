import { useState } from "react";
import { Users, UserPlus } from "lucide-react";
import MemberTable from "@/components/members/MemberTable";
import InviteMemberDialog from "@/components/members/InviteMemberDialog";

export default function MembersPage() {
  const [isInviteOpen, setIsInviteOpen] = useState(false);

  return (
    <div className="space-y-8 animate-in fade-in duration-500">
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h1 className="text-2xl font-bold text-slate-900 flex items-center gap-2">
            <Users className="w-8 h-8 text-indigo-600" />
            Members & Roles
          </h1>
          <p className="text-slate-500 mt-1">
            Manage workspace access and permissions
          </p>
        </div>

        <button
          onClick={() => setIsInviteOpen(true)}
          className="flex items-center gap-2 bg-indigo-600 text-white px-4 py-2.5 rounded-xl font-bold hover:bg-indigo-700 transition-all shadow-lg shadow-indigo-200"
        >
          <UserPlus className="w-5 h-5" />
          Invite Member
        </button>
      </div>

      <MemberTable />

      <InviteMemberDialog
        isOpen={isInviteOpen}
        onClose={() => setIsInviteOpen(false)}
      />
    </div>
  );
}
