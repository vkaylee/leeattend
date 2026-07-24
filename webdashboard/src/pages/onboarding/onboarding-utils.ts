export function getValidationError(dbMode: "auto" | "byod", databaseUrl: string, t: (key: string) => string): string | null {
  if (dbMode === "byod" && databaseUrl) {
    const isValid = /^postgres(ql)?:\/\/.+/i.test(databaseUrl.trim());
    return isValid ? null : t("onboarding.db_validation_error");
  }
  return null;
}
