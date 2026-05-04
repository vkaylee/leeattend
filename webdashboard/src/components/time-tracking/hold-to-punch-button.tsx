import { Loader2 } from "lucide-react";
import React, { useEffect, useRef, useState } from "react";

interface HoldToPunchButtonProps {
  isActive?: boolean;
  isPunching: boolean;
  label: string;
  onPunch: () => void;
}

export function HoldToPunchButton({ onPunch, isPunching, label, isActive }: HoldToPunchButtonProps) {
  const [progress, setProgress] = useState(0);
  const intervalRef = useRef<NodeJS.Timeout | null>(null);
  const isComplete = useRef(false);

  const startHold = () => {
    if (isPunching || isComplete.current) return;
    
    const startTime = Date.now();
    const duration = 1000; // 1 second hold to confirm

    intervalRef.current = setInterval(() => {
      const elapsed = Date.now() - startTime;
      const newProgress = Math.min((elapsed / duration) * 100, 100);
      setProgress(newProgress);

      if (newProgress >= 100) {
        if (intervalRef.current) {
          clearInterval(intervalRef.current);
        }
        isComplete.current = true;
        onPunch();
      }
    }, 16); // roughly 60fps
  };

  const endHold = () => {
    if (intervalRef.current) {
      clearInterval(intervalRef.current);
      intervalRef.current = null;
    }
    if (!isComplete.current) {
      setProgress(0);
    }
  };

  // Reset after punching completes
  useEffect(() => {
    if (!isPunching) {
      // Small delay to allow transition out
      setTimeout(() => {
        setProgress(0);
        isComplete.current = false;
      }, 500);
    }
  }, [isPunching]);

  return (
    <button
      className={`relative w-full overflow-hidden py-6 rounded-[1.5rem] font-bold uppercase tracking-widest text-xs transition-all shadow-2xl z-10 disabled:opacity-50 select-none ${
        isActive 
          ? 'bg-rose-600 text-white shadow-rose-200 dark:shadow-rose-900/20 hover:bg-rose-700' 
          : 'bg-primary-600 text-white shadow-primary-200 dark:shadow-primary-900/20 hover:bg-primary-700'
      }`}
      disabled={isPunching}
      onContextMenu={(e) => { e.preventDefault(); }} // Prevent context menu on long press on mobile
      onPointerCancel={endHold}
      onPointerDown={(e) => {
        // Only left click or touch
        if (e.button !== 0 && e.pointerType === "mouse") return;
        const target = e.target as HTMLElement;
        try {
          target.setPointerCapture(e.pointerId);
        } catch {
          // Ignore in jsdom environments where setPointerCapture might not exist or throws
        }
        startHold();
      }}
      onPointerLeave={endHold}
      onPointerUp={(e) => {
        const target = e.target as HTMLElement;
        try {
          target.releasePointerCapture(e.pointerId);
        } catch {
          // Ignore
        }
        endHold();
      }}
      style={{ touchAction: "none" }} // Prevent scrolling while holding
    >
      <div 
        className="absolute inset-0 bg-black/20 dark:bg-white/20 origin-left"
        style={{ 
          transform: `scaleX(${progress / 100})`,
          transition: progress === 0 ? 'transform 0.3s ease-out' : 'none'
        }}
      />
      <div className="relative z-10 flex items-center justify-center gap-2">
        {isPunching ? (
          <Loader2 className="w-4 h-4 animate-spin" />
        ) : (
          label
        )}
      </div>
    </button>
  );
}
