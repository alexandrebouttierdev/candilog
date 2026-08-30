"use client";

import type { ReactNode } from "react";

import { useScrollReveal } from "@/lib/hooks/useScrollReveal";

/**
 * Conteneur de section qui révèle ses enfants directs au scroll : montée de 16px
 * + fondu, décalage de 80 ms entre enfants (§5).
 *
 * Le prototype applique la même règle — pour chaque `<section>`, les enfants de son
 * conteneur. Encapsuler le hook ici évite de faire basculer les sections entières
 * en composants client : seul ce `<div>` l'est.
 *
 * Le rendu serveur sort visible ; les classes `.reveal` ne sont posées qu'après
 * hydratation. Si le JS ne s'exécute pas, tout reste lisible.
 */
export function Reveal({
  children,
  className,
  decalage,
}: {
  children: ReactNode;
  className?: string;
  decalage?: number;
}) {
  const ref = useScrollReveal<HTMLDivElement>(decalage);
  return (
    <div ref={ref} className={className}>
      {children}
    </div>
  );
}
