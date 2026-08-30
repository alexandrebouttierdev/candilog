import Image from "next/image";
import Link from "next/link";

import { ThemeToggle } from "@/components/layout/ThemeToggle";
import { BrandIcon } from "@/components/ui/BrandIcon";
import { ButtonLink } from "@/components/ui/Button";
import { NAV_SECTIONS } from "@/lib/data/navigation";
import { GITHUB_REPO } from "@/lib/data/liens";

/**
 * Barre sticky du site (§7.1).
 *
 * Elle passe sur deux lignes sous ~700px grâce au `flex-wrap` : le logo et la nav
 * défilante tiennent la première, les actions passent dessous. La nav garde son
 * `overflow-x-auto` — sans lui la page déborde sur mobile (§6).
 */
export function SiteHeader() {
  return (
    <header className="sticky top-0 z-40 flex min-h-[56px] flex-wrap items-center gap-x-6 gap-y-[10px] border-b border-line bg-page-glass px-[clamp(16px,4vw,40px)] py-[10px] backdrop-blur-[18px]">
      <Link href="/" className="flex shrink-0 items-center gap-[9px] text-ink hover:text-ink">
        <Image src="/logo-candilog.svg" alt="" width={27} height={27} className="block shrink-0" />
        <span className="text-[16px] font-semibold tracking-[-0.012em] text-ink">Candilog</span>
      </Link>

      <nav
        aria-label="Sections du site"
        className="no-scrollbar flex min-w-0 flex-[1_1_230px] items-center gap-[22px] overflow-x-auto"
      >
        {NAV_SECTIONS.map(({ libelle, href }) => (
          <a
            key={href}
            href={href}
            className="inline-flex min-h-[44px] items-center md:min-h-0 whitespace-nowrap text-[12.5px] text-ink-muted hover:text-accent-text"
          >
            {libelle}
          </a>
        ))}
      </nav>

      <div className="ml-auto flex shrink-0 items-center gap-2">
        <ButtonLink
          href={GITHUB_REPO}
          target="_blank"
          rel="noopener noreferrer"
          variante="secondaire"
          taille="compact"
        >
          <BrandIcon name="github" size={14} />
          GitHub
        </ButtonLink>
        <ButtonLink href="#telecharger" taille="compact">
          Télécharger
        </ButtonLink>
        <ThemeToggle />
      </div>
    </header>
  );
}
