"use client";

import { useEffect, useId, useRef, useState } from "react";

import { ButtonLink } from "@/components/ui/Button";
import { LineIcon } from "@/components/ui/LineIcon";
import { cn } from "@/lib/cn";
import { GITHUB_REPO } from "@/lib/data/liens";
import { NAV_SECTIONS } from "@/lib/data/navigation";

/**
 * Menu des sections sous 1024 px. Disclosure : `aria-expanded` + `aria-controls`,
 * panneau fermé `inert` (ses liens ne sont pas atteignables au clavier), fermeture à
 * `Échap`, au clic extérieur et au choix d'une section.
 */
export function MobileNav() {
  const [ouvert, setOuvert] = useState(false);
  const id = useId();
  const racine = useRef<HTMLDivElement>(null);
  const declencheur = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    if (!ouvert) return;
    const surClic = (event: PointerEvent) => {
      if (!racine.current?.contains(event.target as Node)) setOuvert(false);
    };
    const surTouche = (event: KeyboardEvent) => {
      if (event.key !== "Escape") return;
      setOuvert(false);
      declencheur.current?.focus();
    };
    document.addEventListener("pointerdown", surClic);
    document.addEventListener("keydown", surTouche);
    return () => {
      document.removeEventListener("pointerdown", surClic);
      document.removeEventListener("keydown", surTouche);
    };
  }, [ouvert]);

  const panneauId = `${id}-menu`;

  return (
    <div ref={racine} className="lg:hidden">
      <button
        ref={declencheur}
        type="button"
        onClick={() => setOuvert(!ouvert)}
        aria-expanded={ouvert}
        aria-controls={panneauId}
        aria-label={ouvert ? "Fermer le menu" : "Ouvrir le menu"}
        className="grid size-[44px] place-items-center rounded-r7 text-tx transition-colors duration-[120ms] hover:bg-hover"
      >
        <LineIcon name={ouvert ? "close" : "menu"} size={18} />
      </button>

      <div
        id={panneauId}
        inert={!ouvert}
        className={cn(
          "absolute inset-x-0 top-full border-b border-bd bg-app px-4 pb-5 pt-2 shadow-pop md:px-8",
          "transition-[opacity,transform,visibility] duration-[200ms] ease-out-soft",
          ouvert ? "visible translate-y-0 opacity-100" : "invisible -translate-y-1 opacity-0",
        )}
      >
        <nav aria-label="Sections" className="flex flex-col">
          {NAV_SECTIONS.map(({ libelle, href }) => (
            <a
              key={href}
              href={`/${href}`}
              onClick={() => setOuvert(false)}
              className="flex min-h-[48px] items-center border-b border-bd-soft text-[16px] text-tx hover:text-ac-tx"
            >
              {libelle}
            </a>
          ))}
        </nav>
        <div className="mt-4 flex flex-col gap-2 sm:flex-row">
          <ButtonLink href="/#telecharger" taille="tactile" onClick={() => setOuvert(false)}>
            Télécharger
          </ButtonLink>
          <ButtonLink
            href={GITHUB_REPO}
            target="_blank"
            rel="noopener noreferrer"
            variante="secondaire"
            taille="tactile"
          >
            Code source sur GitHub
          </ButtonLink>
        </div>
      </div>
    </div>
  );
}
