"use client";

import { useEffect, useId, useRef, useSyncExternalStore } from "react";

import { Button } from "@/components/ui/Button";
import { BrandIcon } from "@/components/ui/BrandIcon";
import { LineIcon } from "@/components/ui/LineIcon";
import { cn } from "@/lib/cn";
import { PLATEFORMES, type Plateforme } from "@/lib/data/plateformes";
import {
  abonnerMenu,
  lireMenuOuvert,
  lireMenuOuvertServeur,
  ouvrirMenu,
} from "@/lib/menuOuvert";

/** Le carré à quatre carreaux de Windows, dessiné en CSS comme dans le prototype :
 *  simple-icons n'a pas de glyphe Windows utilisable ici. */
function LogoWindows() {
  return (
    <span aria-hidden="true" className="grid size-[13px] shrink-0 grid-cols-2 grid-rows-2 gap-[1.5px]">
      <span className="bg-tx-4" />
      <span className="bg-tx-4" />
      <span className="bg-tx-4" />
      <span className="bg-tx-4" />
    </span>
  );
}

function LignePlateforme({ plateforme, derniere }: { plateforme: Plateforme; derniere: boolean }) {
  return (
    <a
      href={plateforme.href}
      target="_blank"
      rel="noopener noreferrer"
      className={cn(
        "flex min-h-[44px] items-center gap-[10px] px-3 py-[9px] text-tx transition-colors duration-[120ms] hover:bg-hover hover:text-tx",
        !derniere && "border-b border-bd-soft",
      )}
    >
      {plateforme.logo === "windows" ? (
        <LogoWindows />
      ) : (
        <span className="block text-tx-4">
          <BrandIcon name={plateforme.logo} size={13} />
        </span>
      )}
      <span className="text-[13px] font-medium">{plateforme.libelle}</span>
      <span className="ml-auto font-mono text-[10.5px] text-tx-4">{plateforme.extension}</span>
    </a>
  );
}

export function DownloadMenu({
  libelle = "Télécharger Candilog",
  taille = "principal",
  alignement = "gauche",
}: {
  libelle?: string;
  taille?: "principal" | "tactile";
  alignement?: "gauche" | "droite";
}) {
  const id = useId();
  const panneauId = `${id}-panneau`;
  const racine = useRef<HTMLDivElement>(null);
  const declencheur = useRef<HTMLButtonElement>(null);

  const ouvert =
    useSyncExternalStore(abonnerMenu, lireMenuOuvert, lireMenuOuvertServeur) === id;

  // Fermeture au clic extérieur et à Échap (§7.11). Le focus revient au déclencheur
  // quand c'est le clavier qui ferme, sinon on le laisse là où le pointeur l'a mis.
  useEffect(() => {
    if (!ouvert) return;

    const surClicExterieur = (event: PointerEvent) => {
      if (!racine.current?.contains(event.target as Node)) ouvrirMenu(null);
    };
    const surTouche = (event: KeyboardEvent) => {
      if (event.key !== "Escape") return;
      ouvrirMenu(null);
      declencheur.current?.focus();
    };

    document.addEventListener("pointerdown", surClicExterieur);
    document.addEventListener("keydown", surTouche);
    return () => {
      document.removeEventListener("pointerdown", surClicExterieur);
      document.removeEventListener("keydown", surTouche);
    };
  }, [ouvert]);

  const groupes = PLATEFORMES.reduce<Array<{ groupe: string; lignes: Plateforme[] }>>(
    (acc, plateforme) => {
      const dernier = acc.at(-1);
      if (dernier && dernier.groupe === plateforme.groupe) dernier.lignes.push(plateforme);
      else acc.push({ groupe: plateforme.groupe, lignes: [plateforme] });
      return acc;
    },
    [],
  );

  return (
    <div
      ref={racine}
      className={cn("relative w-full shrink-0 sm:w-auto", ouvert && "z-50")}
    >
      <Button
        ref={declencheur}
        onClick={() => ouvrirMenu(ouvert ? null : id)}
        aria-expanded={ouvert}
        aria-controls={panneauId}
        taille={taille}
        className="w-full max-sm:h-[48px] max-sm:text-[15px] sm:w-auto"
      >
        <LineIcon name="export-csv" size={14} />
        {libelle}
        <span
          className="ml-[2px] block transition-transform duration-[160ms] ease-out-soft"
          style={{ transform: ouvert ? "rotate(180deg)" : "rotate(0deg)" }}
        >
          <LineIcon name="chevron-down" size={14} />
        </span>
      </Button>

      <div
        id={panneauId}
        inert={!ouvert}
        className={cn(
          "absolute top-[calc(100%+8px)] z-30 w-[min(298px,calc(100vw-32px))] overflow-hidden rounded-r10 border border-bd-menu bg-modal shadow-pop",
          alignement === "droite" ? "right-0 origin-top-right" : "left-0 origin-top-left",
          "transition-[opacity,transform,visibility] duration-[170ms] ease-out-soft",
          ouvert
            ? "visible scale-100 opacity-100 translate-y-0"
            : "invisible scale-[0.98] opacity-0 -translate-y-[6px]",
        )}
      >
        {groupes.map(({ groupe, lignes }, indexGroupe) => (
          <div key={groupe + String(indexGroupe)}>
            <div className="caps border-b border-bd-soft px-3 pb-[7px] pt-[9px] text-tx-5">
              {groupe}
            </div>
            {lignes.map((plateforme, index) => (
              <LignePlateforme
                key={plateforme.libelle}
                plateforme={plateforme}
                derniere={
                  indexGroupe === groupes.length - 1 && index === lignes.length - 1
                }
              />
            ))}
          </div>
        ))}
      </div>
    </div>
  );
}
