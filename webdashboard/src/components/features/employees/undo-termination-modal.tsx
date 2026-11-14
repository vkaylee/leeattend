import { useQueryClient } from "@tanstack/react-query";
import { RotateCcw } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import Modal from "@/components/ui/modal";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import { useOrganizationMutation } from "@/hooks/common/use-organization-query";
import { updateEmployeeEmployment } from "@/lib/api";
import { employeeKeys, createQueryKey  } from "@/lib/query-keys";


interface UndoTerminationModalProps {
  employeeId: string | null;
  employmentId: string | null;
  isOpen: boolean;
  onClose: () => void;
  onSuccess?: () => void;
}

export default function UndoTerminationModal({
  isOpen,
  onClose,
  employeeId,
  employmentId,onSuccess,
}: UndoTerminationModalProps) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const toast = useToast();

  const undoMutation = useOrganizationMutation<
    undefined,
    Awaited<ReturnType<typeof updateEmployeeEmployment>>
  >({ keys: [employeeKeys.all()],
  mutationFn: () => {
    if (!employeeId) throw new Error("No employee selected");
    if (!employmentId) throw new Error("No employment record found");
    return updateEmployeeEmployment(employeeId, employmentId, {
      status: "active",
      end_date: null,
    });
  },
  onSuccess: () => {
    void queryClient.invalidateQueries({ queryKey: createQueryKey("deptHistory", employeeId) });
    void queryClient.invalidateQueries({ queryKey: createQueryKey("employee-employments",employeeId) });
    onClose();
    toast.success(t("common.success"));
    onSuccess?.();
  }, });

  const handleSubmit = (e: React.SyntheticEvent<HTMLFormElement>) => {
    e.preventDefault();
    undoMutation.mutate(undefined);
  };

  return (
    <Modal
      icon={<RotateCcw className="text-primary-600 dark:text-primary-400" size={24} />}
      isOpen={isOpen}
      maxWidth="md"
      onClose={onClose}
      subtitle={t("employees.terminate.undo_subtitle")}
      title={t("employees.terminate.undo_title")}
    >
      <div className="space-y-10 pt-4 animate-in fade-in duration-500">
        <div className="p-8 bg-slate-50 dark:bg-slate-800 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 shadow-inner text-center italic">
          <Typography className="text-sm font-bold text-slate-600 dark:text-slate-300 uppercase tracking-tight leading-relaxed" variant="p">
            &quot;{t("employees.terminate.undo_confirm_msg")}&quot;
          </Typography>
        </div>
        <form className="space-y-8" onSubmit={handleSubmit}>
          <div className="pt-6 grid grid-cols-2 gap-6 border-t border-slate-50 dark:border-slate-800">
            <Button onClick={onClose} size="lg" type="button" variant="outline">
              {t("employees.terminate.cancel").toUpperCase()}
            </Button>
            <Button
              data-testid="confirm-undo-termination-btn"
              loading={undoMutation.isPending}
              size="lg"
              type="submit"
              variant="primary"
            >
              {undoMutation.isPending
                ? t("employees.terminate.processing").toUpperCase()
                : t("employees.terminate.undo_confirm").toUpperCase()}
            </Button>
          </div>
        </form>
      </div>
    </Modal>
  );
}
