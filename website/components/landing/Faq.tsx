"use client";

import { useId, useState } from "react";

import { LineIcon } from "@/components/ui/LineIcon";
import { Reveal } from "@/components/ui/Reveal";
import { cn } from "@/lib/cn";
import { FAQ } from "@/lib/data/faq";
import { GITHUB_DISCUSSIONS } from "@/lib/data/liens";

/**
 * FAQ : une seule question ouverte à la fois, la première au chargement.
 *
 * Vrais `<button>` avec `aria-expanded` et `aria-controls`. L'ouverture passe par
 * `grid-template-rows: 0fr → 1fr` (hauteur inconnue animée sans mesure) ; le panneau
 * fermé porte `inert` pour que ses liens ne restent pas atteignables au clavier.
 */
export function Faq() {
  const [ouverte, setOuverte] = useState<number | null>(0);
  const id = useId();

  return (
    <section id="faq" aria-labelledby="faq-titre" className="border-y border-bd bg-panel">
      <Reveal className="mx-auto grid max-w-[1200px] gap-8 px-4 py-16 md:px-8 md:py-24 lg:grid-cols-[340px_minmax(0,1fr)] lg:gap-20 xl:px-0 xl:py-28">
        <div className="flex flex-col gap-4">
          <span className="eyebrow">Questions</span>
          <h2 id="faq-titre" className="serif-title text-[30px] leading-[1.1] tracking-[-0.025em] md:text-[36px]">
            Avant d&apos;installer.
          </h2>
          <a
            href={GITHUB_DISCUSSIONS}
            target="_blank"
            rel="noopener noreferrer"
            className="inline-flex min-h-[44px] items-center gap-[6px] text-[14px] text-tx-3 hover:text-tx md:min-h-0"
          >
            Une autre question ? Les discussions GitHub sont ouvertes
            <LineIcon name="arrow-up-right" size={13} />
          </a>
        </div>

        <div className="border-t border-bd">
          {FAQ.map((entree, index) => {
            const estOuverte = ouverte === index;
            const idReponse = `${id}-reponse-${String(index)}`;
            return (
              <div key={entree.question} className="border-b border-bd">
                <h3 className="m-0">
                  <button
                    type="button"
                    onClick={() => setOuverte(estOuverte ? null : index)}
                    aria-expanded={estOuverte}
                    aria-controls={idReponse}
                    className="flex min-h-[60px] w-full cursor-pointer items-center gap-6 py-4 text-left text-[16px] font-medium text-tx md:text-[17px]"
                  >
                    <span className="flex-1 text-pretty">{entree.question}</span>
                    <span
                      aria-hidden="true"
                      className={cn(
                        "block flex-none text-tx-4 transition-transform duration-[220ms] ease-out-soft",
                        estOuverte && "rotate-45",
                      )}
                    >
                      <LineIcon name="close" size={14} className="rotate-45" />
                    </span>
                  </button>
                </h3>
                <div
                  id={idReponse}
                  inert={!estOuverte}
                  className={cn(
                    "grid transition-[grid-template-rows,opacity] duration-[320ms] ease-out-soft",
                    estOuverte ? "grid-rows-[1fr] opacity-100" : "grid-rows-[0fr] opacity-0",
                  )}
                >
                  <div className="overflow-hidden">
                    <p className="max-w-[760px] text-pretty pb-6 text-[15px] leading-[1.7] text-tx-3">
                      {entree.reponse}
                    </p>
                  </div>
                </div>
              </div>
            );
          })}
        </div>
      </Reveal>
    </section>
  );
}
