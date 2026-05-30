import React from 'react';
import { cn } from '@/lib/utils';

interface FlagProps {
  className?: string;
}

export function VietnameseFlag({ className }: FlagProps) {
  return (
    <svg
      className={cn("w-5 h-5 rounded-full overflow-hidden shadow-sm border border-slate-200/50 dark:border-slate-800/50 inline-block align-middle select-none flex-shrink-0", className)}
      viewBox="0 0 30 20"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
    >
      <rect width="30" height="20" fill="#da251d" />
      <polygon points="15,4 16.18,7.62 20,7.62 16.91,9.88 18.09,13.5 15,11.25 11.91,13.5 13.09,9.88 10,7.62 13.82,7.62" fill="#ffff00" />
    </svg>
  );
}

export function EnglishFlag({ className }: FlagProps) {
  return (
    <svg
      className={cn("w-5 h-5 rounded-full overflow-hidden shadow-sm border border-slate-200/50 dark:border-slate-800/50 inline-block align-middle select-none flex-shrink-0", className)}
      viewBox="0 0 36 36"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
    >
      <rect width="36" height="36" fill="#fff" />
      <rect width="36" height="3.5" fill="#C8102E" />
      <rect y="7" width="36" height="3.5" fill="#C8102E" />
      <rect y="14" width="36" height="3.5" fill="#C8102E" />
      <rect y="21" width="36" height="3.5" fill="#C8102E" />
      <rect y="28" width="36" height="3.5" fill="#C8102E" />
      <rect y="35" width="36" height="1" fill="#C8102E" />
      <rect width="16" height="20" fill="#012169" />
      <g fill="#fff">
        <circle cx="3" cy="3" r="0.7" />
        <circle cx="7" cy="3" r="0.7" />
        <circle cx="11" cy="3" r="0.7" />
        <circle cx="15" cy="3" r="0.7" />
        <circle cx="5" cy="6" r="0.7" />
        <circle cx="9" cy="6" r="0.7" />
        <circle cx="13" cy="6" r="0.7" />
        <circle cx="3" cy="9" r="0.7" />
        <circle cx="7" cy="9" r="0.7" />
        <circle cx="11" cy="9" r="0.7" />
        <circle cx="15" cy="9" r="0.7" />
        <circle cx="5" cy="12" r="0.7" />
        <circle cx="9" cy="12" r="0.7" />
        <circle cx="13" cy="12" r="0.7" />
        <circle cx="3" cy="15" r="0.7" />
        <circle cx="7" cy="15" r="0.7" />
        <circle cx="11" cy="15" r="0.7" />
        <circle cx="15" cy="15" r="0.7" />
        <circle cx="5" cy="18" r="0.7" />
        <circle cx="9" cy="18" r="0.7" />
        <circle cx="13" cy="18" r="0.7" />
      </g>
    </svg>
  );
}

export function ChineseFlag({ className }: FlagProps) {
  return (
    <svg
      className={cn("w-5 h-5 rounded-full overflow-hidden shadow-sm border border-slate-200/50 dark:border-slate-800/50 inline-block align-middle select-none flex-shrink-0", className)}
      viewBox="0 0 30 20"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
    >
      <rect width="30" height="20" fill="#ee1c25" />
      <polygon points="5,3.33 5.4,4.53 6.67,4.53 5.64,5.28 6.04,6.47 5,5.73 3.96,6.47 4.36,5.28 3.33,4.53 4.6,4.53" fill="#ffff00" />
      <polygon points="10,1 10.15,1.48 10.65,1.48 10.25,1.78 10.4,2.26 10,1.96 9.6,2.26 9.75,1.78 9.35,1.48 9.85,1.48" fill="#ffff00" transform="rotate(10 10 1)" />
      <polygon points="12,3 12.15,3.48 12.65,3.48 12.25,3.78 12.4,4.26 12,3.96 11.6,4.26 11.75,3.78 11.35,3.48 11.85,3.48" fill="#ffff00" transform="rotate(26 12 3)" />
      <polygon points="12,6 12.15,6.48 12.65,6.48 12.25,6.78 12.4,7.26 12,6.96 11.6,7.26 11.75,6.78 11.35,6.48 11.85,6.48" fill="#ffff00" transform="rotate(37 12 6)" />
      <polygon points="10,8 10.15,8.48 10.65,8.48 10.25,8.78 10.4,9.26 10,8.96 9.6,9.26 9.75,8.78 9.35,8.48 9.85,8.48" fill="#ffff00" transform="rotate(10 10 8)" />
    </svg>
  );
}
