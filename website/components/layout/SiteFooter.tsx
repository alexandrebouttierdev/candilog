import Image from "next/image";
import Link from "next/link";

import { BrandIcon } from "@/components/ui/BrandIcon";
import { LICENCE_POLYFORM, SITE_AUTEUR } from "@/lib/data/liens";
import { LIEN_GITHUB_PIED, NAV_LEGALE, NAV_PIED } from "@/lib/data/navigation";

/** Pied de page centré à trois niveaux (§7.12) : marque + nav, rangée légale,
 *  puis mention de copyright et licence. */
export function SiteFooter() {
  return (
    <footer className="bg-page">
      <div className="mx-auto max-w-[1240px] px-[clamp(16px,4vw,40px)]">
        <div className="flex flex-wrap items-center justify-center gap-5 pb-[18px] pt-[30px]">
          <div className="flex items-center gap-[10px]">
            <span className="grid size-6 place-items-center rounded-control border border-line bg-surface">
              <Image src="/logo-candilog.svg" alt="" width={17} height={17} className="block" />
            </span>
            <span className="text-[13px] font-semibold text-ink">Candilog</span>
          </div>

          <nav
            aria-label="Pied de page"
            className="flex flex-wrap items-center justify-center gap-5"
          >
            {NAV_PIED.map(({ libelle, href }) => (
              <a
                key={href}
                href={href}
                className="inline-flex min-h-[44px] items-center md:min-h-0 whitespace-nowrap text-[12.5px] text-ink-muted hover:text-accent-text"
              >
                {libelle}
              </a>
            ))}
            <a
              href={LIEN_GITHUB_PIED.href}
              target="_blank"
              rel="noopener noreferrer"
              className="inline-flex min-h-[44px] items-center gap-[7px] whitespace-nowrap text-[12.5px] text-ink-muted hover:text-accent-text md:min-h-0"
            >
              <BrandIcon name="github" size={13} />
              {LIEN_GITHUB_PIED.libelle}
            </a>
          </nav>
        </div>

        <nav
          aria-label="Informations légales"
          className="flex flex-wrap items-center justify-center gap-[18px] pb-4"
        >
          {NAV_LEGALE.map(({ libelle, href }) => (
            <Link
              key={href}
              href={href}
              className="inline-flex min-h-[44px] items-center md:min-h-0 text-[12px] text-ink-tertiary hover:text-accent-text"
            >
              {libelle}
            </Link>
          ))}
        </nav>

        <div className="flex flex-wrap items-center justify-center gap-x-[18px] gap-y-2 border-t border-line pb-[30px] pt-[14px]">
          <span className="text-[12px] text-ink-faint">Copyright © 2026 Alexandre Bouttier</span>
          <a
            href={LICENCE_POLYFORM}
            target="_blank"
            rel="noopener noreferrer"
            className="text-[12px] text-ink-faint hover:text-accent-text"
          >
            Licence PolyForm Noncommercial 1.0.0
          </a>
          <span className="text-[12px] text-ink-faint">
            Conçu et développé par{" "}
            <a
              href={SITE_AUTEUR}
              target="_blank"
              rel="noopener noreferrer"
              className="font-semibold text-ink-muted hover:text-accent-text"
            >
              Alexandre Bouttier
            </a>
          </span>
        </div>
      </div>
    </footer>
  );
}
