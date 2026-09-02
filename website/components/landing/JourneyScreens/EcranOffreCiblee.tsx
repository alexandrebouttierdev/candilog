import { Icon } from "@/components/ui/Icon";

import { EtiquetteMono } from "./primitives";

/** Écran 01 — Documents → Générer un CV, avant génération.
 *
 *  Trois panneaux comme dans l'application : l'offre collée à gauche, l'aperçu A4 au
 *  centre, l'analyse ATS à droite en attente. Il n'y a pas de champ URL : Candilog ne
 *  télécharge aucune annonce, il lit le texte que vous collez. */
export function EcranOffreCiblee() {
  return (
    <div className="grid min-h-[320px] grid-cols-[repeat(auto-fit,minmax(min(240px,100%),1fr))]">
      {/* ── Panneau « Offre ciblée » ────────────────────────────────────── */}
      <div className="min-w-0 border-r border-line-soft px-[18px] py-4">
        <div className="mb-3 flex items-center gap-[7px]">
          <span className="text-ink-faint">
            <Icon name="target" size={15} />
          </span>
          <span className="text-[12.5px] font-semibold text-ink">Offre ciblée</span>
        </div>

        <p className="mb-[5px] text-[11.5px] text-ink-muted">
          Texte de l&apos;offre
          <span className="text-danger"> *</span>
        </p>
        <div className="rounded-[9px] border border-accent px-[10px] py-[9px] text-[11.5px] leading-[1.65] text-ink-muted outline outline-1 outline-tint-border-strong">
          <span className="font-semibold text-ink">Designer produit — Atelier Nord</span>
          <br />
          Lyon · CDI · Hybride 2 jours
          <br />
          Nous cherchons une personne capable de mener une fonctionnalité de la recherche
          utilisateur à la mise en production, en binôme avec l&apos;équipe technique. Vous
          animerez les revues de design et ferez vivre notre système de composants.
          <span aria-hidden="true" className="ml-px inline-block h-[13px] w-px translate-y-[2px] animate-caret bg-accent" />
        </div>
        <p className="mt-[6px] text-[11px] leading-[1.5] text-ink-tertiary">
          Le texte est envoyé uniquement au fournisseur configuré.
        </p>

        <div className="mt-3 flex h-[30px] items-center justify-center gap-[6px] rounded-control border border-accent-strong bg-accent px-[11px] text-[12.5px] font-semibold text-on-accent">
          <Icon name="auto_awesome" size={15} />
          Générer le CV ciblé
        </div>
      </div>

      {/* ── Aperçu A4, vide tant que rien n'est généré ───────────────────── */}
      <div className="min-w-0 border-r border-line-soft bg-surface-alt px-[18px] py-4">
        <div className="mb-3 flex items-center gap-[7px]">
          <span className="text-ink-faint">
            <Icon name="article" size={15} />
          </span>
          <span className="text-[12.5px] font-semibold text-ink">Aperçu HTML · A4</span>
        </div>

        <div className="grid min-h-[210px] place-items-center rounded-tile border border-line bg-surface px-4 py-6 text-center">
          <div>
            <span className="mx-auto mb-[10px] grid size-[42px] place-items-center rounded-[12px] bg-page text-ink-faint">
              <Icon name="article" size={21} />
            </span>
            <p className="text-[12.5px] font-semibold text-ink">Aucun CV généré</p>
            <p className="mx-auto mt-[5px] max-w-[190px] text-[11.5px] leading-[1.55] text-ink-tertiary">
              L&apos;aperçu apparaîtra ici, à la géométrie exacte de la page exportée.
            </p>
          </div>
        </div>
      </div>

      {/* ── Analyse ATS, en attente ──────────────────────────────────────── */}
      <div className="min-w-0 bg-surface-alt px-[18px] py-4">
        <div className="mb-3 flex items-center gap-[7px]">
          <span className="text-ink-faint">
            <Icon name="query_stats" size={15} />
          </span>
          <span className="text-[12.5px] font-semibold text-ink">Analyse ATS</span>
        </div>

        <div className="grid min-h-[210px] place-items-center rounded-tile border border-line bg-surface px-4 py-6 text-center">
          <div>
            <span className="mx-auto mb-[10px] grid size-[42px] place-items-center rounded-[12px] bg-page text-ink-faint">
              <Icon name="query_stats" size={21} />
            </span>
            <p className="text-[12.5px] font-semibold text-ink">Analyse en attente</p>
            <p className="mx-auto mt-[5px] max-w-[190px] text-[11.5px] leading-[1.55] text-ink-tertiary">
              Le score et les recommandations suivront la génération.
            </p>
          </div>
        </div>

        <EtiquetteMono className="mt-4">Tout reste sur votre machine</EtiquetteMono>
      </div>
    </div>
  );
}
