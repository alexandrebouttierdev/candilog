"use client";

import { useEffect, useState } from "react";

import { Icon } from "@/components/ui/Icon";
import { Reveal } from "@/components/ui/Reveal";
import { cn } from "@/lib/cn";
import {
  ATOUTS_SUIVI,
  BOUCLE_STATUT,
  CANDIDATURES,
  CARTE_ANIMEE,
  COLONNES_BOARD,
  ICONE_STATUT,
  LIBELLE_STATUT,
  PERIODE_BOUCLE_MS,
  PUCE_STATUT,
  STATUT,
  type CarteBoard,
  type CleStatut,
} from "@/lib/data/suivi";

/* Huit colonnes, comme `ApplicationsPage` : poste, entreprise, ville, contrat, durée,
   type, statut, date d'envoi. La table défile horizontalement sous 940px plutôt que
   d'écraser ses colonnes. */
const COLONNES_LISTE = "grid-cols-[2.2fr_1.3fr_0.9fr_0.7fr_1fr_0.8fr_1.1fr_0.8fr]";

function Badge({
  statut,
  className,
}: {
  statut: CleStatut;
  className?: string;
}) {
  return (
    <span
      className={cn(
        "inline-flex h-[19px] shrink-0 items-center gap-[4px] whitespace-nowrap rounded-pill border px-[7px] text-[11px] font-semibold",
        STATUT[statut],
        className,
      )}
    >
      <Icon name={ICONE_STATUT[statut]} size={12} />
      {LIBELLE_STATUT[statut]}
    </span>
  );
}

/** Carte du Kanban — même composition que `ApplicationCard` : pastille d'initiales,
 *  intitulé, entreprise, puis contrat, ville et ancienneté en jours. */
function CarteCandidature({
  carte,
  deplacee = false,
}: {
  carte: CarteBoard;
  /** La carte que la boucle vient de déposer dans cette colonne. */
  deplacee?: boolean;
}) {
  return (
    <div
      className={cn(
        "min-w-0 rounded-tile border bg-surface px-3 py-[10px]",
        deplacee
          ? "border-accent-strong animate-depose"
          : "border-line",
      )}
    >
      <div className="mb-[9px] flex items-start gap-[9px]">
        <span className="grid size-[26px] flex-none place-items-center rounded-control bg-page text-[10.5px] font-semibold text-ink-muted">
          {carte.initiales}
        </span>
        <div className="min-w-0 flex-1">
          <p className="text-[12.5px] font-semibold leading-[1.35] text-ink">
            {carte.poste}
          </p>
          <p className="mt-[2px] truncate text-[11.5px] text-ink-faint">
            {carte.entreprise}
          </p>
        </div>
      </div>

      <div className="flex flex-wrap items-center gap-[6px]">
        <span className="inline-flex h-[18px] items-center rounded-[6px] border border-line bg-surface-alt px-[6px] text-[10.5px] text-ink-muted">
          {carte.contrat}
        </span>
        <span className="truncate text-[10.5px] text-ink-faint">{carte.ville}</span>
        <span className="flex-1" />
        <span
          className={cn(
            "inline-flex flex-none items-center gap-1 text-[10.5px] tabular-nums",
            carte.jours >= 15 ? "text-warning" : "text-ink-faint",
          )}
        >
          <Icon name={carte.jours >= 15 ? "schedule" : "event"} size={13} />
          {carte.jours} j
        </span>
      </div>
    </div>
  );
}

/**
 * Suivi (§7.6) : bascule Kanban / Liste et boucle d'animation de statut.
 *
 * La boucle est un `setInterval` borné qui s'arrête au démontage — pas une
 * animation CSS infinie — et elle ne démarre pas du tout sous
 * `prefers-reduced-motion` (§12).
 */
export function TrackingBoard({
  vueInitiale = "kanban",
  animationStatut = true,
}: {
  vueInitiale?: "kanban" | "liste";
  animationStatut?: boolean;
}) {
  const [vue, setVue] = useState<"kanban" | "liste">(vueInitiale);
  const [index, setIndex] = useState(0);

  useEffect(() => {
    if (!animationStatut) return;
    if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) return;

    const boucle = setInterval(() => {
      setIndex((precedent) => (precedent + 1) % BOUCLE_STATUT.length);
    }, PERIODE_BOUCLE_MS);
    return () => {
      clearInterval(boucle);
    };
  }, [animationStatut]);

  const etat = BOUCLE_STATUT[index] ?? BOUCLE_STATUT[0];
  if (!etat) return null;

  const bascule = (cible: "kanban" | "liste") =>
    cn(
      "inline-flex h-[26px] cursor-pointer items-center gap-[5px] rounded-[7px] px-[10px] text-[12.5px] font-semibold transition-colors duration-[160ms]",
      vue === cible
        ? "bg-tint-12 text-accent-text"
        : "bg-transparent text-ink-muted",
    );

  return (
    <section
      id="suivi"
      className="overflow-hidden border-b border-line bg-page"
    >
      <Reveal className="mx-auto max-w-[1240px] px-[clamp(16px,4vw,40px)] pt-[clamp(48px,6vw,88px)]">
        <div className="mb-[30px] flex flex-wrap items-end gap-6">
          <h2 className="max-w-[520px] text-[clamp(24px,2.6vw,34px)] font-semibold leading-[1.14] tracking-[-0.02em] text-ink">
            Où en est chaque candidature,
            <br />
            sans avoir à s&apos;en souvenir.
          </h2>

          <div className="mb-1 ml-auto flex items-center gap-[10px]">
            <span className="text-[12.5px] text-ink-muted" id="libelle-vue">
              Vue
            </span>
            {/* Le prototype utilisait des <div onClick> : ici de vrais boutons
                dans un groupe nommé, avec aria-pressed sur chaque option (§9). */}
            <div
              role="group"
              aria-labelledby="libelle-vue"
              className="flex gap-[2px] rounded-[9px] border border-control bg-surface p-[2px]"
            >
              <button
                type="button"
                onClick={() => setVue("kanban")}
                aria-pressed={vue === "kanban"}
                className={bascule("kanban")}
              >
                <Icon name="view_kanban" size={14} />
                Kanban
              </button>
              <button
                type="button"
                onClick={() => setVue("liste")}
                aria-pressed={vue === "liste"}
                className={bascule("liste")}
              >
                <Icon name="view_list" size={14} />
                Liste
              </button>
            </div>
          </div>
        </div>
      </Reveal>

      <div className="pb-[clamp(48px,6vw,88px)]">
        <div className="mx-auto max-w-[1240px] px-[clamp(16px,4vw,40px)]">
          <div className="overflow-hidden rounded-card border border-control bg-surface">
            <div className="flex h-10 items-center gap-[10px] border-b border-line-soft bg-surface-alt px-[14px]">
              <span className="text-[12.5px] font-semibold text-ink">
                Candidatures
              </span>
              <span className="text-[11.5px] text-ink-faint">
                14 candidatures · 4 statuts
              </span>
              <span className="ml-auto font-mono text-[10.5px] text-ink-faint">
                {etat.note}
              </span>
            </div>

            {vue === "kanban" ? (
              /* Quatre colonnes auto-ajustées d'au moins 240px, sur `surface-alt` et
                 encadrées — la géométrie de `KanbanBoard` (§7.6). */
              <div className="grid min-h-[400px] grid-cols-[repeat(auto-fit,minmax(min(240px,100%),1fr))] gap-[14px] bg-page p-[14px]">
                {COLONNES_BOARD.map((colonne) => {
                  const accueilleCarte = etat.statut === colonne.statut;
                  const total = colonne.cartes.length + (accueilleCarte ? 1 : 0);

                  return (
                  <section
                    key={colonne.statut}
                    className="flex min-w-0 flex-col rounded-card border border-line bg-surface-alt"
                  >
                    <header className="flex flex-none items-center gap-2 border-b border-line px-[14px] py-3">
                      <span
                        aria-hidden="true"
                        className={cn(
                          "size-[7px] flex-none rounded-full",
                          PUCE_STATUT[colonne.statut],
                        )}
                      />
                      <h3 className="min-w-0 truncate text-[12.5px] font-semibold text-ink">
                        {LIBELLE_STATUT[colonne.statut]}
                      </h3>
                      <span className="flex-none rounded-[6px] border border-control bg-surface px-[6px] py-px text-[11px] font-semibold tabular-nums text-ink">
                        {total}
                      </span>
                      <span className="flex-1" />
                      <span className="grid size-[26px] place-items-center rounded-control text-ink-faint">
                        <Icon name="add" size={16} />
                      </span>
                    </header>

                    <div className="flex flex-1 flex-col gap-2 p-[10px]">
                      {/* La carte animée n'apparaît que dans la colonne du statut
                          courant : c'est le geste de glisser-déposer de l'application,
                          pas une pastille qui change de couleur. */}
                      {accueilleCarte ? (
                        <CarteCandidature carte={CARTE_ANIMEE} deplacee />
                      ) : null}
                      {colonne.cartes.map((carte) => (
                        <CarteCandidature
                          key={`${carte.entreprise}-${carte.poste}`}
                          carte={carte}
                        />
                      ))}
                    </div>
                  </section>
                  );
                })}
              </div>
            ) : (
              <div className="min-h-[400px] overflow-x-auto">
                <div className="min-w-[940px]">
                  <div
                    className={cn(
                      "grid gap-3 border-b border-line-soft bg-surface-alt px-4 py-[9px] font-mono text-[10.5px] font-semibold uppercase tracking-[0.07em] text-ink-faint",
                      COLONNES_LISTE,
                    )}
                  >
                    <span>Poste</span>
                    <span>Entreprise</span>
                    <span>Ville</span>
                    <span>Contrat</span>
                    <span>Durée</span>
                    <span>Type</span>
                    <span>Statut</span>
                    <span className="text-right">Envoyée</span>
                  </div>
                  {CANDIDATURES.map((ligne) => (
                    <div
                      key={`${ligne.entreprise}-${ligne.poste}`}
                      className={cn(
                        "grid items-center gap-3 border-b border-line-soft px-4 py-[11px] transition-colors duration-[120ms] hover:bg-surface-alt",
                        COLONNES_LISTE,
                      )}
                    >
                      <div className="flex min-w-0 items-center gap-[9px]">
                        <span className="grid size-[26px] shrink-0 place-items-center rounded-control bg-page text-[10.5px] font-semibold text-ink-muted">
                          {ligne.initiales}
                        </span>
                        <span className="min-w-0">
                          <span className="block truncate text-[13px] font-semibold text-ink">
                            {ligne.poste}
                          </span>
                          <span className="block truncate text-[11px] text-ink-faint">
                            {ligne.domaine}
                          </span>
                        </span>
                      </div>
                      <span className="truncate text-[12.5px] text-ink-muted">
                        {ligne.entreprise}
                      </span>
                      <span className="truncate text-[11.5px] text-ink-faint">
                        {ligne.ville}
                      </span>
                      <span className="text-[11.5px] text-ink-faint">
                        {ligne.contrat}
                      </span>
                      <span className="truncate text-[11.5px] text-ink-faint">
                        {ligne.duree}
                      </span>
                      <span className="truncate text-[11.5px] text-ink-faint">
                        {ligne.type}
                      </span>
                      <span>
                        <Badge statut={ligne.statut} />
                      </span>
                      <span className="text-right text-[11.5px] tabular-nums text-ink-faint">
                        {ligne.date}
                      </span>
                    </div>
                  ))}
                </div>
              </div>
            )}
          </div>

          <div className="mt-9 grid grid-cols-[repeat(auto-fit,minmax(min(240px,100%),1fr))] gap-3">
            {ATOUTS_SUIVI.map((atout) => (
              <div
                key={atout.titre}
                className="rounded-card border border-line bg-surface p-[18px] transition-colors duration-[120ms] hover:border-control-strong"
              >
                <span
                  className={cn(
                    "grid size-8 place-items-center rounded-tile",
                    atout.accentue
                      ? "bg-tint-10 text-accent-text"
                      : "bg-page text-ink-muted",
                  )}
                >
                  <Icon name={atout.icone} size={19} />
                </span>
                <p className="mt-[14px] text-[13.5px] font-semibold text-ink">
                  {atout.titre}
                </p>
                <p className="mt-[6px] text-[12.5px] leading-[1.65] text-ink-tertiary">
                  {atout.texte}
                </p>
              </div>
            ))}
          </div>
        </div>
      </div>
    </section>
  );
}
