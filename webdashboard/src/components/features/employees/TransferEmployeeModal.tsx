import React from "react";
import { Building2 } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { format } from "date-fns";

import Modal from "@/components/ui/Modal";
import FormField from "@/components/ui/FormField";
import { Combobox } from "@/components/ui/Combobox";
import Button from "@/components/ui/Button";
import { useToast } from "@/components/ui/Toast";

import { transferEmployee, Department } from "@/lib/api";

interface TransferEmployeeModalProps {
  isOpen: boolean;
  onClose: () => void;
  employeeId: string | null;
  wsId: string;
  departments: Department[] | undefined;
  selectedDeptId: string;
  setSelectedDeptId: (id: string) => void;
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
      isOpen={isOpen}
      onClose={() => {
        onClose();
        setSelectedDeptId("");
      }}
      title={t("employees.transfer.title")}
      subtitle={t("employees.transfer.subtitle")}
      icon={<Building2 size={24} />}
      maxWidth="md"
    >
      <form onSubmit={handleTransferSubmit} className="space-y-8 pt-4 animate-in fade-in duration-500">
        <div className="space-y-3 px-1">
          <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">
            {t("employees.transfer.new_dept")}
          </label>
          <Combobox
            options={
              departments?.map((d) => ({
                value: d.id,
                label: d.name.toUpperCase(),
                icon: <Building2 size={14} />,
              })) || []
            }
            value={selectedDeptId}
            onChange={setSelectedDeptId}
            placeholder={t("employees.transfer.select_dept")}
            className="bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 h-14"
          />
        </div>
        <FormField
          label={t("employees.transfer.effective_date")}
          name="effective_date"
          type="date"
          required
          defaultValue={format(new Date(), "yyyy-MM-dd")}
        />
        <FormField
          label={t("employees.transfer.notes")}
          name="notes"
          type="textarea"
          placeholder={t("employees.transfer.notes_placeholder")}
          rows={4}
        />
        <div className="pt-6 grid grid-cols-2 gap-6 border-t border-slate-50 dark:border-slate-800">
          <Button variant="outline" size="lg" onClick={onClose} type="button">
            {t("common.cancel").toUpperCase()}
          </Button>
          <Button
            variant="secondary"
            size="lg"
            type="submit"
            data-testid="submit-transfer-btn"
            loading={transferMutation.isPending}
          >
            {t("employees.transfer.submit").toUpperCase()}
          </Button>
        </div>
      </form>
    </Modal>
  );
}
