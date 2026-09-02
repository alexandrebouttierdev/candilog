"use client";

import { useEffect, useRef, useState } from "react";

/** Le score du CV « Atelier Nord » après arbitrage des propositions, et le gain
 *  accumulé depuis la génération — les deux seules mesures que `ResumeAtsPanel`
 *  affiche. L'application ne calcule ni jauge de lisibilité, ni verdict global. */
const CIBLE = 84;
export const GAIN_CUMULE = 13;
const PAS = 3;
const PERIODE_MS = 26;

/**
 * Compte le score ATS à l'entrée dans le viewport (§7.5) : +3 toutes les 26 ms
 * jusqu'à 84, une seule fois, seuil d'intersection 0.35.
 *
 * Sous `prefers-reduced-motion`, la valeur est posée d'un coup à l'entrée dans le
 * viewport : pas de compteur qui tourne (§12).
 */
export function useAtsReveal<T extends HTMLElement>() {
  const ref = useRef<T>(null);
  const [score, setScore] = useState(0);

  useEffect(() => {
    const hote = ref.current;
    if (!hote) return;

    let compteur: ReturnType<typeof setInterval> | undefined;

    const io = new IntersectionObserver(
      (entrees) => {
        for (const entree of entrees) {
          if (!entree.isIntersecting) continue;
          io.disconnect();

          if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) {
            setScore(CIBLE);
            return;
          }

          let n = 0;
          compteur = setInterval(() => {
            n += PAS;
            if (n >= CIBLE) {
              n = CIBLE;
              clearInterval(compteur);
            }
            setScore(n);
          }, PERIODE_MS);
        }
      },
      { threshold: 0.35 },
    );

    io.observe(hote);
    return () => {
      io.disconnect();
      clearInterval(compteur);
    };
  }, []);

  return { ref, score };
}
