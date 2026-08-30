import type { ReactNode } from "react";

import { cn } from "@/lib/cn";

/** Chapô de page : puce indigo + sur-titre, H1, paragraphe d'introduction et,
 *  quand la page la porte, la date de dernière mise à jour. */
export function LegalHero({
  surTitre,
  titre,
  chapo,
  miseAJour,
}: {
  surTitre: string;
  titre: string;
  chapo: string;
  miseAJour?: string;
}) {
  return (
    <div className="max-w-full">
      <div className="mb-[18px] flex items-center gap-[9px]">
        <span aria-hidden="true" className="size-[6px] rounded-full bg-accent" />
        <span className="text-[12.5px] font-semibold text-ink-tertiary">{surTitre}</span>
      </div>
      <h1 className="text-balance text-[clamp(28px,3.2vw,40px)] font-semibold leading-[1.12] tracking-[-0.022em] text-ink">
        {titre}
      </h1>
      <p className="mt-[18px] text-pretty text-[16px] leading-[1.7] text-ink-muted">{chapo}</p>
      {miseAJour ? (
        <div className="mt-[22px] border-t border-line pt-[18px] font-mono text-[11.5px] text-ink-faint">
          Dernière mise à jour : {miseAJour}
        </div>
      ) : null}
    </div>
  );
}

/** Sommaire ancré (§7.13) — présent sur Confidentialité et Conditions. Les cibles
 *  ont `scroll-margin-top: 84px` via la règle `section[id]` de globals.css. */
export function Sommaire({ entrees }: { entrees: ReadonlyArray<{ href: string; libelle: string }> }) {
  return (
    <nav
      aria-label="Sommaire"
      className="mt-[clamp(32px,4vw,48px)] max-w-full rounded-card border border-control bg-surface px-5 py-[18px]"
    >
      <p className="mb-3 text-[12.5px] text-ink-faint">Sommaire</p>
      <div className="flex flex-col gap-2">
        {entrees.map(({ href, libelle }) => (
          <a
            key={href}
            href={href}
            className="text-[13.5px] text-ink hover:text-accent-text"
          >
            {libelle}
          </a>
        ))}
      </div>
    </nav>
  );
}

/** Corps de page : conteneur des sections, décalé du chapô. */
export function LegalCorps({ children }: { children: ReactNode }) {
  return <div className="mt-[clamp(36px,4vw,56px)] max-w-full">{children}</div>;
}

/** Section de contenu. La première n'a pas de filet de tête ; les suivantes sont
 *  séparées par `margin-top: 44px; padding-top: 34px` + filet (§7.13). */
export function LegalSection({
  id,
  titre,
  premiere = false,
  children,
}: {
  id?: string;
  titre: string;
  premiere?: boolean;
  children: ReactNode;
}) {
  return (
    <section
      id={id}
      className={cn(!premiere && "mt-[44px] border-t border-line pt-[34px]")}
    >
      <h2 className="text-[20px] font-semibold leading-[1.3] tracking-[-0.014em] text-ink">
        {titre}
      </h2>
      {children}
    </section>
  );
}

/** Paragraphe de corps : 16px sous le titre, 14px entre paragraphes. */
export function P({ children }: { children: ReactNode }) {
  return (
    <p className="mt-[14px] text-pretty text-[15px] leading-[1.75] text-ink-body first-of-type:mt-4">
      {children}
    </p>
  );
}

/** Liste de lignes clé/valeur à filets (mentions légales, encadré licence). */
export function ListeDefinitions({
  children,
  encadree = false,
}: {
  children: ReactNode;
  encadree?: boolean;
}) {
  return (
    <div
      className={cn(
        encadree
          ? "mt-[clamp(32px,4vw,48px)] max-w-full overflow-hidden rounded-card border border-control bg-surface"
          : "mt-4 border-t border-line",
      )}
    >
      {children}
    </div>
  );
}

export function LigneDefinition({
  libelle,
  children,
  encadree = false,
  derniere = false,
}: {
  libelle: string;
  children: ReactNode;
  encadree?: boolean;
  derniere?: boolean;
}) {
  return (
    <div
      className={cn(
        "flex flex-wrap gap-x-5 gap-y-2",
        encadree ? "px-[18px] py-[14px]" : "py-3",
        encadree
          ? !derniere && "border-b border-line-soft"
          : "border-b border-line",
      )}
    >
      <span
        className={cn(
          "text-ink-tertiary",
          encadree ? "min-w-[170px] text-[13.5px]" : "min-w-[190px] text-[14px]",
        )}
      >
        {libelle}
      </span>
      <span className={cn("font-semibold text-ink", encadree ? "text-[13.5px]" : "text-[14px]")}>
        {children}
      </span>
    </div>
  );
}

/** Valeur en pastille mono — l'identifiant SPDX de la page Licence. */
export function ValeurMono({ children }: { children: ReactNode }) {
  return (
    <span className="rounded-pill border border-line bg-surface-alt px-2 py-[2px] font-mono text-[12.5px] font-normal text-ink-body">
      {children}
    </span>
  );
}

/** Ligne « Contact » en fin de section (confidentialité, conditions). */
export function LigneContact({ email }: { email: string }) {
  return (
    <div className="mt-4 flex flex-wrap items-center gap-[10px]">
      <span className="text-[13.5px] text-ink-tertiary">Contact</span>
      <a href={`mailto:${email}`} className="text-[14px] font-semibold">
        {email}
      </a>
    </div>
  );
}

/** Carte à filet : titre discret puis contenu (encart de contact commercial). */
export function CarteInfo({ titre, children }: { titre: string; children: ReactNode }) {
  return (
    <div className="mt-5 rounded-card border border-control bg-surface px-[18px] py-4">
      <p className="text-[12.5px] text-ink-faint">{titre}</p>
      {children}
    </div>
  );
}

/** Tableau à filets avec en-tête — la liste des fournisseurs IA (§7.13). */
export function TableauFournisseurs({
  entete,
  lignes,
  note,
}: {
  entete: string;
  lignes: ReadonlyArray<{ nom: string; detail: string }>;
  note: string;
}) {
  return (
    <div className="mt-[22px] overflow-hidden rounded-card border border-control bg-surface">
      <div className="flex items-center gap-[10px] border-b border-line-soft bg-surface-alt px-4 py-3">
        <span className="text-[13px] font-semibold text-ink">{entete}</span>
      </div>
      {lignes.map(({ nom, detail }) => (
        <div
          key={nom}
          className="flex flex-wrap gap-x-4 gap-y-[10px] border-b border-line-soft px-4 py-[14px]"
        >
          <span className="min-w-[160px] text-[13.5px] font-semibold text-ink">{nom}</span>
          <span className="min-w-[200px] flex-1 text-[13px] text-ink-muted">{detail}</span>
        </div>
      ))}
      <p className="px-4 py-[14px] text-[13px] leading-[1.7] text-ink-muted">{note}</p>
    </div>
  );
}
