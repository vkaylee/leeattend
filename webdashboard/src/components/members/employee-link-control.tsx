import { useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import { EmployeeSearch } from "@/components/common/employee-search";
import Button from "@/components/ui/button";
import ConfirmDialog from "@/components/ui/confirm-dialog";
import { useMyPermissions } from "@/hooks/use-permissions";
import { organizationKeys } from "@/hooks/use-organization-queries";
import { linkMemberEmployee, unlinkMemberEmployee } from "@/lib/api";
import { employeeKeys } from "@/lib/query-keys";

interface EmployeeLinkControlProps {
  employeeId?: string | null;
  userId: string;
}

export default function EmployeeLinkControl({ employeeId, userId }: EmployeeLinkControlProps) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const { hasPermission } = useMyPermissions();
  const [selectedEmployeeId, setSelectedEmployeeId] = useState(employeeId ?? "");
  const [confirmUnlink, setConfirmUnlink] = useState(false);
  const [isPending, setIsPending] = useState(false);
  const canSearch = hasPermission("members:employee:link") && Boolean(userId);

  const refresh = () => {
    void queryClient.invalidateQueries({ queryKey: organizationKeys.members() });
    void queryClient.invalidateQueries({ queryKey: employeeKeys.all() });
  };

  const handleLink = async () => {
    if (!selectedEmployeeId || isPending) return;
    setIsPending(true);
    try {
      await linkMemberEmployee(userId, selectedEmployeeId);
      refresh();
    } finally {
      setIsPending(false);
    }
  };

  const handleUnlink = async () => {
    if (isPending) return;
    setIsPending(true);
    try {
      await unlinkMemberEmployee(userId);
      setSelectedEmployeeId("");
      refresh();
      setConfirmUnlink(false);
    } finally {
      setIsPending(false);
    }
  };

  if (!canSearch) return null;

  return (
    <div className="mt-4 space-y-3 border-t border-slate-100 pt-4 dark:border-slate-800">
      <div className="text-xs font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400">
        {t("members.table.employee_link", { defaultValue: "Employee record" })}
      </div>
      {employeeId ? (
        <div className="flex items-center justify-between gap-3">
          <span className="text-sm text-slate-700 dark:text-slate-200">{t("members.table.employee_linked", { defaultValue: "Linked" })}</span>
          <Button disabled={isPending} loading={isPending} onClick={() => { setConfirmUnlink(true); }} size="sm" type="button" variant="outline">
            {t("members.table.unlink_employee", { defaultValue: "Unlink" })}
          </Button>
        </div>
      ) : (
        <div className="space-y-2">
          <EmployeeSearch
            data-testid={`member-employee-search-${userId}`}
            emptyQueryBehavior="prompt-to-type"
            onChange={(value) => { setSelectedEmployeeId(typeof value === "string" ? value : ""); }}
            permission="employees:read"
            placeholder={t("members.table.select_employee", { defaultValue: "Select employee" })}
            value={selectedEmployeeId}
          />
          <Button disabled={!selectedEmployeeId || isPending} loading={isPending} onClick={() => { void handleLink(); }} size="sm" type="button">
            {t("members.table.link_employee", { defaultValue: "Link employee" })}
          </Button>
        </div>
      )}
      <ConfirmDialog
        cancelLabel={t("common.cancel", { defaultValue: "Cancel" })}
        confirmLabel={t("common.confirm", { defaultValue: "Unlink" })}
        isLoading={isPending}
        isOpen={confirmUnlink}
        message={t("members.table.confirm_unlink_employee", { defaultValue: "Unlink this employee account?" })}
        onClose={() => { setConfirmUnlink(false); }}
        onConfirm={handleUnlink}
        title={t("common.warning", { defaultValue: "Warning" })}
        variant="warning"
      />
    </div>
  );
}
