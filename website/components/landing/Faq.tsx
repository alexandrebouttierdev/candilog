"use client";

import { useId, useState } from "react";

import { Icon } from "@/components/ui/Icon";
import { Reveal } from "@/components/ui/Reveal";
import { cn } from "@/lib/cn";
import { FAQ } from "@/lib/data/faq";
import { GITHUB_DISCUSSIONS } from "@/lib/data/liens";

/**
 * FAQ (§7.9) : une seule question ouverte à la fois, ouverte sur la première au
 * chargement.
 *
 * L'ouverture passe par `grid-template-rows: 0fr → 1fr`, ce qui anime une hauteur
 * inconnue sans la mesurer. Le prototype utilisait des `<div onClick>` : ici de
 * vrais `<button>` avec `aria-expanded` et `aria-controls` (§9).
 */
export function Faq({
  ouverteInitiale = 0,
}: {
  ouverteInitiale?: number | null;
}) {
  const [ouverte, setOuverte] = useState<number | null>(ouverteInitiale);
  const id = useId();

  return (
    <section id="faq" className="border-b border-line bg-page">
      <Reveal className="mx-auto max-w-[1240px] px-[clamp(16px,4vw,40px)] py-[clamp(48px,6vw,88px)]">
        <div className="mb-[34px] text-center">
          <h2 className="text-[clamp(24px,2.6vw,34px)] font-semibold leading-[1.14] tracking-[-0.02em] text-ink">
            Questions fréquentes
          </h2>
          <p className="mx-auto mt-3 max-w-[420px] text-[14px] leading-[1.65] text-ink-tertiary">
            Ce qu&apos;il faut savoir avant d&apos;installer Candilog.
          </p>
        </div>

        <div className="mx-auto max-w-[800px] overflow-hidden rounded-panel border border-control bg-surface">
          {FAQ.map((entree, index) => {
            const estOuverte = ouverte === index;
            const idReponse = `${id}-reponse-${String(index)}`;
            return (
              <div
                key={entree.question}
                className={cn(index > 0 && "border-t border-line-soft")}
              >
                <button
                  type="button"
                  onClick={() => {
                    setOuverte(estOuverte ? null : index);
                  }}
                  aria-expanded={estOuverte}
                  aria-controls={idReponse}
                  className={cn(
                    "flex w-full cursor-pointer items-center gap-4 py-[19px] pr-5 text-left transition-[padding-left,background] duration-[220ms] ease-out-soft hover:bg-surface-alt",
                    estOuverte ? "pl-7" : "pl-5",
                  )}
                >
                  <span
                    aria-hidden="true"
                    className={cn(
                      "w-[3px] shrink-0 self-stretch rounded-[2px] bg-accent transition-opacity duration-[220ms]",
                      estOuverte ? "opacity-100" : "opacity-0",
                    )}
                  />
                  <span
                    className={cn(
                      "text-pretty text-[15.5px] font-semibold leading-[1.4] transition-colors duration-[200ms]",
                      estOuverte ? "text-ink" : "text-ink-body",
                    )}
                  >
                    {entree.question}
                  </span>
                  <span
                    className={cn(
                      "ml-auto block shrink-0 transition-[transform,color] duration-[180ms] ease-out-soft",
                      estOuverte
                        ? "rotate-180 text-accent-text"
                        : "rotate-0 text-ink-faint",
                    )}
                  >
                    <Icon name="expand_more" size={20} />
                  </span>
                </button>

                <div
                  id={idReponse}
                  /* Replié, le panneau garde une hauteur nulle mais reste dans le
                     flux : sans `inert`, le lien de la réponse « Candilog est-il
                     gratuit ? » resterait atteignable au clavier alors qu'il est
                     invisible. */
                  inert={!estOuverte}
                  className={cn(
                    "grid transition-[grid-template-rows,opacity] duration-[320ms] ease-out-soft",
                    estOuverte
                      ? "grid-rows-[1fr] opacity-100"
                      : "grid-rows-[0fr] opacity-0",
                  )}
                >
                  <div className="overflow-hidden">
                    <p
                      className={cn(
                        "max-w-[640px] text-pretty pb-[22px] pl-5 pr-14 text-[14px] leading-[1.75] text-ink-muted transition-transform duration-[320ms] ease-out-soft",
                        estOuverte ? "translate-y-0" : "-translate-y-[6px]",
                      )}
                    >
                      {entree.reponse}
                    </p>
                  </div>
                </div>
              </div>
            );
          })}
        </div>

        <div className="mt-[22px] text-center">
          <a
            href={GITHUB_DISCUSSIONS}
            target="_blank"
            rel="noopener noreferrer"
            className="inline-flex items-center gap-[7px] text-[13px] text-ink-muted hover:text-accent-text"
          >
            Une autre question ? Les discussions GitHub sont ouvertes
            <Icon name="arrow_outward" size={16} />
          </a>
        </div>
      </Reveal>
    </section>
  );
}
