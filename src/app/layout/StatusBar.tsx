import { useChromeValue } from "@/shared/lib/chrome";
import type { StatusKey } from "@/shared/lib/chrome";
import { Kbd } from "@/shared/ui";

/** Contrat clavier commun à tous les écrans : la palette reste toujours atteignable. */
const ACTIONS: StatusKey = { label: "Actions", shortcut: "mod+k", accent: true };

/**
 * Barre d'état de 34 px (`DESIGN_SYSTEM.md` §9.1) : décompte de l'écran à gauche, en mono,
 * et **contrat clavier du contexte courant** à droite. Les indications de touches ne sont
 * pas focalisables : ce sont des rappels, les raccourcis eux-mêmes restent actifs.
 */
export function StatusBar() {
  const chrome = useChromeValue();
  const keys = [...(chrome.keys ?? []), ACTIONS];

  return (
    <footer
      aria-label="Barre d'état"
      className="flex h-statusbar flex-none items-center gap-[13px] border-t border-bd-soft px-3.5"
    >
      <p aria-live="polite" className="min-w-0 truncate font-mono text-caps text-tx-5">
        {chrome.status}
      </p>
      <ul className="ml-auto flex flex-none items-center gap-[13px] text-sub text-tx-3">
        {keys.map((key) => (
          <li key={key.label} className="flex items-center gap-1.5 whitespace-nowrap">
            {key.label}
            <Kbd shortcut={key.shortcut} tone={key.accent ? "accent" : "chip"} />
          </li>
        ))}
      </ul>
    </footer>
  );
}
