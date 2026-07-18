import React from "react";
import type { UseFormRegister, FieldErrors } from "react-hook-form";

import FormField from "@/components/ui/form-field";
import { Typography } from "@/components/ui/typography";
import type { ContractFormValues } from "./contract-form-schema";

interface EsignSectionProps {
  watchEnableEsign: boolean | undefined;
  watchAuthType: string | undefined;
  errors: FieldErrors<ContractFormValues>;
  register: UseFormRegister<ContractFormValues>;
}

export function EsignSection({
  watchEnableEsign,
  watchAuthType,
  errors,
  register,
}: EsignSectionProps) {
  return (
    <div className="space-y-4 p-5 border border-slate-100 dark:border-slate-800 rounded-3xl bg-slate-50/50 dark:bg-slate-900/30">
      <label className="flex items-center justify-between cursor-pointer" htmlFor="enable_esign">
        <div>
          <span className="text-[11px] font-bold text-slate-700 dark:text-slate-200 uppercase tracking-wider block">
            Kích hoạt ký số điện tử (Enable e-Signature)
          </span>
          <span className="text-[9px] text-slate-400 font-semibold uppercase tracking-wider mt-0.5 block">
            Tự động khởi tạo giao dịch ký số trên FPT/SmartCA
          </span>
        </div>
        <input
          className="h-4 w-4 rounded border-slate-350 text-primary-600 focus:ring-primary-500 cursor-pointer"
          id="enable_esign"
          type="checkbox"
          {...register("enable_esign")}
        />
      </label>

      {!!watchEnableEsign && (
        <div className="space-y-4 pt-4 border-t border-slate-100 dark:border-slate-850 animate-in fade-in duration-300">
          <div className="space-y-2">
            <label
              className="text-[11px] font-bold text-slate-500 uppercase tracking-widest"
              htmlFor="esign_auth_type"
            >
              Phương thức xác thực (Auth Type)
              <select
                className="w-full h-10 px-3 mt-2 bg-slate-50 border border-slate-200 rounded-xl focus:ring-primary-500 focus:border-primary-500 dark:bg-slate-900/50 dark:border-slate-800 text-sm font-semibold"
                id="esign_auth_type"
                {...register("esign_auth_type")}
              >
                <option value="sms">SMS OTP Code</option>
                <option value="email">Email OTP Code</option>
                <option value="smartca">SmartCA Remote CA (VNPT)</option>
                <option value="hsm">Corporate HSM Signature</option>
              </select>
            </label>
          </div>

          {watchAuthType === "smartca" && (
            <FormField
              id="national_id"
              label="Số CCCD / National ID (Bắt buộc đối với SmartCA)"
              {...register("national_id")}
              error={errors.national_id?.message}
              placeholder="VD: 037123456789"
              required
            />
          )}

          <div className="space-y-2">
            <div className="flex items-start gap-2.5">
              <input
                className="mt-0.5 h-4 w-4 rounded border-slate-350 text-primary-600 focus:ring-primary-500 cursor-pointer"
                id="consent_privacy"
                type="checkbox"
                {...register("consent_privacy")}
              />
              <label
                className="text-[9px] text-slate-500 font-bold leading-normal uppercase tracking-wider cursor-pointer select-none"
                htmlFor="consent_privacy"
              >
                Tôi đồng ý chia sẻ thông tin cá nhân (Họ tên, Email, SĐT, CCCD) cho bên nhà cung cấp dịch vụ ký số CeCA để thực hiện giao dịch này. (Nghị định 13/2023/NĐ-CP)
              </label>
            </div>
            {!!errors.consent_privacy && (
              <Typography className="text-xs text-rose-500 font-medium" variant="p">
                {errors.consent_privacy.message}
              </Typography>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
