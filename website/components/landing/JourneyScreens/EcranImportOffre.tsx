import { Icon } from "@/components/ui/Icon";

import { EcranDeuxVolets, EtiquetteMono } from "./primitives";

const EXTRAITS = [
  ["Poste", "Designer produit"],
  ["Entreprise", "Atelier Nord"],
  ["Lieu", "Lyon"],
  ["Contrat", "CDI"],
  ["Publiée le", "24 juil. 2026"],
] as const;

/** Écran 01 — Import d'offre : champ URL à caret clignotant, texte de l'annonce
 *  et volet « Extrait automatiquement ». */
export function EcranImportOffre() {
  return (
    <EcranDeuxVolets
      gauche={
        <>
          <div className="flex h-[30px] items-center gap-2 rounded-[9px] border border-control px-[9px] font-mono text-[12px] text-ink-muted">
            <span className="text-ink-faint">
              <Icon name="link" size={15} />
            </span>
            emplois.exemple.fr/offre/4821
            <span aria-hidden="true" className="h-[14px] w-px animate-caret bg-accent" />
          </div>

          <div className="mt-[14px] text-[12px] leading-[1.7] text-ink-muted">
            <p className="mb-[6px] text-[13.5px] font-semibold text-ink">
              Designer produit — Atelier Nord
            </p>
            Lyon · CDI · Hybride 2 jours
            <br />
            Nous cherchons une personne capable de mener une fonctionnalité de la recherche
            utilisateur à la mise en production, en binôme avec l&apos;équipe technique. Vous
            animerez les revues de design et ferez vivre notre système de composants.
          </div>

          <div className="mt-[14px] flex gap-2">
            <span className="inline-flex h-[30px] shrink-0 items-center gap-[6px] whitespace-nowrap rounded-control border border-accent-strong bg-accent px-[11px] text-[12.5px] font-semibold text-on-accent">
              <Icon name="bookmark_add" size={15} />
              Enregistrer l&apos;offre
            </span>
            <span className="inline-flex h-[30px] items-center rounded-control border border-control bg-surface px-[11px] text-[12.5px] font-semibold text-ink">
              Annuler
            </span>
          </div>
        </>
      }
      droite={
        <>
          <EtiquetteMono>Extrait automatiquement</EtiquetteMono>
          <div className="mt-3 border-t border-line">
            {EXTRAITS.map(([libelle, valeur]) => (
              <div
                key={libelle}
                className="flex justify-between gap-3 border-b border-line py-2 text-[12px]"
              >
                <span className="text-ink-tertiary">{libelle}</span>
                <span className="font-semibold tabular-nums text-ink">{valeur}</span>
              </div>
            ))}
          </div>
          <p className="mt-3 text-[11.5px] leading-[1.6] text-ink-tertiary">
            Vous pouvez corriger chaque champ avant d&apos;enregistrer.
          </p>
        </>
      }
    />
  );
}
