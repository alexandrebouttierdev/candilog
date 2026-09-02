"use client";

import { useId, useRef, useState } from "react";

import { EcranAnalyse } from "@/components/landing/JourneyScreens/EcranAnalyse";
import { EcranCalendrier } from "@/components/landing/JourneyScreens/EcranCalendrier";
import { EcranCandidature } from "@/components/landing/JourneyScreens/EcranCandidature";
import { EcranDocuments } from "@/components/landing/JourneyScreens/EcranDocuments";
import { EcranOffreCiblee } from "@/components/landing/JourneyScreens/EcranOffreCiblee";
import { Icon } from "@/components/ui/Icon";
import { Reveal } from "@/components/ui/Reveal";
import { cn } from "@/lib/cn";
import { ETAPES } from "@/lib/data/parcours";

const ECRANS = [
  EcranOffreCiblee,
  EcranAnalyse,
  EcranDocuments,
  EcranCandidature,
  EcranCalendrier,
] as const;

/**
 * Frise des 5 étapes (§7.4).
 *
 * Le prototype utilisait des `<div onClick>` ; ici c'est le vrai motif onglets —
 * `role="tablist"`, `aria-selected`, tabindex mobile et navigation aux flèches,
 * ce que le motif ARIA impose dès qu'on annonce des onglets (§9).
 */
export function JourneyTabs({ etapeInitiale = 0 }: { etapeInitiale?: number }) {
  const [etape, setEtape] = useState(etapeInitiale);
  const id = useId();
  const onglets = useRef<Array<HTMLButtonElement | null>>([]);

  const active = ETAPES[etape] ?? ETAPES[0];
  const Ecran = ECRANS[etape] ?? ECRANS[0];
  if (!active) return null;

  const aller = (index: number) => {
    const suivant = (index + ETAPES.length) % ETAPES.length;
    setEtape(suivant);
    onglets.current[suivant]?.focus();
  };

  const surTouche = (event: React.KeyboardEvent) => {
    const deplacements: Record<string, number> = {
      ArrowRight: etape + 1,
      ArrowLeft: etape - 1,
      Home: 0,
      End: ETAPES.length - 1,
    };
    const cible = deplacements[event.key];
    if (cible === undefined) return;
    event.preventDefault();
    aller(cible);
  };

  return (
    <section id="parcours" className="border-b border-line bg-surface">
      <Reveal className="mx-auto max-w-[1240px] px-[clamp(16px,4vw,40px)] py-[clamp(48px,6vw,88px)]">
        <div className="mb-[34px] flex flex-wrap items-end gap-6">
          <h2 className="max-w-[520px] text-[clamp(24px,2.6vw,34px)] font-semibold leading-[1.14] tracking-[-0.02em] text-ink">
            Une offre trouvée ce matin,
            <br />
            une candidature suivie ce soir.
          </h2>
        </div>

        {/* grid-auto-flow: column + overflow-x auto — colonnes de 158px mini (§6). */}
        <div
          role="tablist"
          aria-label="Étapes du parcours"
          onKeyDown={surTouche}
          className="grid auto-cols-[minmax(158px,1fr)] grid-flow-col overflow-x-auto border-y border-line"
        >
          {ETAPES.map((item, index) => {
            const selectionne = index === etape;
            return (
              <button
                key={item.titre}
                ref={(node) => {
                  onglets.current[index] = node;
                }}
                type="button"
                role="tab"
                id={`${id}-onglet-${String(index)}`}
                aria-selected={selectionne}
                aria-controls={`${id}-panneau`}
                tabIndex={selectionne ? 0 : -1}
                onClick={() => setEtape(index)}
                className={cn(
                  "relative cursor-pointer py-[18px] text-left transition-colors duration-[120ms] hover:bg-surface-alt",
                  index === 0 ? "pl-0 pr-4" : "border-l border-line px-4",
                )}
              >
                {selectionne ? (
                  <span
                    aria-hidden="true"
                    className={cn(
                      "absolute -top-px left-0 h-[2px] bg-accent",
                      index === 0 ? "right-px" : "right-0",
                    )}
                  />
                ) : null}

                <span className="flex items-center gap-[9px]">
                  <span
                    className={cn(
                      "grid size-[30px] place-items-center rounded-[9px] transition-colors duration-[160ms]",
                      selectionne
                        ? "bg-tint-12 text-accent-text"
                        : "bg-page text-ink-faint",
                    )}
                  >
                    <Icon name={item.icone} size={19} />
                  </span>
                  <span
                    className={cn(
                      "ml-auto font-mono text-[19px] font-semibold leading-none tracking-[-0.02em] transition-colors duration-[160ms]",
                      selectionne ? "text-accent-text" : "text-control-strong",
                    )}
                  >
                    {String(index + 1).padStart(2, "0")}
                  </span>
                </span>

                <span className="mt-3 block text-[13px] font-semibold text-ink">
                  {item.titre}
                </span>
                <span className="mt-[3px] block text-[12px] text-ink-tertiary">
                  {item.sousTitre}
                </span>
              </button>
            );
          })}
        </div>

        <div className="mt-7 grid grid-cols-[repeat(auto-fit,minmax(min(300px,100%),1fr))] items-start gap-[clamp(20px,3vw,40px)]">
          <div className="order-2 max-w-[340px]">
            <div className="border-t border-control pt-5">
              <h3 className="text-pretty text-[20px] font-semibold leading-[1.25] tracking-[-0.014em] text-ink">
                {active.detailTitre}
              </h3>
              <p className="mt-[14px] text-pretty text-[14px] leading-[1.7] text-ink-muted">
                {active.detailTexte}
              </p>
            </div>

            <div className="mt-6 flex items-center gap-2">
              <span className="font-mono text-[11px] font-semibold text-accent-text">
                {String(etape + 1).padStart(2, "0")}
              </span>
              <span className="font-mono text-[11px] text-control-strong">
                / 05
              </span>
              <button
                type="button"
                onClick={() =>
                  setEtape((precedent) => (precedent + 1) % ETAPES.length)
                }
                className="ml-[10px] inline-flex h-[30px] items-center gap-[7px] rounded-control border border-control bg-surface px-3 text-[12.5px] font-semibold text-ink transition-colors duration-[120ms] hover:border-control-strong hover:bg-surface-alt"
              >
                Étape suivante
                <Icon name="arrow_forward" size={15} />
              </button>
            </div>
          </div>

          <div
            id={`${id}-panneau`}
            role="tabpanel"
            aria-labelledby={`${id}-onglet-${String(etape)}`}
            tabIndex={0}
            className="order-1 col-span-2 min-w-0"
          >
            <div className="overflow-hidden rounded-card border border-line bg-surface">
              <div className="flex h-[38px] items-center gap-[9px] border-b border-line-soft bg-surface-alt px-3">
                <span className="text-ink-tertiary">
                  <Icon name={active.panneau.icone} size={15} />
                </span>
                <span className="text-[12.5px] font-semibold text-ink">
                  {active.panneau.titre}
                </span>
                <span className="ml-auto font-mono text-[10.5px] text-ink-faint">
                  {active.panneau.fil}
                </span>
              </div>
              <Ecran />
            </div>
          </div>
        </div>
      </Reveal>
    </section>
  );
}
