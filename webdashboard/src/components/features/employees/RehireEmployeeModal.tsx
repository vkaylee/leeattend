import React from "react";

import { useMutation, useQueryClient } from "@tanstack/react-query";
import { format } from "date-fns";
import { UserPlus } from "lucide-react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/Button";
import FormField from "@/components/ui/FormField";
import Modal from "@/components/ui/Modal";
import { useToast } from "@/components/ui/Toast";

import { rehireEmployee, Employee } from "@/lib/api";

interface RehireEmployeeModalProps {
  isOpen: boolean;
  onClose: () => void;
  employeeId: string | null;
  wsId: string;
  onSuccess?: (employee: Employee) => void;
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
      queryClient.invalidateQueries({ queryKey: ["employees", wsId] });
      queryClient.invalidateQueries({ queryKey: ["employmentHistory", employeeId] });
      onClose();
      toast.success(t("common.success"));
      onSuccess?.(updatedEmp);
    },
  });

  const handleRehireSubmit = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const formData = new FormData(e.currentTarget);
    rehireMutation.mutate({ hire_date: formData.get("hire_date") as string });
  };

  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      title={t("employees.rehire.title")}
      subtitle={t("employees.rehire.subtitle")}
      icon={<UserPlus size={24} />}
      maxWidth="md"
    >
      <div className="space-y-10 pt-4 animate-in fade-in duration-500">
        <div className="p-8 bg-slate-50 dark:bg-slate-800 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 shadow-inner text-center italic">
          <p className="text-sm font-bold text-slate-600 dark:text-slate-300 uppercase tracking-tight leading-relaxed">
            "{t("employees.rehire.confirm_msg")}"
          </p>
        </div>
        <form onSubmit={handleRehireSubmit} className="space-y-8">
          <FormField
            label={t("employees.rehire.rehire_date")}
            name="hire_date"
            type="date"
            required
            defaultValue={format(new Date(), "yyyy-MM-dd")}
          />
          <div className="pt-6 grid grid-cols-2 gap-6 border-t border-slate-50 dark:border-slate-800">
            <Button variant="outline" size="lg" onClick={onClose} type="button">
              {t("common.cancel").toUpperCase()}
            </Button>
            <Button
              size="lg"
              type="submit"
              data-testid="confirm-rehire-btn"
              loading={rehireMutation.isPending}
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
