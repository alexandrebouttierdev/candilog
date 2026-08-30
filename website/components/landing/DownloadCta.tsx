import Image from "next/image";

import { BrandIcon } from "@/components/ui/BrandIcon";
import { ButtonLink } from "@/components/ui/Button";
import { DownloadMenu } from "@/components/ui/DownloadMenu";
import { Reveal } from "@/components/ui/Reveal";
import { GITHUB_REPO } from "@/lib/data/liens";

const SYSTEMES = [
  { logo: "windows" as const, nom: "Windows", detail: "Installateur .exe" },
  { logo: "apple" as const, nom: "macOS", detail: "Apple Silicon · Intel" },
  { logo: "linux" as const, nom: "Linux", detail: "Ubuntu · Fedora prévus" },
];

/** Bloc de téléchargement (§7.10) : icône d'app, accroche, DownloadMenu et la
 *  rangée des trois plateformes. */
export function DownloadCta() {
  return (
    <section id="telecharger" className="border-b border-line bg-surface">
      <Reveal className="mx-auto max-w-[1240px] px-[clamp(16px,4vw,40px)] py-[clamp(56px,7vw,104px)] text-center">
        <Image
          src="/logo-candilog-app.svg"
          alt=""
          width={56}
          height={56}
          className="inline-block"
        />

        <h2 className="mx-auto mt-[22px] max-w-[600px] text-balance text-[clamp(25px,2.8vw,38px)] font-semibold leading-[1.14] tracking-[-0.02em] text-ink">
          Installez Candilog et ouvrez votre suivi.
        </h2>
        <p className="mx-auto mt-[14px] max-w-[440px] text-pretty text-[14.5px] leading-[1.65] text-ink-muted">
          Gratuit pour un usage personnel. Une application à télécharger, rien à
          configurer, aucun compte à créer.
        </p>

        <div className="mt-7 flex flex-wrap justify-center gap-[10px]">
          <DownloadMenu />
          <ButtonLink
            href={GITHUB_REPO}
            target="_blank"
            rel="noopener noreferrer"
            variante="secondaire"
            className="h-10 gap-[9px] px-[17px] text-[14px]"
          >
            <BrandIcon name="github" size={17} />
            Voir sur GitHub
          </ButtonLink>
        </div>

        <div className="mx-auto mt-[38px] flex max-w-[720px] flex-wrap justify-center border-t border-line">
          {SYSTEMES.map((systeme, index) => (
            <div
              key={systeme.nom}
              className={`flex-[1_1_200px] px-[18px] py-4 text-left ${
                index > 0 ? "border-l border-line" : ""
              }`}
            >
              <div className="flex items-center gap-[9px]">
                {systeme.logo === "windows" ? (
                  <span
                    aria-hidden="true"
                    className="grid size-[15px] shrink-0 grid-cols-2 grid-rows-2 gap-[2px]"
                  >
                    <span className="bg-ink-muted" />
                    <span className="bg-ink-muted" />
                    <span className="bg-ink-muted" />
                    <span className="bg-ink-muted" />
                  </span>
                ) : (
                  <span className="block text-ink-muted">
                    <BrandIcon name={systeme.logo} size={15} />
                  </span>
                )}
                <span className="text-[13px] font-semibold text-ink">
                  {systeme.nom}
                </span>
              </div>
              <p className="mt-[5px] font-mono text-[10.5px] text-ink-faint">
                {systeme.detail}
              </p>
            </div>
          ))}
        </div>
      </Reveal>
    </section>
  );
}
