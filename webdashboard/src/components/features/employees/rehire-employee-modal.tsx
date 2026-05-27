import { useMutation, useQueryClient } from "@tanstack/react-query";
import { UserPlus } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { format } from "date-fns";

import Button from "@/components/ui/button";
import FormField from "@/components/ui/form-field";
import Modal from "@/components/ui/modal";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import { rehireEmployee, Employee } from "@/lib/api";

interface RehireEmployeeModalProps {
  employeeId: string | null;
  isOpen: boolean;
  onClose: () => void;
  onSuccess?: (employee: Employee) => void;
  wsId: string;
}

export default function RehireEmployeeModal({
  isOpen,
  onClose,
  employeeId,
  wsId,
  onSuccess,
}: RehireEmployeeModalProps) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const toast = useToast();

  const rehireMutation = useMutation({
    mutationFn: (data: { hire_date: string }) => {
      if (!employeeId) throw new Error("No employee selected");
      return rehireEmployee(wsId, employeeId, data);
    },
    onSuccess: (updatedEmp) => {
      void queryClient.invalidateQueries({ queryKey: ["employees", wsId] });
      void queryClient.invalidateQueries({ queryKey: ["employmentHistory", employeeId] });
      onClose();
      toast.success(t("common.success"));
      onSuccess?.(updatedEmp);
    },
  });

  const handleRehireSubmit = (e: React.SyntheticEvent<HTMLFormElement>) => {
    e.preventDefault();
    const formData = new FormData(e.currentTarget);
    rehireMutation.mutate({ hire_date: formData.get("hire_date") as string });
  };

  return (
    <Modal
      icon={<UserPlus size={24} />}
      isOpen={isOpen}
      maxWidth="md"
      onClose={onClose}
      subtitle={t("employees.rehire.subtitle")}
      title={t("employees.rehire.title")}
    >
      <div className="space-y-10 pt-4 animate-in fade-in duration-500">
        <div className="p-8 bg-slate-50 dark:bg-slate-800 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 shadow-inner text-center italic">
          <Typography className="text-sm font-bold text-slate-600 dark:text-slate-300 uppercase tracking-tight leading-relaxed" variant="p">
            &quot;{t("employees.rehire.confirm_msg")}&quot;
          </Typography>
        </div>
        <form className="space-y-8" onSubmit={handleRehireSubmit}>
          <FormField
            defaultValue={format(new Date(), "yyyy-MM-dd")}
            label={t("employees.rehire.rehire_date")}
            name="hire_date"
            required
            type="date"
          />
          <div className="pt-6 grid grid-cols-2 gap-6 border-t border-slate-50 dark:border-slate-800">
            <Button onClick={onClose} size="lg" type="button" variant="outline">
              {t("common.cancel").toUpperCase()}
            </Button>
            <Button data-testid="confirm-rehire-btn"
              loading={rehireMutation.isPending}
              size="lg"
              type="submit"
              variant="primary"
            >
              {rehireMutation.isPending
                ? t("employees.rehire.processing").toUpperCase()
                : t("employees.rehire.confirm").toUpperCase()}
            </Button>
          </div>
        </form>
      </div>
    </Modal>
  );
}
