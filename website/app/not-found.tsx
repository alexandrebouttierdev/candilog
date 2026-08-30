import type { Metadata } from "next";

import { SiteFooter } from "@/components/layout/SiteFooter";
import { SiteHeader } from "@/components/layout/SiteHeader";
import { ButtonLink } from "@/components/ui/Button";
import { NAV_SECTIONS } from "@/lib/data/navigation";
import { PAGES_LEGALES } from "@/lib/data/legal";

export const metadata: Metadata = {
  title: "Page introuvable — Candilog",
};

/* Le design ne prévoyait pas d'écran 404. Celui-ci reprend la mise en page des pages
   légales — colonne centrée, sur-titre à puce, H1 en clamp — pour ne rien inventer :
   aucune valeur ni aucun token qui ne soit déjà ailleurs sur le site.
   En export statique, Next en fait `out/404.html`, que GitHub Pages sert tel quel. */
export default function NotFound() {
  return (
    <>
      <SiteHeader />
      <main className="mx-auto w-full max-w-[800px] px-[clamp(20px,4vw,40px)] py-[clamp(64px,9vw,120px)]">
        <div className="mb-[18px] flex items-center gap-[9px]">
          <span aria-hidden="true" className="size-[6px] rounded-full bg-accent" />
          <span className="font-mono text-[12.5px] font-semibold text-ink-tertiary">404</span>
        </div>

        <h1 className="text-balance text-[clamp(28px,3.2vw,40px)] font-semibold leading-[1.12] tracking-[-0.022em] text-ink">
          Cette page n&apos;existe pas.
        </h1>
        <p className="mt-[18px] max-w-[560px] text-pretty text-[16px] leading-[1.7] text-ink-muted">
          Le lien est peut-être ancien, ou l&apos;adresse comporte une erreur. Le reste du site
          est toujours là.
        </p>

        <div className="mt-8 flex flex-wrap gap-[10px]">
          <ButtonLink href="/">Retour à l&apos;accueil</ButtonLink>
          <ButtonLink href="/#telecharger" variante="secondaire">
            Télécharger Candilog
          </ButtonLink>
        </div>

        <nav
          aria-label="Sections du site"
          className="mt-12 flex flex-wrap gap-[10px] border-t border-control pt-[22px]"
        >
          <span className="mb-1 w-full text-[12.5px] text-ink-faint">Aller à</span>
          {NAV_SECTIONS.map(({ libelle, href }) => (
            <a
              key={href}
              href={`/${href}`}
              className="inline-flex h-[30px] items-center rounded-control border border-control bg-surface px-3 text-[12.5px] font-semibold text-ink transition-colors duration-[120ms] hover:bg-surface-alt"
            >
              {libelle}
            </a>
          ))}
        </nav>

        <nav
          aria-label="Pages légales"
          className="mt-6 flex flex-wrap items-center gap-x-[18px] gap-y-2"
        >
          {PAGES_LEGALES.map(({ cle, libelle, href }) => (
            <a
              key={cle}
              href={href}
              className="text-[12px] text-ink-tertiary hover:text-accent-text"
            >
              {libelle}
            </a>
          ))}
        </nav>
      </main>
      <SiteFooter />
    </>
  );
}
