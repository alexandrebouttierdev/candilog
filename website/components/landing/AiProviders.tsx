"use client";

import { useState } from "react";

import { BrandIcon } from "@/components/ui/BrandIcon";
import { Reveal } from "@/components/ui/Reveal";
import { FOURNISSEURS_IA, type FournisseurIa } from "@/lib/data/fournisseursIa";

/* Les pastilles sont le seul endroit du site avec une ombre portée (§5), et le
   seul avec des couleurs hors palette : ce sont les dégradés de marque des
   fournisseurs, pas des couleurs de thème. */

const ATOUTS = [
  {
    titre: "Votre clé, votre compte",
    detail: "Aucune facturation par Candilog",
  },
  { titre: "Ou 100 % local", detail: "Modèle Ollama, hors ligne" },
];

function Pastille({ fournisseur }: { fournisseur: FournisseurIa }) {
  const [survol, setSurvol] = useState(false);

  return (
    <span
      onMouseEnter={() => setSurvol(true)}
      onMouseLeave={() => setSurvol(false)}
      className="relative grid size-[74px] place-items-center rounded-app transition-[transform,box-shadow] duration-[240ms] ease-out-soft"
      style={{
        background: fournisseur.degrade,
        boxShadow: survol ? fournisseur.ombreSurvol : fournisseur.ombre,
        transform: survol
          ? "rotate(0deg) translateY(-6px)"
          : `rotate(${String(fournisseur.rotation)}deg)`,
      }}
    >
      {/* --on-accent : le token du site pour un premier plan sur fond coloré
          saturé. Le design encode la même valeur dans ses URL (…/ffffff). */}
      <span className="block text-on-accent">
        <BrandIcon name={fournisseur.logo} size={36} set="providers" />
      </span>
    </span>
  );
}

export function AiProviders() {
  return (
    <section id="ia" className="border-b border-line bg-surface">
      <Reveal className="mx-auto grid max-w-[1240px] grid-cols-[repeat(auto-fit,minmax(min(320px,100%),1fr))] items-center gap-[clamp(28px,5vw,64px)] px-[clamp(16px,4vw,40px)] py-[clamp(48px,6vw,88px)]">
        <div>
          <h2 className="text-pretty text-[clamp(24px,2.6vw,34px)] font-semibold leading-[1.14] tracking-[-0.02em] text-ink">
            L&apos;assistance IA, avec le moteur que vous choisissez.
          </h2>
          <p className="mt-4 max-w-[470px] text-pretty text-[14px] leading-[1.7] text-ink-muted">
            L&apos;analyse d&apos;une offre, l&apos;extraction des informations
            et l&apos;adaptation d&apos;un document passent par le fournisseur
            que vous connectez avec votre propre clé. Vous pouvez aussi faire
            tourner un modèle en local avec Ollama : dans ce cas, rien ne quitte
            votre ordinateur.
          </p>

          <div className="mt-[26px] flex flex-wrap gap-5 border-t border-line pt-[18px]">
            {ATOUTS.map((atout) => (
              <div key={atout.titre} className="min-w-[130px]">
                <p className="text-[13px] font-semibold text-ink">
                  {atout.titre}
                </p>
                <p className="mt-1 text-[12.5px] leading-[1.55] text-ink-tertiary">
                  {atout.detail}
                </p>
              </div>
            ))}
          </div>
        </div>

        <div className="min-w-0">
          {/* flex-wrap : les décalages verticaux de la vague tiennent tant que les
              cinq pastilles sont sur une ligne (§6). */}
          <ul className="flex flex-wrap items-start justify-center gap-[14px] py-[26px]">
            {FOURNISSEURS_IA.map((fournisseur) => (
              <li
                key={fournisseur.nom}
                className="flex w-[94px] flex-col items-center gap-[11px]"
                style={{ marginTop: fournisseur.decalage }}
              >
                <Pastille fournisseur={fournisseur} />
                {fournisseur.badge ? (
                  <span className="flex flex-col items-center gap-1">
                    <span className="text-center text-[12.5px] font-semibold text-ink">
                      {fournisseur.nom}
                    </span>
                    <span className="inline-flex h-[19px] items-center whitespace-nowrap rounded-pill border border-tint-border bg-tint-10 px-2 text-[10.5px] font-semibold text-accent-text">
                      {fournisseur.badge}
                    </span>
                  </span>
                ) : (
                  <span className="text-center text-[12.5px] font-semibold text-ink">
                    {fournisseur.nom}
                  </span>
                )}
              </li>
            ))}
          </ul>
        </div>
      </Reveal>
    </section>
  );
}
