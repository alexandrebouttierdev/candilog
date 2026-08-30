"use client";

import { useEffect, useRef, useState } from "react";

const CIBLE = { score: 72, barres: [64, 78, 42] as const };
const PAS = 3;
const PERIODE_MS = 26;

type Valeurs = { score: number; barres: readonly [number, number, number]; anime: boolean };

const DEPART: Valeurs = { score: 0, barres: [0, 0, 0], anime: true };

/**
 * Compte le score ATS à l'entrée dans le viewport (§7.5) : +3 toutes les 26 ms
 * jusqu'à 72, une seule fois, seuil d'intersection 0.35. Les barres reçoivent leur
 * valeur cible dès le déclenchement — c'est la transition CSS de 900 ms qui les
 * anime, pas ce compteur.
 *
 * Sous `prefers-reduced-motion`, les valeurs sont posées d'un coup à l'entrée dans
 * le viewport et `anime` repasse à faux pour que l'appelant retire la transition :
 * ni compteur qui tourne, ni barre qui glisse (§12).
 */
export function useAtsReveal<T extends HTMLElement>() {
  const ref = useRef<T>(null);
  const [valeurs, setValeurs] = useState<Valeurs>(DEPART);

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
            setValeurs({ score: CIBLE.score, barres: CIBLE.barres, anime: false });
            return;
          }

          setValeurs({ score: 0, barres: CIBLE.barres, anime: true });
          let n = 0;
          compteur = setInterval(() => {
            n += PAS;
            if (n >= CIBLE.score) {
              n = CIBLE.score;
              clearInterval(compteur);
            }
            setValeurs({ score: n, barres: CIBLE.barres, anime: true });
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

  return { ref, ...valeurs };
}
