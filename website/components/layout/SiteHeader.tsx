import Link from "next/link";

import { MobileNav } from "@/components/layout/MobileNav";
import { ThemeToggle } from "@/components/layout/ThemeToggle";
import { BrandIcon } from "@/components/ui/BrandIcon";
import { BrandMark } from "@/components/ui/BrandMark";
import { ButtonLink } from "@/components/ui/Button";
import { GITHUB_REPO } from "@/lib/data/liens";
import { NAV_SECTIONS } from "@/lib/data/navigation";

/**
 * Barre collante du site : marque, sections, GitHub, Télécharger, thème.
 *
 * Dès 1024 px les sections tiennent sur une ligne ; en dessous elles passent dans un
 * menu déroulant (`MobileNav`), et Télécharger reste visible dès 640 px.
 */
export function SiteHeader() {
  return (
    <header className="sticky top-0 z-40 border-b border-bd bg-app-glass backdrop-blur-[18px]">
      <div className="mx-auto flex h-[60px] max-w-[1200px] items-center gap-6 px-4 md:px-8 xl:px-0">
        <Link
          href="/"
          aria-label="Candilog, accueil"
          className="flex shrink-0 items-center gap-[9px] text-tx hover:text-tx"
        >
          <BrandMark size={24} />
          <span className="text-[15px] font-semibold tracking-[-0.01em]">Candilog</span>
        </Link>

        <nav aria-label="Sections" className="hidden flex-1 justify-center gap-7 lg:flex">
          {NAV_SECTIONS.map(({ libelle, href }) => (
            <a
              key={href}
              href={`/${href}`}
              className="whitespace-nowrap text-[13.5px] text-tx-3 hover:text-tx"
            >
              {libelle}
            </a>
          ))}
        </nav>

        <div className="ml-auto flex shrink-0 items-center gap-2 lg:ml-0">
          <ButtonLink
            href={GITHUB_REPO}
            target="_blank"
            rel="noopener noreferrer"
            variante="secondaire"
            taille="compact"
            visible="md"
          >
            <BrandIcon name="github" size={14} />
            GitHub
          </ButtonLink>
          <ButtonLink href="/#telecharger" taille="compact" visible="sm">
            Télécharger
          </ButtonLink>
          <ThemeToggle />
          <MobileNav />
        </div>
      </div>
    </header>
  );
}
