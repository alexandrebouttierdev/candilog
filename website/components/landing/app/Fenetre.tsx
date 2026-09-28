import type { ReactNode } from "react";

import { Kbd } from "@/components/landing/app/primitives";
import { BrandMark } from "@/components/ui/BrandMark";
import { LineIcon, type LineIconName } from "@/components/ui/LineIcon";
import { cn } from "@/lib/cn";

/*
 * Coque de l'application v2 reproduite pour les aperçus (`src/app/layout/`) :
 * barre de titre 40 px, navigation 202 px (52 px sous 1060 px dans l'application, ici
 * sous 1024 px), panneau de travail à rayon 9, barre d'état 34 px.
 *
 * Sous 768 px la navigation disparaît : l'aperçu devient un recadrage de l'écran, pas
 * une fenêtre réduite illisible.
 */

/** Cadre de fenêtre. `hauteur` n'est appliquée qu'à partir de 768 px : en dessous, la
 *  fenêtre prend la hauteur de son contenu recomposé. */
export function Fenetre({
  children,
  hauteur,
  className,
}: {
  children: ReactNode;
  hauteur?: string;
  className?: string;
}) {
  return (
    <div
      className={cn(
        "flex flex-col overflow-hidden rounded-r12 border border-frame bg-app shadow-window",
        hauteur,
        className,
      )}
    >
      {children}
    </div>
  );
}

/** Barre de titre : feux macOS, fil d'Ariane, accessoires à droite. Une surcouche
 *  plein écran (générateur, analyse) remplace les feux par son bouton « ✕ ». */
export function BarreTitre({
  fil,
  droite,
  surcouche = false,
}: {
  fil: readonly [string, string];
  droite?: ReactNode;
  surcouche?: boolean;
}) {
  return (
    <div className="flex h-[40px] flex-none items-center gap-3 px-3 md:gap-[22px] md:px-[14px]">
      {surcouche ? (
        <span className="grid size-[22px] flex-none place-items-center rounded-r6 bg-elev text-[10px] text-tx-3">
          ✕
        </span>
      ) : (
        <span className="hidden flex-none gap-2 md:flex">
          <span className="size-3 rounded-full bg-wc1" />
          <span className="size-3 rounded-full bg-wc2" />
          <span className="size-3 rounded-full bg-wc3" />
        </span>
      )}
      <span className={cn("flex min-w-0 gap-[7px] text-[12.5px]", !surcouche && "md:pl-3")}>
        <span className="truncate text-tx-4">{fil[0]}</span>
        <span className="text-tx-6">›</span>
        <span className="truncate font-medium text-tx">{fil[1]}</span>
      </span>
      <span className="flex-1" />
      {droite}
    </div>
  );
}

/** Onglets de vue de la barre de titre (Liste, Kanban, Calendrier, Analyse). */
export function OngletsVue({ actif }: { actif: "Liste" | "Kanban" | "Analyse" }) {
  return (
    <span className="hidden gap-[2px] text-[12px] sm:flex">
      {(["Liste", "Kanban", "Calendrier", "Analyse"] as const).map((vue) => (
        <span
          key={vue}
          className={cn("rounded-r6 px-2 py-1", vue === actif ? "bg-elev font-medium text-tx" : "text-tx-3")}
        >
          {vue}
        </span>
      ))}
    </span>
  );
}

export function Accessoire({ children }: { children: ReactNode }) {
  return (
    <span className="hidden flex-none rounded-r6 bg-elev px-2 py-1 text-[11.5px] text-tx-4 sm:inline">
      {children}
    </span>
  );
}

type Destination = "today" | "applications" | "relations" | "documents" | "ai" | "profile";

const DESTINATIONS: ReadonlyArray<{
  cle: Destination;
  libelle: string;
  compte: string;
  ton: "ac" | "a" | "neutre";
}> = [
  { cle: "today", libelle: "Aujourd'hui", compte: "3", ton: "ac" },
  { cle: "applications", libelle: "Candidatures", compte: "14", ton: "neutre" },
  { cle: "relations", libelle: "Relations", compte: "18", ton: "neutre" },
  { cle: "documents", libelle: "Documents", compte: "6", ton: "neutre" },
  { cle: "ai", libelle: "Intelligence artificielle", compte: "", ton: "neutre" },
  { cle: "profile", libelle: "Profil", compte: "92 %", ton: "a" },
];

const VUES = [
  { libelle: "À relancer", compte: "3", ton: "c" },
  { libelle: "Entretiens à venir", compte: "2", ton: "neutre" },
  { libelle: "Spontanées 2026", compte: "4", ton: "neutre" },
] as const;

const COMPTE = { ac: "text-ac-tx", a: "text-st-a", c: "text-st-c", neutre: "text-tx-6" } as const;

/* Seuil à partir duquel la navigation s'affiche en entier (202 px) ; en dessous et dès
   768 px, elle se replie en icônes (52 px), comme l'application sous 1060 px. Les
   classes sont écrites en toutes lettres pour que Tailwind les détecte. */
const DEPLIE = {
  lg: {
    largeur: "lg:w-[202px]",
    texte: "hidden lg:inline",
    bloc: "hidden lg:block",
    flex: "hidden lg:flex",
    aligne: "lg:justify-start",
  },
  xl: {
    largeur: "xl:w-[202px]",
    texte: "hidden xl:inline",
    bloc: "hidden xl:block",
    flex: "hidden xl:flex",
    aligne: "xl:justify-start",
  },
} as const;

type Seuil = keyof typeof DEPLIE;

function EntreeNav({
  icone,
  libelle,
  compte,
  ton,
  active = false,
  seuil,
}: {
  icone: LineIconName;
  libelle: string;
  compte?: string;
  ton?: keyof typeof COMPTE;
  active?: boolean;
  seuil: Seuil;
}) {
  const d = DEPLIE[seuil];
  return (
    <span
      className={cn(
        "flex h-[26px] items-center justify-center gap-[9px] rounded-r6 px-2 text-[12.5px]",
        d.aligne,
        active ? "bg-elev font-medium text-tx" : "text-tx-3",
      )}
    >
      <LineIcon name={icone} size={15} />
      <span className={cn("flex-1 truncate", d.texte)}>{libelle}</span>
      {compte ? <span className={cn("font-mono text-[10.5px]", d.texte, COMPTE[ton ?? "neutre"])}>{compte}</span> : null}
    </span>
  );
}

/** Navigation latérale : 202 px dès `seuil` (1024 px par défaut), 52 px (icônes seules)
 *  entre 768 px et ce seuil, absente en dessous. */
export function Navigation({
  active,
  pied,
  seuil = "lg",
}: {
  active: Destination;
  pied?: ReactNode;
  seuil?: Seuil;
}) {
  const d = DEPLIE[seuil];
  return (
    <div className={cn("hidden w-[52px] flex-none flex-col gap-[2px] px-2 pb-[10px] pt-1 md:flex", d.largeur)}>
      <span className={cn("mb-2 flex h-[30px] items-center justify-center gap-2 px-2", d.aligne)}>
        <BrandMark size={19} />
        <span className={cn("flex-1 text-[13px] font-semibold", d.texte)}>Candilog</span>
        <span className={d.texte}>
          <Kbd>⌘K</Kbd>
        </span>
      </span>
      {DESTINATIONS.map((dest) => (
        <EntreeNav
          key={dest.cle}
          icone={dest.cle}
          libelle={dest.libelle}
          compte={dest.compte}
          ton={dest.ton}
          active={dest.cle === active}
          seuil={seuil}
        />
      ))}
      <span className={cn("caps px-2 pb-[6px] pt-4", d.bloc)}>Vues</span>
      <span className={cn("flex-col gap-[2px]", d.flex)}>
        {VUES.map((v) => (
          <EntreeNav key={v.libelle} icone="bookmark" libelle={v.libelle} compte={v.compte} ton={v.ton} seuil={seuil} />
        ))}
      </span>
      <span className="flex-1" />
      <EntreeNav icone="settings" libelle="Réglages" compte="⌘," seuil={seuil} />
      {pied ?? (
        <span className={cn("h-[26px] items-center gap-2 px-3 text-[11.5px] text-tx-3", d.flex)}>
          <span className="size-[6px] rounded-full bg-st-g" />
          <span className="flex-1">IA locale</span>
          <span className="font-mono text-[10px] text-tx-6">ministral 3B</span>
        </span>
      )}
    </div>
  );
}

/** Panneau de travail, à droite de la navigation. */
export function Panneau({ children, className }: { children: ReactNode; className?: string }) {
  return (
    <div
      className={cn(
        "flex min-w-0 flex-1 bg-panel md:rounded-tl-r9 md:border-l md:border-t md:border-bd",
        "border-t border-bd",
        className,
      )}
    >
      {children}
    </div>
  );
}

/** Barre d'état : décompte à gauche, contrat clavier à droite (masqué sur mobile). */
export function BarreEtat({
  gauche,
  touches,
}: {
  gauche: string;
  touches: ReadonlyArray<{ libelle: string; touche: string; accent?: boolean }>;
}) {
  return (
    <div className="flex h-[34px] flex-none items-center justify-between gap-4 border-t border-bd px-[14px]">
      <span className="truncate font-mono text-[10.5px] text-tx-5">{gauche}</span>
      <span className="hidden flex-none items-center gap-[14px] text-[11.5px] text-tx-3 sm:flex">
        {touches.map(({ libelle, touche, accent }) => (
          <span key={libelle} className="flex items-center gap-[6px]">
            {libelle} <Kbd accent={accent}>{touche}</Kbd>
          </span>
        ))}
      </span>
    </div>
  );
}
