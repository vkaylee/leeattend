import { Briefcase } from "lucide-react";
import React, { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";

import type { Employer, CreateEmployerRequest, UpdateEmployerRequest } from "@/lib/api/types";

import Button from "@/components/ui/button";
import FormField from "@/components/ui/form-field";
import Modal from "@/components/ui/modal";
import { useToast } from "@/components/ui/toast";

export interface EmployerFormModalProps {
  editingEmployer: Employer | null;
  isLoading: boolean;
  isOpen: boolean;
  onClose: () => void;
  onSubmit: (data: CreateEmployerRequest | UpdateEmployerRequest) => Promise<void>;
}

export function EmployerFormModal({
  editingEmployer,
  isLoading,
  isOpen,
  onClose,
  onSubmit,
}: EmployerFormModalProps) {
  const { t } = useTranslation();
  const toast = useToast();
  const [name, setName] = useState("");
  const [taxCode, setTaxCode] = useState("");
  const [legalAddress, setLegalAddress] = useState("");

  useEffect(() => {
    if (isOpen) {
      if (editingEmployer) {
        // eslint-disable-next-line react-hooks/set-state-in-effect -- Derived state synced from props; intentional pattern
        setName(editingEmployer.name);
         
        setTaxCode(editingEmployer.tax_code ?? "");
         
        setLegalAddress(editingEmployer.legal_address ?? "");
      } else {
         
        setName("");
         
        setTaxCode("");
         
        setLegalAddress("");
      }
    }
  }, [isOpen, editingEmployer]);

  const handleSubmit = async (e: React.SyntheticEvent<HTMLFormElement>) => {
    e.preventDefault();
    if (!name.trim()) {
      toast.error(t("settings.employers.error_name_required", { defaultValue: "Tên pháp nhân không được để trống" }));
      return;
    }

    try {
      if (editingEmployer) {
        const payload: UpdateEmployerRequest = {
          name: name.trim(),
          tax_code: taxCode.trim() || null,
          legal_address: legalAddress.trim() || null,
        };
        await onSubmit(payload);
      } else {
        const payload: CreateEmployerRequest = {
          name: name.trim(),
          tax_code: taxCode.trim() || undefined,
          legal_address: legalAddress.trim() || undefined,
        };
        await onSubmit(payload);
      }
      onClose();
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      toast.error(msg);
    }
  };

  return (
    <Modal
      icon={<Briefcase size={24} />}
      isOpen={isOpen}
      maxWidth="lg"
      onClose={onClose}
      subtitle={t("settings.employers.modal.subtitle", { defaultValue: "Thông tin chi tiết pháp nhân doanh nghiệp" })}
      title={
        editingEmployer
          ? t("settings.employers.modal.edit_title", { defaultValue: "CẬP NHẬT PHÁP NHÂN" })
          : t("settings.employers.modal.add_title", { defaultValue: "THÊM PHÁP NHÂN MỚI" })
      }
    >
      <form className="space-y-8 pt-4 animate-in fade-in duration-500" onSubmit={handleSubmit}>
        <FormField
          id="employer-name"
          label={t("settings.employers.form.name_label", { defaultValue: "Tên pháp nhân" })}
          name="name"
          onChange={(e) => { setName(e.target.value); }}
          placeholder={t("settings.employers.form.name_placeholder", { defaultValue: "Nhập tên pháp nhân đầy đủ..." })}
          required
          value={name}
        />

        <FormField
          id="employer-tax-code"
          label={t("settings.employers.form.tax_code_label", { defaultValue: "Mã số thuế" })}
          name="tax_code"
          onChange={(e) => { setTaxCode(e.target.value); }}
          placeholder={t("settings.employers.form.tax_code_placeholder", { defaultValue: "Nhập mã số thuế..." })}
          value={taxCode}
        />

        <FormField
          id="employer-legal-address"
          label={t("settings.employers.form.legal_address_label", { defaultValue: "Địa chỉ pháp lý" })}
          name="legal_address"
          onChange={(e) => { setLegalAddress(e.target.value); }}
          placeholder={t("settings.employers.form.legal_address_placeholder", { defaultValue: "Nhập địa chỉ đăng ký kinh doanh..." })}
          rows={3}
          type="textarea"
          value={legalAddress}
        />

        <div className="pt-6 grid grid-cols-2 gap-6 border-t border-slate-50 dark:border-slate-800">
          <Button disabled={isLoading} onClick={onClose} size="lg" variant="outline">
            {t("common.cancel", { defaultValue: "Hủy" }).toUpperCase()}
          </Button>
          <Button
            data-testid="save-employer-btn"
            loading={isLoading}
            size="lg"
            type="submit"
            variant="secondary"
          >
            {editingEmployer
              ? t("common.update", { defaultValue: "Cập nhật" }).toUpperCase()
              : t("settings.employers.modal.submit_btn", { defaultValue: "Tạo mới" }).toUpperCase()}
          </Button>
        </div>
      </form>
    </Modal>
  );
}
