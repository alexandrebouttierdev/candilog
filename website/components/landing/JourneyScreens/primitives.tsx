import type { ReactNode } from "react";

export { EtiquetteMono } from "@/components/ui/EtiquetteMono";

import { cn } from "@/lib/cn";

/** Gabarit à deux volets des écrans 01, 02, 03 et 05 : contenu à gauche sur
 *  `--surface`, volet secondaire à droite sur `--surface-alt`. */
export function EcranDeuxVolets({
  gauche,
  droite,
  colonneMin = 260,
}: {
  gauche: ReactNode;
  droite: ReactNode;
  colonneMin?: 260 | 280;
}) {
  return (
    <div
      className={cn(
        "grid min-h-[320px]",
        colonneMin === 280
          ? "grid-cols-[repeat(auto-fit,minmax(min(280px,100%),1fr))]"
          : "grid-cols-[repeat(auto-fit,minmax(min(260px,100%),1fr))]",
      )}
    >
      <div className="min-w-0 border-r border-line-soft px-[18px] py-4">{gauche}</div>
      <div className="min-w-0 bg-surface-alt px-[18px] py-4">{droite}</div>
    </div>
  );
}

/** Surlignage indigo des trois ajouts de l'aperçu de CV (écran 03). */
export function Surlignage({ children }: { children: ReactNode }) {
  return (
    <span className="rounded-[4px] bg-tint-12 px-[3px] py-px text-accent-text">{children}</span>
  );
}

/** Faux champ de formulaire (écran 04) — décoratif, jamais un vrai input.
 *
 *  `aide` rend la ligne d'assistance de `FormField` : dans la modale de candidature, c'est
 *  elle qui annonce la valeur héritée de l'entreprise. `attenue` marque une valeur héritée
 *  plutôt que saisie. */
export function Champ({
  libelle,
  obligatoire = false,
  focus = false,
  mono = false,
  attenue = false,
  aide,
  children,
}: {
  libelle: string;
  obligatoire?: boolean;
  focus?: boolean;
  mono?: boolean;
  attenue?: boolean;
  aide?: string;
  children: ReactNode;
}) {
  return (
    <div className="min-w-0">
      <p className="mb-[5px] text-[11.5px] text-ink-muted">
        {libelle}
        {obligatoire ? <span className="text-danger"> *</span> : null}
      </p>
      <div
        className={cn(
          "flex h-[30px] items-center gap-[7px] rounded-[9px] border px-[9px] text-[12.5px]",
          focus ? "border-accent outline outline-1 outline-tint-border-strong" : "border-control",
          attenue ? "text-ink-tertiary" : "text-ink",
          mono && "font-mono",
        )}
      >
        {children}
      </div>
      {aide ? <p className="mt-[4px] text-[10.5px] text-ink-tertiary">{aide}</p> : null}
    </div>
  );
}
