import type { ReactNode } from "react";
import { createContext, useContext, useLayoutEffect, useState } from "react";

/** Indication de touche de la barre d'état : « Statut S », « Actions ⌘K ». */
export interface StatusKey {
  readonly label: string;
  /** Notation neutre (`s`, `mod+k`, `enter`). */
  readonly shortcut: string;
  /** Pastille accentuée (la dernière, « Actions »). */
  readonly accent?: boolean;
}

/**
 * Ce qu'un écran écrit dans le chrome de la fenêtre : fil d'Ariane de la barre de titre,
 * mention à droite, décompte et contrat clavier de la barre d'état. Le contrat clavier du
 * contexte courant est **toujours** imprimé en bas à droite (`INTERACTIONS.md` §6).
 */
export interface Chrome {
  readonly crumb?: string;
  readonly aside?: string;
  readonly status?: string;
  readonly keys?: readonly StatusKey[];
}

const ChromeValue = createContext<Chrome>({});
const ChromeSetter = createContext<(chrome: Chrome) => void>(() => undefined);

export function ChromeProvider({ children }: { children: ReactNode }) {
  const [chrome, setChrome] = useState<Chrome>({});
  return (
    <ChromeSetter value={setChrome}>
      <ChromeValue value={chrome}>{children}</ChromeValue>
    </ChromeSetter>
  );
}

export function useChromeValue(): Chrome {
  return useContext(ChromeValue);
}

/**
 * Déclare le chrome de l'écran courant ; il est effacé quand l'écran se démonte.
 *
 * Les dépendances sont les valeurs sérialisées : un objet recréé à chaque rendu ne doit pas
 * relancer l'effet, sans quoi écran et coque se renverraient la balle indéfiniment.
 */
export function useChrome(chrome: Chrome) {
  const set = useContext(ChromeSetter);
  const key = JSON.stringify(chrome);
  useLayoutEffect(() => {
    set(JSON.parse(key) as Chrome);
    return () => set({});
  }, [key, set]);
}
