export function formatChecklistTypeLabel(
  type: string,
  t: (key: string, options?: { defaultValue?: string }) => string,
): string {
  switch (type.toLowerCase()) {
    case "onboarding":
      return t("lifecycle.types.onboarding", { defaultValue: "Tiếp nhận nhân sự mới (Onboarding)" });
    case "offboarding":
      return t("lifecycle.types.offboarding", { defaultValue: "Bàn giao nghỉ việc (Offboarding)" });
    case "probation_end":
      return t("lifecycle.types.probation", { defaultValue: "Đánh giá hết thử việc (Probation)" });
    case "transfer":
      return t("lifecycle.types.transfer", { defaultValue: "Điều chuyển vị trí / luân chuyển (Transfer)" });
    default:
      return type.toUpperCase();
  }
}
