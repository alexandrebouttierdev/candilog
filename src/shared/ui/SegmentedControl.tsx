
/**
 * Groupe segmenté (`COMPONENTS.md` §3.1 du design) : piste `bg-chip`, segment actif
 * `bg-panel` en graisse 500. Remplace les listes déroulantes pour 2 à 4 options courtes
 * (thème, densité, ton, longueur, contrat…).
 */
export function SegmentedControl<TValue extends string>({
  value,
  options,
  onChange,
  label,
  dense = false,
}: {
  value: TValue;
  options: readonly { readonly value: TValue; readonly label: string }[];
  onChange: (value: TValue) => void;
  label: string;
  dense?: boolean;
}) {
  return (
    <div
      role="group"
      aria-label={label}
      className="flex flex-none items-center gap-px rounded-r7 bg-chip p-0.5"
    >
      {options.map((option) => {
        const actif = option.value === value;
        return (
          <button
            key={option.value}
            type="button"
            aria-pressed={actif}
            onClick={() => onChange(option.value)}
            className={[
              "inline-flex h-[22px] items-center gap-1.5 rounded-r6 transition-color",
              dense ? "px-2 text-small" : "px-2.5 text-ui",
              actif ? "bg-panel font-medium text-tx" : "text-tx-4 hover:text-tx-2",
            ].join(" ")}
          >
            {option.label}
          </button>
        );
      })}
    </div>
  );
}
