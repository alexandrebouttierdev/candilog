import type { ReactNode } from "react";

import { cn } from "@/lib/cn";

/**
 * En-tête commun des sections : sur-titre, H2 en Plex Serif, chapô.
 *
 * Dès 1024 px, titre à gauche et chapô à droite, alignés sur leur ligne de base basse :
 * une composition éditoriale plutôt qu'un bloc centré.
 */
export function EnTeteSection({
  id,
  surTitre,
  titre,
  children,
  empile = false,
}: {
  id: string;
  surTitre: string;
  titre: ReactNode;
  children?: ReactNode;
  /** Titre et chapô l'un sous l'autre à toutes les largeurs. */
  empile?: boolean;
}) {
  return (
    <div className={cn("grid gap-5", !empile && "lg:grid-cols-[minmax(0,1fr)_440px] lg:items-end lg:gap-20")}>
      <div className="flex flex-col gap-4">
        <span className="eyebrow">{surTitre}</span>
        <h2
          id={id}
          className="serif-title text-balance text-[32px] leading-[1.1] tracking-[-0.025em] md:text-[40px] xl:text-[44px] xl:leading-[1.08]"
        >
          {titre}
        </h2>
      </div>
      {children ? <p className="text-pretty text-[16px] leading-[1.65] text-tx-3">{children}</p> : null}
    </div>
  );
}
