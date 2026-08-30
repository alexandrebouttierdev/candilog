import Image from "next/image";
import Link from "next/link";
import type { ReactNode } from "react";

import { ThemeToggle } from "@/components/layout/ThemeToggle";
import { ButtonLink } from "@/components/ui/Button";
import { Icon } from "@/components/ui/Icon";
import { LICENCE_POLYFORM } from "@/lib/data/liens";
import { PAGES_LEGALES, type ClePageLegale } from "@/lib/data/legal";

/** En-tête des pages légales : logo, bascule de thème, retour au site (§7.13).
 *  Plus simple que celui de la landing — pas de nav de sections. */
function LegalHeader() {
  return (
    <header className="sticky top-0 z-40 flex h-[56px] items-center gap-6 border-b border-line bg-page-glass px-[clamp(16px,4vw,40px)] backdrop-blur-[18px]">
      <Link href="/" className="mr-auto flex items-center gap-[9px] text-ink hover:text-ink">
        <Image src="/logo-candilog.svg" alt="" width={27} height={27} className="block shrink-0" />
        <span className="text-[16px] font-semibold tracking-[-0.012em]">Candilog</span>
      </Link>
      <ThemeToggle />
      <ButtonLink href="/" variante="secondaire" taille="compact">
        <Icon name="arrow_back" size={16} />
        Retour au site
      </ButtonLink>
    </header>
  );
}

/** Pied de page des pages légales — volontairement plus court que celui de la
 *  landing : pas de nav de sections, et le filet est en haut de la première rangée. */
function LegalFooter() {
  return (
    <footer className="mt-[clamp(48px,6vw,88px)] bg-page">
      <div className="mx-auto max-w-[1240px] px-[clamp(16px,4vw,40px)]">
        <nav
          aria-label="Pages du site"
          className="flex flex-wrap items-center justify-center gap-5 border-t border-line pb-[18px] pt-[30px]"
        >
          <Link
            href="/"
            className="inline-flex min-h-[44px] items-center text-[12.5px] text-ink-muted hover:text-accent-text md:min-h-0"
          >
            Accueil
          </Link>
          {PAGES_LEGALES.map(({ cle, libelle, href }) => (
            <Link
              key={cle}
              href={href}
              className="inline-flex min-h-[44px] items-center text-[12.5px] text-ink-muted hover:text-accent-text md:min-h-0"
            >
              {libelle}
            </Link>
          ))}
        </nav>
        <div className="flex flex-wrap items-center justify-center gap-x-[18px] gap-y-2 pb-[30px]">
          <span className="text-[12px] text-ink-faint">Copyright © 2026 Alexandre Bouttier</span>
          <a
            href={LICENCE_POLYFORM}
            target="_blank"
            rel="noopener noreferrer"
            className="text-[12px] text-ink-faint hover:text-accent-text"
          >
            Licence PolyForm Noncommercial 1.0.0
          </a>
        </div>
      </div>
    </footer>
  );
}

/** Navigation entre les quatre pages légales, en pied de colonne. La page courante
 *  est un `<span>` inerte plutôt qu'un lien vers elle-même. */
function NavPagesLegales({ courante }: { courante: ClePageLegale }) {
  return (
    <nav
      aria-label="Autres pages légales"
      className="mt-12 flex flex-wrap gap-[10px] border-t border-control pt-[22px]"
    >
      <span className="mb-1 w-full text-[12.5px] text-ink-faint">Pages légales</span>
      {PAGES_LEGALES.map(({ cle, libelle, href }) =>
        cle === courante ? (
          <span
            key={cle}
            aria-current="page"
            className="inline-flex h-[30px] items-center rounded-control border border-control bg-surface px-3 text-[12.5px] font-semibold text-ink-faint"
          >
            {libelle}
          </span>
        ) : (
          <Link
            key={cle}
            href={href}
            className="inline-flex h-[30px] items-center rounded-control border border-control bg-surface px-3 text-[12.5px] font-semibold text-ink transition-colors duration-[120ms] hover:bg-surface-alt"
          >
            {libelle}
          </Link>
        ),
      )}
    </nav>
  );
}

/** Gabarit commun aux quatre pages légales : colonne unique de 800px. */
export function LegalLayout({
  courante,
  children,
}: {
  courante: ClePageLegale;
  children: ReactNode;
}) {
  return (
    <div className="flex min-h-screen flex-col bg-page text-ink">
      <LegalHeader />
      <main className="mx-auto w-full max-w-[800px] px-[clamp(20px,4vw,40px)] pt-[clamp(44px,5vw,76px)]">
        {children}
        <NavPagesLegales courante={courante} />
      </main>
      <div className="mt-auto">
        <LegalFooter />
      </div>
    </div>
  );
}
