import { createLazyFileRoute } from "@tanstack/react-router";

import OnboardingPage from "@/pages/OnboardingPage";

export const Route = createLazyFileRoute('/_authenticated/onboarding')({
  component: OnboardingPage,
});
