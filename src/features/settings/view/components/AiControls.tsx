import type { ReactNode } from "react";
import { cn } from "@/shared/lib/cn";

export function ControlLabel({ children }: { children: ReactNode }) {
  return <p className="mb-1.5 text-label font-mid text-tx-3">{children}</p>;
}

export function Temperature({ value, onChange }: { value: number; onChange: (value: number) => void }) {
  const part = Math.round((value / 2) * 100);

  return (
    <div className="max-w-[380px]">
      <div className="mb-1.5 flex items-baseline justify-between">
        <p className="text-label font-mid text-tx-3">Température</p>
        <span className="rounded-chip bg-chip px-1.5 py-0.5 font-mono tabular text-meta text-tx">
          {value.toFixed(1)}
        </span>
      </div>
      <input
        type="range"
        min={0}
        max={2}
        step={0.1}
        value={value}
        aria-label="Température"
        onChange={(event) => onChange(Number(event.target.value))}
        style={{
          background: `linear-gradient(to right, var(--color-accent) 0 ${part}%, var(--color-fill) ${part}% 100%)`,
        }}
        className={cn(
          "h-1.5 w-full cursor-pointer appearance-none rounded-full",
          "focus-visible:outline-1 focus-visible:outline-accent-focus",
          "[&::-webkit-slider-thumb]:size-[14px] [&::-webkit-slider-thumb]:appearance-none",
          "[&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:border-2",
          "[&::-webkit-slider-thumb]:border-panel [&::-webkit-slider-thumb]:bg-ac",
          "[&::-moz-range-thumb]:size-[14px] [&::-moz-range-thumb]:rounded-full",
          "[&::-moz-range-thumb]:border-2 [&::-moz-range-thumb]:border-panel",
          "[&::-moz-range-thumb]:bg-ac",
        )}
      />
      <div className="mt-1.5 flex justify-between text-meta text-tx-5">
        <span>Précise</span>
        <span>Créative</span>
      </div>
    </div>
  );
}
