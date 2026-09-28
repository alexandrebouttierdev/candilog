import type { ProfileSection } from "@/shared/types/generated/ai";
import { Switch } from "@/shared/ui";
import { cn } from "@/shared/lib/cn";
import type { SectionOption } from "../../model/profileSections";
import { PaneSection } from "./GeneratorFrame";

/**
 * « Ce que l'IA peut utiliser » (CV) ou « Arguments autorisés » (lettre) : une ligne par
 * section du profil, son nombre d'éléments, un interrupteur. Une section vide ne se règle
 * pas — il n'y a rien à retirer.
 */
export function SectionToggles({
  title,
  options,
  excluded,
  disabled,
  onToggle,
}: {
  title: string;
  options: readonly SectionOption[];
  excluded: readonly ProfileSection[];
  disabled: boolean;
  onToggle: (section: ProfileSection) => void;
}) {
  const available = options.filter((option) => option.count > 0);
  const allowed = available.filter((option) => !excluded.includes(option.section)).length;
  return (
    <PaneSection title={title} aside={`${allowed} / ${available.length}`}>
      <ul className="flex flex-col gap-1.5">
        {options.map((option) => {
          const empty = option.count === 0;
          const on = !empty && !excluded.includes(option.section);
          return (
            <li key={option.section} className="flex items-center gap-2.5">
              <Switch
                checked={on}
                disabled={disabled || empty}
                label={option.label}
                onChange={() => onToggle(option.section)}
              />
              <span className={cn("flex-1 text-small", on ? "text-tx-2" : "text-tx-5")}>{option.label}</span>
              <span className="font-mono text-caps text-tx-6">{option.count}</span>
            </li>
          );
        })}
      </ul>
    </PaneSection>
  );
}
