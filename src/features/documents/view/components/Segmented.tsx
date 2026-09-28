import { cn } from "@/shared/lib/cn";

/** Choix exclusif compact des panneaux de génération (ton, longueur). */
export function Segmented<T extends string>({
  label,
  value,
  options,
  disabled,
  onChange,
}: {
  label: string;
  value: string;
  options: ReadonlyArray<{ value: T; label: string }>;
  disabled: boolean;
  onChange: (value: T) => void;
}) {
  return (
    <div role="radiogroup" aria-label={label} className="flex gap-px rounded-r7 bg-chip p-0.5">
      {options.map((option) => (
        <button
          key={option.value}
          type="button"
          role="radio"
          aria-checked={value === option.value}
          disabled={disabled}
          onClick={() => onChange(option.value)}
          className={cn(
            "h-6 flex-1 rounded-r5 text-small",
            value === option.value ? "bg-panel font-medium text-tx" : "text-tx-4 hover:text-tx-2",
          )}
        >
          {option.label}
        </button>
      ))}
    </div>
  );
}
