import React from "react";

interface ToggleSwitchProps {
  checked: boolean;
  onChange: (next: boolean) => void;
}

function ToggleSwitch({ checked, onChange }: ToggleSwitchProps) {
  return (
    <div
      aria-checked={checked}
      className={`relative inline-flex h-5 w-9 items-center rounded-full transition-colors focus:outline-none focus:ring-2 focus:ring-primary-500 focus:ring-offset-2 cursor-pointer ${
        checked ? "bg-primary-600" : "bg-slate-200 dark:bg-slate-700"
      }`}
      onClick={() => { onChange(!checked); }}
      onKeyDown={(e) => {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          onChange(!checked);
        }
      }}
      role="switch"
      tabIndex={0}
    >
      <span
        className={`inline-block h-3.5 w-3.5 transform rounded-full bg-white dark:bg-slate-900 transition-transform ${
          checked ? "translate-x-5" : "translate-x-1"
        }`}
      />
    </div>
  );
}

export default ToggleSwitch;
