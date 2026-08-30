import Link from "next/link";

import { BrandIcon } from "@/components/ui/BrandIcon";
import { Icon } from "@/components/ui/Icon";
import { Reveal } from "@/components/ui/Reveal";
import { GITHUB_ISSUES, GITHUB_REPO } from "@/lib/data/liens";

const INSPECTEUR = [
  { libelle: "Modèle", valeur: "Source available", mono: false },
  {
    libelle: "Usage non commercial",
    valeur: "PolyForm Noncommercial 1.0.0",
    mono: false,
  },
  {
    libelle: "Identifiant SPDX",
    valeur: "PolyForm-Noncommercial-1.0.0",
    mono: true,
  },
  { libelle: "Usage commercial", valeur: "Licence séparée", mono: false },
  {
    libelle: "Mises à jour",
    valeur: "Vérifiables depuis l'application",
    mono: false,
  },
];

const BOUTON_SECONDAIRE =
  "inline-flex h-9 shrink-0 items-center gap-2 whitespace-nowrap rounded-control border border-control bg-surface px-[14px] text-[13px] font-semibold text-ink transition-colors duration-[120ms] hover:border-control-strong hover:bg-surface-alt";

/** Code source (§7.8) : texte et boutons à gauche, inspecteur de licence à droite. */
export function SourceAvailable() {
  return (
    <section id="opensource" className="border-b border-line bg-page">
      <Reveal className="mx-auto grid max-w-[1240px] grid-cols-[repeat(auto-fit,minmax(min(320px,100%),1fr))] items-center gap-[clamp(28px,5vw,64px)] px-[clamp(16px,4vw,40px)] py-[clamp(48px,6vw,88px)]">
        <div>
          <h2 className="text-[clamp(24px,2.6vw,34px)] font-semibold leading-[1.14] tracking-[-0.02em] text-ink">
            Le code source de Candilog est disponible.
          </h2>
          <p className="mt-4 max-w-[470px] text-pretty text-[14px] leading-[1.7] text-ink-muted">
            Le code du projet est public sur GitHub. Vous pouvez vérifier ce que
            fait l&apos;application, signaler un problème, proposer une
            correction ou suivre les versions qui sortent.
          </p>
          <p className="mt-[14px] max-w-[470px] text-pretty text-[14px] leading-[1.7] text-ink-muted">
            Candilog est un projet source available : les usages autorisés non
            commerciaux sont régis par la{" "}
            <Link href="/licence">PolyForm Noncommercial License 1.0.0</Link>.
            Toute utilisation commerciale nécessite une licence commerciale
            séparée.
          </p>

          <div className="mt-[26px] flex flex-wrap gap-[10px]">
            {/* Bouton « encre » : le seul du site à inverser fond et texte plutôt
                qu'à utiliser l'accent — c'est la convention GitHub. */}
            <a
              href={GITHUB_REPO}
              target="_blank"
              rel="noopener noreferrer"
              className="inline-flex h-9 shrink-0 items-center gap-2 whitespace-nowrap rounded-control border border-ink bg-ink px-[15px] text-[13px] font-semibold text-page transition-opacity duration-[120ms] hover:opacity-90"
            >
              <BrandIcon name="github" size={16} />
              Voir sur GitHub
            </a>
            <a
              href={GITHUB_ISSUES}
              target="_blank"
              rel="noopener noreferrer"
              className={BOUTON_SECONDAIRE}
            >
              <Icon name="bug_report" size={16} />
              Signaler un problème
            </a>
            <Link href="/licence" className={BOUTON_SECONDAIRE}>
              <Icon name="gavel" size={16} />
              Voir la licence
            </Link>
          </div>
        </div>

        <div className="min-w-0">
          {INSPECTEUR.map((ligne, index) => (
            <div
              key={ligne.libelle}
              className={`flex flex-wrap justify-between gap-3 ${
                index === 0
                  ? "border-b border-control pb-[14px]"
                  : "border-b border-line py-[14px]"
              }`}
            >
              <span className="text-[13px] text-ink-tertiary">
                {ligne.libelle}
              </span>
              <span
                className={
                  ligne.mono
                    ? "font-mono text-[11.5px] text-ink-body"
                    : "text-right text-[13px] font-semibold text-ink"
                }
              >
                {ligne.valeur}
              </span>
            </div>
          ))}
        </div>
      </Reveal>
    </section>
  );
}
