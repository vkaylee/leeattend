import { createLazyFileRoute } from "@tanstack/react-router";

import OnboardingPage from "@/pages/onboarding-page";

export const Route = createLazyFileRoute('/_authenticated/onboarding')({
  component: OnboardingPage,
});
