import { useQueryClient } from "@tanstack/react-query";
import { Calendar } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import FormField from "@/components/ui/form-field";
import Modal from "@/components/ui/modal";
import { useToast } from "@/components/ui/toast";
import { useOrganizationMutation } from "@/hooks/common/use-organization-query";
import { updateEmployeeEmployment } from "@/lib/api";
import { employeeKeys } from "@/lib/query-keys";

interface EditTerminationModalProps {
  currentEndDate: string | null;
  employeeId: string | null;
  employmentId: string | null;
  isOpen: boolean;
  onClose: () => void;
  onSuccess?: () => void;
}

export default function EditTerminationModal({
  isOpen,
  onClose,
  employeeId,
  employmentId,
  currentEndDate,onSuccess,
}: EditTerminationModalProps) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const toast = useToast();

  const editMutation = useOrganizationMutation<
    string,
    Awaited<ReturnType<typeof updateEmployeeEmployment>>
  >({
    keys: [employeeKeys.all()],
    mutationFn: (newEndDate) => {
      if (!employeeId) throw new Error("No employee selected");
      if (!employmentId) throw new Error("No employment record found");
      return updateEmployeeEmployment(employeeId, employmentId, {
        status: "terminated",
        end_date: newEndDate,
      });
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["deptHistory", employeeId] });
      void queryClient.invalidateQueries({ queryKey: ["employee-employments",employeeId] });
      onClose();
      toast.success(t("common.success"));
      onSuccess?.();
    },
  });

  const handleSubmit = (e: React.SyntheticEvent<HTMLFormElement>) => {
    e.preventDefault();
    const formData = new FormData(e.currentTarget);
    editMutation.mutate(formData.get("endDate") as string);
  };

  return (
    <Modal
      icon={<Calendar className="text-primary-600 dark:text-primary-400" size={24} />}
      isOpen={isOpen}
      maxWidth="md"
      onClose={onClose}
      subtitle={t("employees.terminate.edit_date_subtitle")}
      title={t("employees.terminate.edit_date_title")}
    >
      <div className="space-y-10 pt-4 animate-in fade-in duration-500">
        <form className="space-y-8" onSubmit={handleSubmit}>
          <FormField
            defaultValue={currentEndDate ?? ""}
            label={t("employees.terminate.terminate_date")}
            name="endDate"
            required
            type="date"
          />
          <div className="pt-6 grid grid-cols-2 gap-6 border-t border-slate-50 dark:border-slate-800">
            <Button onClick={onClose} size="lg" type="button" variant="outline">
              {t("employees.terminate.cancel").toUpperCase()}
            </Button>
            <Button
              data-testid="confirm-edit-termination-btn"
              loading={editMutation.isPending}
              size="lg"
              type="submit"
              variant="primary"
            >
              {editMutation.isPending
                ? t("employees.terminate.processing").toUpperCase()
                : t("employees.terminate.edit_confirm").toUpperCase()}
            </Button>
          </div>
        </form>
      </div>
    </Modal>
  );
}
