import { Briefcase } from "lucide-react";
import React, { useMemo } from "react";
import { useTranslation } from "react-i18next";

import type { Employer, CreateEmployerRequest, UpdateEmployerRequest } from "@/lib/api/types";

import Button from "@/components/ui/button";
import FormField from "@/components/ui/form-field";
import Modal from "@/components/ui/modal";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import { useDerivedStateFrom } from "@/hooks/use-derived-state-from";

export interface EmployerFormModalProps {
  editingEmployer: Employer | null;
  isLoading: boolean;
  isOpen: boolean;
  onClose: () => void;
  onSubmit: (data: CreateEmployerRequest | UpdateEmployerRequest) => Promise<void>;
}

const INITIAL_STATE = {
  name: "",
  tax_code: "",
  legal_address: "",
  business_type: "",
  registration_number: "",
  phone: "",
  email: "",
  website: "",
  incorporation_date: "",
  country_code: "",
  currency_code: "VND",
};

// eslint-disable-next-line max-lines-per-function -- Form contains many fields for full legal entity info
export function EmployerFormModal({
  editingEmployer,
  isLoading,
  isOpen,
  onClose,
  onSubmit,
}: EmployerFormModalProps) {
  const { t } = useTranslation();
  const toast = useToast();

  // Compute external form data — stabilized via useMemo so reference only
  // changes when isOpen or editingEmployer identity changes.
  const externalFormData = useMemo(() => {
    if (isOpen && editingEmployer) {
      return {
        name: editingEmployer.name,
        tax_code: editingEmployer.tax_code ?? "",
        legal_address: editingEmployer.legal_address ?? "",
        business_type: editingEmployer.business_type ?? "",
        registration_number: editingEmployer.registration_number ?? "",
        phone: editingEmployer.phone ?? "",
        email: editingEmployer.email ?? "",
        website: editingEmployer.website ?? "",
        incorporation_date: editingEmployer.incorporation_date ?? "",
        country_code: editingEmployer.country_code ?? "",
        currency_code: editingEmployer.currency_code ?? "VND",
      };
    }
    return INITIAL_STATE;
  }, [
    isOpen,
    editingEmployer?.id,
    editingEmployer?.name,
    editingEmployer?.tax_code,
    editingEmployer?.legal_address,
    editingEmployer?.business_type,
    editingEmployer?.registration_number,
    editingEmployer?.phone,
    editingEmployer?.email,
    editingEmployer?.website,
    editingEmployer?.incorporation_date,
    editingEmployer?.country_code,
    editingEmployer?.currency_code,
  ]);

  const [formData, setFormData] = useDerivedStateFrom(externalFormData);

  const updateField = (key: keyof typeof formData, value: string) => {
    setFormData((prev) => ({ ...prev, [key]: value }));
  };

  // eslint-disable-next-line complexity
  const handleSubmit = async (e: React.SyntheticEvent<HTMLFormElement>) => {
    e.preventDefault();
    if (!formData.name.trim()) {
      toast.error(t("settings.employers.error_name_required", { defaultValue: "Tên pháp nhân không được để trống" }));
      return;
    }

    if (formData.country_code && formData.country_code.trim().length !== 3) {
      toast.error(t("settings.employers.error_country_code_invalid", { defaultValue: "Mã quốc gia phải gồm đúng 3 ký tự (Ví dụ: VNM, USA)" }));
      return;
    }

    if (formData.currency_code && formData.currency_code.trim().length !== 3) {
      toast.error(t("settings.employers.error_currency_code_invalid", { defaultValue: "Mã tiền tệ phải gồm đúng 3 ký tự (Ví dụ: VND, USD)" }));
      return;
    }

    try {
      const payload = {
        name: formData.name.trim(),
        tax_code: formData.tax_code.trim() || null,
        legal_address: formData.legal_address.trim() || null,
        business_type: formData.business_type.trim() || null,
        registration_number: formData.registration_number.trim() || null,
        phone: formData.phone.trim() || null,
        email: formData.email.trim() || null,
        website: formData.website.trim() || null,
        incorporation_date: formData.incorporation_date.trim() || null,
        country_code: formData.country_code.trim().toUpperCase() || null,
        currency_code: formData.currency_code.trim().toUpperCase() || "VND",
      };

      await onSubmit(payload);
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
      maxWidth="2xl"
      onClose={onClose}
      subtitle={t("settings.employers.modal.subtitle", { defaultValue: "Thông tin chi tiết pháp nhân doanh nghiệp" })}
      title={
        editingEmployer
          ? t("settings.employers.modal.edit_title", { defaultValue: "CẬP NHẬT PHÁP NHÂN" })
          : t("settings.employers.modal.add_title", { defaultValue: "THÊM PHÁP NHÂN MỚI" })
      }
    >
      <form className="space-y-6 pt-4 animate-in fade-in duration-500" onSubmit={handleSubmit}>
        <div className="space-y-4">
          <Typography className="text-sm font-bold text-slate-700 dark:text-slate-300 border-b border-slate-100 dark:border-slate-800 pb-2" variant="h4">
            {t("settings.employers.form.section_general", { defaultValue: "Thông tin chung" })}
          </Typography>
          <FormField
            id="employer-name"
            label={t("settings.employers.form.name_label", { defaultValue: "Tên pháp nhân" })}
            name="name"
            onChange={(e) => { updateField("name", e.target.value); }}
            placeholder={t("settings.employers.form.name_placeholder", { defaultValue: "Nhập tên pháp nhân đầy đủ..." })}
            required
            value={formData.name}
          />
          <FormField
            id="employer-legal-address"
            label={t("settings.employers.form.legal_address_label", { defaultValue: "Địa chỉ pháp lý" })}
            name="legal_address"
            onChange={(e) => { updateField("legal_address", e.target.value); }}
            placeholder={t("settings.employers.form.legal_address_placeholder", { defaultValue: "Nhập địa chỉ đăng ký kinh doanh..." })}
            rows={2}
            type="textarea"
            value={formData.legal_address}
          />
        </div>

        <div className="space-y-4">
          <Typography className="text-sm font-bold text-slate-700 dark:text-slate-300 border-b border-slate-100 dark:border-slate-800 pb-2" variant="h4">
            {t("settings.employers.form.section_legal", { defaultValue: "Thông tin pháp lý" })}
          </Typography>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <FormField
              id="employer-tax-code"
              label={t("settings.employers.form.tax_code_label", { defaultValue: "Mã số thuế" })}
              name="tax_code"
              onChange={(e) => { updateField("tax_code", e.target.value); }}
              placeholder={t("settings.employers.form.tax_code_placeholder", { defaultValue: "Nhập mã số thuế..." })}
              value={formData.tax_code}
            />

            <div className="space-y-3 px-1">
              <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1" htmlFor="employer-business-type">
                {t("settings.employers.form.business_type_label", { defaultValue: "Loại hình doanh nghiệp" })}
              </label>
              <select
                className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-4 font-bold text-sm text-slate-700 dark:text-slate-200 focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all uppercase tracking-tight shadow-sm cursor-pointer h-[var(--ui-height-md)]"
                id="employer-business-type"
                onChange={(e) => { updateField("business_type", e.target.value); }}
                value={formData.business_type}
              >
                <option value="">{t("settings.employers.form.business_type_placeholder", { defaultValue: "Chọn loại hình..." })}</option>
                <option value="llc">LLC (TNHH)</option>
                <option value="corporation">Corporation (Cổ phần)</option>
                <option value="sole_proprietorship">Sole Proprietorship (Tư nhân)</option>
                <option value="partnership">Partnership (Hợp danh)</option>
              </select>
            </div>

            <FormField
              id="employer-registration-number"
              label={t("settings.employers.form.registration_number_label", { defaultValue: "Số đăng ký kinh doanh" })}
              name="registration_number"
              onChange={(e) => { updateField("registration_number", e.target.value); }}
              placeholder={t("settings.employers.form.registration_number_placeholder", { defaultValue: "Nhập số ĐKKD..." })}
              value={formData.registration_number}
            />

            <FormField
              id="employer-incorporation-date"
              label={t("settings.employers.form.incorporation_date_label", { defaultValue: "Ngày thành lập" })}
              name="incorporation_date"
              onChange={(e) => { updateField("incorporation_date", e.target.value); }}
              type="date"
              value={formData.incorporation_date}
            />
          </div>
        </div>

        <div className="space-y-4">
          <Typography className="text-sm font-bold text-slate-700 dark:text-slate-300 border-b border-slate-100 dark:border-slate-800 pb-2" variant="h4">
            {t("settings.employers.form.section_contact", { defaultValue: "Thông tin liên hệ & Tài chính" })}
          </Typography>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <FormField
              id="employer-phone"
              label={t("settings.employers.form.phone_label", { defaultValue: "Số điện thoại" })}
              name="phone"
              onChange={(e) => { updateField("phone", e.target.value); }}
              placeholder={t("settings.employers.form.phone_placeholder", { defaultValue: "Nhập số điện thoại..." })}
              type="tel"
              value={formData.phone}
            />

            <FormField
              id="employer-email"
              label={t("settings.employers.form.email_label", { defaultValue: "Địa chỉ email" })}
              name="email"
              onChange={(e) => { updateField("email", e.target.value); }}
              placeholder={t("settings.employers.form.email_placeholder", { defaultValue: "Nhập địa chỉ email..." })}
              type="email"
              value={formData.email}
            />

            <FormField
              id="employer-website"
              label={t("settings.employers.form.website_label", { defaultValue: "Trang web" })}
              name="website"
              onChange={(e) => { updateField("website", e.target.value); }}
              placeholder={t("settings.employers.form.website_placeholder", { defaultValue: "https://example.com" })}
              value={formData.website}
            />

            <FormField
              id="employer-country-code"
              label={t("settings.employers.form.country_code_label", { defaultValue: "Mã quốc gia (ISO 3166-1 alpha-3)" })}
              name="country_code"
              onChange={(e) => { updateField("country_code", e.target.value); }}
              placeholder={t("settings.employers.form.country_code_placeholder", { defaultValue: "Ví dụ: VNM, USA" })}
              value={formData.country_code}
            />

            <FormField
              id="employer-currency-code"
              label={t("settings.employers.form.currency_code_label", { defaultValue: "Mã tiền tệ (Ví dụ: VND, USD)" })}
              name="currency_code"
              onChange={(e) => { updateField("currency_code", e.target.value); }}
              placeholder={t("settings.employers.form.currency_code_placeholder", { defaultValue: "Ví dụ: VND, USD" })}
              value={formData.currency_code}
            />
          </div>
        </div>

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
