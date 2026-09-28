import Link from "next/link";

import { BrandMark } from "@/components/ui/BrandMark";
import {
  GITHUB_DISCUSSIONS,
  GITHUB_ISSUES,
  GITHUB_REPO,
  LICENCE_POLYFORM,
  SITE_AUTEUR,
} from "@/lib/data/liens";
import { NAV_LEGALE } from "@/lib/data/navigation";

const PROJET = [
  { libelle: "Code source", href: GITHUB_REPO },
  { libelle: "Versions publiées", href: `${GITHUB_REPO}/releases` },
  { libelle: "Signaler un problème", href: GITHUB_ISSUES },
  { libelle: "Discussions", href: GITHUB_DISCUSSIONS },
] as const;

const LIEN =
  "inline-flex min-h-[44px] items-center text-[13.5px] text-tx-3 hover:text-tx md:min-h-[28px]";

/** Pied de page : marque et auteur, liens du projet, pages légales, licence. */
export function SiteFooter() {
  return (
    <footer className="border-t border-bd bg-app">
      <div className="mx-auto flex max-w-[1200px] flex-col gap-10 px-4 pb-12 pt-10 md:flex-row md:justify-between md:px-8 xl:px-0">
        <div className="flex flex-col gap-3">
          <span className="flex items-center gap-[9px]">
            <BrandMark size={20} />
            <span className="text-[14px] font-semibold">Candilog</span>
          </span>
          <p className="max-w-[320px] text-[13px] leading-[1.6] text-tx-4">
            Copyright © 2026 Alexandre Bouttier.{" "}
            <a href={LICENCE_POLYFORM} target="_blank" rel="noopener noreferrer" className="text-tx-3 hover:text-tx">
              Licence PolyForm Noncommercial 1.0.0
            </a>
            . Conçu et développé par{" "}
            <a href={SITE_AUTEUR} target="_blank" rel="noopener noreferrer" className="text-tx-3 hover:text-tx">
              Alexandre Bouttier
            </a>
            .
          </p>
        </div>

        <div className="grid grid-cols-2 gap-x-10 gap-y-2 md:flex md:gap-16">
          <nav aria-label="Projet" className="flex flex-col">
            <span className="caps mb-2 text-tx-4">Projet</span>
            {PROJET.map(({ libelle, href }) => (
              <a key={href} href={href} target="_blank" rel="noopener noreferrer" className={LIEN}>
                {libelle}
              </a>
            ))}
          </nav>
          <nav aria-label="Informations légales" className="flex flex-col">
            <span className="caps mb-2 text-tx-4">Légal</span>
            {NAV_LEGALE.map(({ libelle, href }) => (
              <Link key={href} href={href} className={LIEN}>
                {libelle}
              </Link>
            ))}
          </nav>
        </div>
      </div>
    </footer>
  );
}
