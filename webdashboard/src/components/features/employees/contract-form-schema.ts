import { z } from "zod";

export const contractSchema = z.object({
  contract_number: z.string().optional(),
  contract_type: z.enum(["probation", "training", "fixed_term", "indefinite"]),
  assign_phase_id: z.string().optional(),
  status: z.enum(["draft", "active", "expired", "terminated"]).optional(),
  start_date: z.string().min(1, { message: "Ngày hiệu lực là bắt buộc" }),
  end_date: z.string().optional(),
  notes: z.string().optional(),
  enable_esign: z.boolean().optional(),
  esign_auth_type: z.enum(["sms", "email", "smartca", "hsm"]).optional(),
  national_id: z.string().optional(),
  consent_privacy: z.boolean().optional(),
}).superRefine((data, ctx) => {
  if (data.contract_type !== "indefinite") {
    if (!data.end_date || data.end_date.trim() === "") {
      ctx.addIssue({
        code: "custom",
        message: "Ngày hết hạn là bắt buộc đối với loại hợp đồng này",
        path: ["end_date"],
      });
    }
  }
  if (data.start_date && data.end_date) {
    if (new Date(data.start_date) > new Date(data.end_date)) {
      ctx.addIssue({
        code: "custom",
        message: "Ngày hết hạn phải sau ngày hiệu lực",
        path: ["end_date"],
      });
    }
  }
  if (data.enable_esign) {
    if (!data.consent_privacy) {
      ctx.addIssue({
        code: "custom",
        message: "Bạn phải đồng ý chia sẻ thông tin cá nhân để thực hiện ký số",
        path: ["consent_privacy"],
      });
    }
    if (data.esign_auth_type === "smartca" && (!data.national_id || data.national_id.trim() === "")) {
      ctx.addIssue({
        code: "custom",
        message: "Số CCCD/National ID là bắt buộc khi sử dụng SmartCA",
        path: ["national_id"],
      });
    }
  }
});

export type ContractFormValues = z.infer<typeof contractSchema>;
