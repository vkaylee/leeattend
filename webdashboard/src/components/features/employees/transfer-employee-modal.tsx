import { useMutation, useQueryClient } from "@tanstack/react-query";
import { Building2 } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { format } from "date-fns";

import Button from "@/components/ui/button";
import { Combobox } from "@/components/ui/combobox";
import FormField from "@/components/ui/form-field";
import Modal from "@/components/ui/modal";
import { useToast } from "@/components/ui/toast";
import { transferEmployee, Department } from "@/lib/api";

interface TransferEmployeeModalProps {
  departments: Department[] | undefined;
  employeeId: string | null;
  isOpen: boolean;
  onClose: () => void;
  selectedDeptId: string;
  setSelectedDeptId: (id: string) => void;
  wsId: string;
}

export default function TransferEmployeeModal({
  isOpen,
  onClose,
  employeeId,
  wsId,
  departments,
  selectedDeptId,
  setSelectedDeptId,
}: TransferEmployeeModalProps) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const toast = useToast();

  const transferMutation = useMutation({
    mutationFn: (data: { department_id: string; effective_date: string; notes?: string }) => {
      if (!employeeId) throw new Error("No employee selected");
      return transferEmployee(wsId, employeeId, data);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["deptHistory", employeeId] });
      onClose();
      setSelectedDeptId("");
      toast.success(t("common.success"));
    },
  });

  const handleTransferSubmit = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const formData = new FormData(e.currentTarget);
    transferMutation.mutate({
      department_id: selectedDeptId,
      effective_date: formData.get("effective_date") as string,
      notes: (formData.get("notes") as string) || undefined,
    });
  };

  return (
    <Modal
      icon={<Building2 size={24} />}
      isOpen={isOpen}
      maxWidth="md"
      onClose={() => {
        onClose();
        setSelectedDeptId("");
      }}
      subtitle={t("employees.transfer.subtitle")}
      title={t("employees.transfer.title")}
    >
      <form className="space-y-8 pt-4 animate-in fade-in duration-500" onSubmit={handleTransferSubmit}>
        <div className="space-y-3 px-1">
          <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">
            {t("employees.transfer.new_dept")}
          </label>
          <Combobox
            className="bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 h-14"
            onChange={setSelectedDeptId}
            options={
              departments?.map((d) => ({
                value: d.id,
                label: d.name.toUpperCase(),
                icon: <Building2 size={14} />,
              })) || []
            }
            placeholder={t("employees.transfer.select_dept")}
            value={selectedDeptId}
          />
        </div>
        <FormField
          defaultValue={format(new Date(), "yyyy-MM-dd")}
          label={t("employees.transfer.effective_date")}
          name="effective_date"
          required
          type="date"
        />
        <FormField
          label={t("employees.transfer.notes")}
          name="notes"
          placeholder={t("employees.transfer.notes_placeholder")}
          rows={4}
          type="textarea"
        />
        <div className="pt-6 grid grid-cols-2 gap-6 border-t border-slate-50 dark:border-slate-800">
          <Button onClick={onClose} size="lg" type="button" variant="outline">
            {t("common.cancel").toUpperCase()}
          </Button>
          <Button
            data-testid="submit-transfer-btn"
            loading={transferMutation.isPending}
            size="lg"
            type="submit"
            variant="secondary"
          >
            {t("employees.transfer.submit").toUpperCase()}
          </Button>
        </div>
      </form>
    </Modal>
  );
}
