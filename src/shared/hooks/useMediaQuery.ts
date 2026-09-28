import { useSyncExternalStore } from "react";

/**
 * La requête média correspond-elle ? Suivi en direct du redimensionnement.
 *
 * Réservé à ce que le CSS ne sait pas exprimer seul — l'inspecteur qui devient un panneau
 * flottant **ouvrable** sous 1060 px change de comportement, pas seulement d'apparence.
 * Sans `matchMedia` (tests), la fenêtre est considérée comme large.
 */
export function useMediaQuery(query: string): boolean {
  return useSyncExternalStore(
    (onChange) => {
      if (typeof window === "undefined" || typeof window.matchMedia !== "function") return () => undefined;
      const media = window.matchMedia(query);
      media.addEventListener("change", onChange);
      return () => media.removeEventListener("change", onChange);
    },
    () =>
      typeof window === "undefined" || typeof window.matchMedia !== "function"
        ? true
        : window.matchMedia(query).matches,
  );
}

/** Palier unique de la fenêtre (`DESIGN_SYSTEM.md` §9.3). */
export const WIDE_QUERY = "(min-width: 1060px)";
