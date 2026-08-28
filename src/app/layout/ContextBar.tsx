import type { ReactNode } from "react";
import { createContext, useContext, useState } from "react";
import { createPortal } from "react-dom";
import { Icon } from "@/shared/ui/Icon";

/**
 * Emplacement droit de la barre d'onglets contextuels.
 *
 * Les maquettes y placent, selon l'écran, un champ de recherche (Tableau de bord, Suivi,
 * Relations, Documents) ou une note de contexte (Analyses, Réglages). L'accessoire
 * appartient donc à l'écran, mais s'affiche dans la coque : un portail vers un nœud
 * exposé par `AppShell` évite de faire remonter l'état de recherche de chaque page
 * jusqu'à la coque, ou de dupliquer la barre dans chaque écran.
 */
const SlotContext = createContext<HTMLElement | null>(null);

/** Fournit le nœud d'accueil ; monté par `AppShell` autour de la barre et des écrans. */
export function ContextBarProvider({
  children,
}: {
  children: (slotRef: (node: HTMLElement | null) => void) => ReactNode;
}) {
  const [slot, setSlot] = useState<HTMLElement | null>(null);
  return <SlotContext value={slot}>{children(setSlot)}</SlotContext>;
}

/** Rend son contenu à droite de la barre d'onglets de la section courante. */
export function ContextBarAccessory({ children }: { children: ReactNode }) {
  const slot = useContext(SlotContext);
  if (!slot) return null;
  return createPortal(children, slot);
}

/**
 * Champ de recherche de la barre contextuelle.
 *
 * Géométrie des maquettes : 30 px de haut, fond page dans une barre en surface, raccourci
 * affiché en pastille monospace. Le raccourci est décoratif tant qu'aucune palette de
 * commandes n'existe : il est masqué aux lecteurs d'écran plutôt qu'annoncé comme une
 * action inexistante.
 */
export function ContextSearch({
  value,
  placeholder,
  onChange,
  width = 260,
}: {
  value: string;
  placeholder: string;
  onChange: (valeur: string) => void;
  /** Largeur minimale ; les maquettes vont de 230 à 260 px selon l'écran. */
  width?: number;
}) {
  return (
    <div
      className="flex h-tab items-center gap-2 rounded-button border border-line bg-page px-[11px] focus-within:border-accent"
      style={{ minWidth: width }}
    >
      <Icon name="search" size={16} className="flex-none text-ink-faint" />
      <input
        type="search"
        value={value}
        onChange={(event) => onChange(event.target.value)}
        placeholder={placeholder}
        aria-label={placeholder}
        className="min-w-0 flex-1 bg-transparent text-note text-ink outline-none placeholder:text-ink-faint"
      />
      <span
        aria-hidden="true"
        className="flex-none rounded-tag border border-line px-[5px] py-px font-mono text-micro text-ink-faint"
      >
        ⌘K
      </span>
    </div>
  );
}

/** Note de contexte affichée à droite de la barre, à la place d'une recherche. */
export function ContextNote({ children }: { children: ReactNode }) {
  return <p className="text-note text-ink-faint">{children}</p>;
}
