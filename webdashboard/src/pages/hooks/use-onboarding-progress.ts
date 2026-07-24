import { useRef, useState } from "react";

interface UseOnboardingProgressReturn {
  showProgress: boolean;
  setShowProgress: (v: boolean) => void;
  progressPercent: number;
  setProgressPercent: React.Dispatch<React.SetStateAction<number>>;
  progressStep: number;
  setProgressStep: React.Dispatch<React.SetStateAction<number>>;
  progressIntervalRef: React.MutableRefObject<NodeJS.Timeout | null>;
  startProgressInterval: () => void;
  clearProgressInterval: () => void;
}

export function useOnboardingProgress(): UseOnboardingProgressReturn {
  const [showProgress, setShowProgress] = useState(false);
  const [progressPercent, setProgressPercent] = useState(0);
  const [progressStep, setProgressStep] = useState(0);
  const progressIntervalRef = useRef<NodeJS.Timeout | null>(null);

  const clearProgressInterval = () => {
    if (progressIntervalRef.current !== null) {
      clearInterval(progressIntervalRef.current);
      progressIntervalRef.current = null;
    }
  };

  const startProgressInterval = () => {
    progressIntervalRef.current = setInterval(() => {
      setProgressPercent((prev) => {
        if (prev >= 98) {
          return 98;
        }
        const step = Math.random() * 4 + 1;
        const nextPercent = Math.min(prev + step, 98);

        const currentStep = nextPercent < 25 ? 1 : nextPercent < 60 ? 2 : nextPercent < 85 ? 3 : 4;
        setProgressStep(currentStep);

        return nextPercent;
      });
    }, 150);
  };

  return {
    showProgress,
    setShowProgress,
    progressPercent,
    setProgressPercent,
    progressStep,
    setProgressStep,
    progressIntervalRef,
    startProgressInterval,
    clearProgressInterval,
  };
}
