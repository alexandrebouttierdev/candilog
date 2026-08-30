import { HeroAppWindow } from "@/components/landing/HeroAppWindow";
import { BrandIcon } from "@/components/ui/BrandIcon";
import { ButtonLink } from "@/components/ui/Button";
import { DownloadMenu } from "@/components/ui/DownloadMenu";
import { Icon } from "@/components/ui/Icon";
import { Reveal } from "@/components/ui/Reveal";
import { GITHUB_REPO } from "@/lib/data/liens";

const MENTIONS = ["Gratuit pour un usage personnel", "Sans compte"] as const;

/**
 * Hero (§7.2) : deux colonnes, la fenêtre de l'app débordant à droite.
 *
 * `overflow-x-clip` sur la section : la fenêtre déborde volontairement de 110px à
 * droite au-delà de 1180px (§6). Sans ce clip, ce débordement devient une barre de
 * défilement horizontale sur toute la page — le prototype le coupe sur son conteneur
 * racine, on le fait ici, au plus près de la cause.
 *
 * `clip` et non `hidden` : `overflow-x: hidden` force `overflow-y` à `auto`, ce qui
 * transforme la section en conteneur de défilement et fait apparaître une barre
 * verticale de 15px à l'intérieur du hero. `clip` coupe sans créer ce conteneur.
 */
export function Hero() {
  return (
    <section className="overflow-x-clip border-b border-line pt-[clamp(48px,7vw,96px)]">
      <Reveal className="grid grid-cols-[repeat(auto-fit,minmax(min(420px,100%),1fr))] items-end gap-[clamp(32px,5vw,64px)] pl-[clamp(16px,4vw,40px)]">
        <div className="max-w-[560px] pb-[clamp(40px,6vw,80px)]">
          <h1 className="text-balance text-[clamp(30px,3.6vw,44px)] font-semibold leading-[1.1] tracking-[-0.022em] text-ink">
            Vos candidatures, vos documents
            <br />
            et vos entretiens au même endroit.
          </h1>

          <p className="mt-[22px] max-w-[460px] text-pretty text-[16px] leading-[1.6] text-ink-muted">
            De l&apos;offre repérée le matin à l&apos;entretien de la semaine
            prochaine : vos candidatures, vos documents et vos relances dans une
            seule fenêtre, sur votre ordinateur.
          </p>

          <div className="mt-8 flex flex-wrap items-center gap-[10px]">
            <DownloadMenu />
            <ButtonLink
              href={GITHUB_REPO}
              target="_blank"
              rel="noopener noreferrer"
              variante="secondaire"
            >
              <BrandIcon name="github" size={16} />
              Voir sur GitHub
            </ButtonLink>
          </div>

          <div className="mt-[18px] flex flex-wrap items-center gap-[14px]">
            {MENTIONS.map((mention) => (
              <span
                key={mention}
                className="inline-flex items-center gap-[7px] text-[12.5px] text-ink-muted"
              >
                <span className="text-success">
                  <Icon name="check" size={16} />
                </span>
                {mention}
              </span>
            ))}
          </div>
        </div>

        <div className="relative min-w-0">
          <HeroAppWindow />
        </div>
      </Reveal>
    </section>
  );
}
