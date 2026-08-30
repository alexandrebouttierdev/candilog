"use client";

import { useEffect, useState } from "react";

import { Icon } from "@/components/ui/Icon";
import { Reveal } from "@/components/ui/Reveal";
import { cn } from "@/lib/cn";
import {
  ATOUTS_SUIVI,
  BOUCLE_STATUT,
  CANDIDATURES,
  COLONNES_BOARD,
  LIBELLE_STATUT,
  PERIODE_BOUCLE_MS,
  STATUT,
  type CarteBoard,
  type CleStatut,
} from "@/lib/data/suivi";

const COLONNES_LISTE = "grid-cols-[1.4fr_1fr_0.8fr_0.8fr_0.7fr]";

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
        "inline-flex h-[19px] shrink-0 items-center whitespace-nowrap rounded-pill border px-[7px] text-[11px] font-semibold",
        STATUT[statut],
        className,
      )}
    >
      {LIBELLE_STATUT[statut]}
    </span>
  );
}

function CarteCandidature({
  carte,
  statutAnime,
  decalage,
}: {
  carte: CarteBoard;
  statutAnime: CleStatut;
  decalage: number;
}) {
  return (
    <div
      className={cn(
        "rounded-tile border bg-surface px-[11px] py-[10px]",
        carte.animee ? "border-tint-border-strong" : "border-line",
        carte.attenuee && "opacity-60",
        carte.animee && "transition-transform duration-[320ms] ease-out-soft",
      )}
      style={
        carte.animee
          ? { transform: `translateY(${String(decalage)}px)` }
          : undefined
      }
    >
      <p className="text-[12.5px] font-semibold text-ink">{carte.poste}</p>
      <p className="mt-[2px] text-[11.5px] text-ink-tertiary">
        {carte.entreprise}
      </p>

      {carte.animee ? (
        <div className="mt-2 flex items-center gap-[6px]">
          <Badge
            statut={statutAnime}
            className="transition-colors duration-[260ms]"
          />
          <span className="ml-auto text-[11px] tabular-nums text-ink-tertiary">
            {carte.detail}
          </span>
        </div>
      ) : carte.detail ? (
        <div className="mt-2 flex items-center gap-[6px]">
          {carte.icone ? (
            <span className="text-ink-faint">
              <Icon name={carte.icone} size={13} />
            </span>
          ) : null}
          <span className="text-[11px] tabular-nums text-ink-tertiary">
            {carte.detail}
          </span>
        </div>
      ) : null}
    </div>
  );
}

/**
 * Suivi (§7.6) : bascule board / liste et boucle d'animation de statut.
 *
 * La boucle est un `setInterval` borné qui s'arrête au démontage — pas une
 * animation CSS infinie — et elle ne démarre pas du tout sous
 * `prefers-reduced-motion` (§12).
 */
export function TrackingBoard({
  vueInitiale = "board",
  animationStatut = true,
}: {
  vueInitiale?: "board" | "liste";
  animationStatut?: boolean;
}) {
  const [vue, setVue] = useState<"board" | "liste">(vueInitiale);
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
  const decalage = etat.statut === "entretien" ? -2 : 0;

  const bascule = (cible: "board" | "liste") =>
    cn(
      "inline-flex h-[26px] cursor-pointer items-center rounded-[7px] px-3 text-[12.5px] font-semibold transition-colors duration-[160ms]",
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
                onClick={() => setVue("board")}
                aria-pressed={vue === "board"}
                className={bascule("board")}
              >
                Board
              </button>
              <button
                type="button"
                onClick={() => setVue("liste")}
                aria-pressed={vue === "liste"}
                className={bascule("liste")}
              >
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
                12 candidatures · 5 statuts
              </span>
              <span className="ml-auto font-mono text-[10.5px] text-ink-faint">
                {etat.note}
              </span>
            </div>

            {vue === "board" ? (
              /* grid-auto-flow column + overflow-x auto, colonnes de 190px (§6). */
              <div className="grid min-h-[400px] auto-cols-[minmax(190px,1fr)] grid-flow-col overflow-x-auto">
                {COLONNES_BOARD.map((colonne) => (
                  <div
                    key={colonne.statut}
                    className="min-w-0 border-r border-line-soft"
                  >
                    <div className="flex items-center gap-[7px] border-b border-line-soft px-3 py-[10px]">
                      <span
                        aria-hidden="true"
                        className={cn("size-[7px] rounded-[2px]", colonne.puce)}
                      />
                      <span className="text-[12px] font-semibold text-ink">
                        {LIBELLE_STATUT[colonne.statut]}
                      </span>
                      <span className="ml-auto text-[11px] tabular-nums text-ink-faint">
                        {colonne.total}
                      </span>
                    </div>
                    <div
                      className={cn(
                        "flex min-h-[340px] flex-col gap-2 p-[10px]",
                        colonne.fondCreuse && "bg-surface-sunken",
                      )}
                    >
                      {colonne.cartes.map((carte) => (
                        <CarteCandidature
                          key={carte.poste}
                          carte={carte}
                          statutAnime={etat.statut}
                          decalage={decalage}
                        />
                      ))}
                    </div>
                  </div>
                ))}
              </div>
            ) : (
              <div className="min-h-[400px] overflow-x-auto">
                <div className="min-w-[720px]">
                  <div
                    className={cn(
                      "grid gap-3 border-b border-line-soft bg-surface-alt px-4 py-[9px] font-mono text-[10.5px] font-semibold uppercase tracking-[0.07em] text-ink-faint",
                      COLONNES_LISTE,
                    )}
                  >
                    <span>Poste</span>
                    <span>Entreprise</span>
                    <span>Statut</span>
                    <span>Prochaine étape</span>
                    <span className="text-right">Envoyée</span>
                  </div>
                  {CANDIDATURES.map((ligne) => (
                    <div
                      key={ligne.poste}
                      className={cn(
                        "grid items-center gap-3 border-b border-line-soft px-4 py-[11px] transition-colors duration-[120ms] hover:bg-surface-alt",
                        COLONNES_LISTE,
                      )}
                    >
                      <div className="flex min-w-0 items-center gap-[9px]">
                        <span className="grid size-[22px] shrink-0 place-items-center rounded-[7px] bg-page text-[10.5px] font-semibold text-ink-muted">
                          {ligne.initiales}
                        </span>
                        <span className="truncate text-[13px] font-semibold text-ink">
                          {ligne.poste}
                        </span>
                      </div>
                      <span className="truncate text-[12.5px] text-ink-muted">
                        {ligne.entreprise}
                      </span>
                      <span>
                        <Badge statut={ligne.statut} />
                      </span>
                      <span className="text-[12px] tabular-nums text-ink-muted">
                        {ligne.etape}
                      </span>
                      <span className="text-right text-[12px] tabular-nums text-ink-tertiary">
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
