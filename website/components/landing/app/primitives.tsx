import type { ReactNode } from "react";

import { cn } from "@/lib/cn";
import type { Avatar as TeinteAvatar, Teinte, Ton } from "@/lib/data/demo";

/*
 * Primitives des aperçus : reproductions statiques des composants de l'application
 * (`src/shared/ui/`). Elles ne servent qu'à dessiner des captures — jamais de contrôle
 * réel ici : les fenêtres d'aperçu sont décoratives (`aria-hidden`), sans élément
 * focusable.
 */

/* Visibilité responsive : la classe `display` d'une primitive vit ici, pas chez
   l'appelant (pas de `tailwind-merge` : deux `display` concurrents seraient ambigus). */
export type Seuil = "sm" | "md" | "lg";

/* Glyphe de statut (`StatusGlyph`) : cercle vide, demi, trois quarts, plein. La forme
   porte l'information autant que la couleur. */
const GLYPHE: Record<Ton, string> = {
  n: "border-st-n",
  a: "border-st-a bg-[linear-gradient(90deg,var(--st-a)_50%,transparent_50%)]",
  g: "border-st-g bg-[conic-gradient(var(--st-g)_55%,transparent_55%)]",
  c: "border-st-c bg-st-c",
};

export function StatusGlyph({ ton, petit = false, className }: { ton: Ton; petit?: boolean; className?: string }) {
  return (
    <span
      className={cn(
        "inline-block flex-none rounded-full",
        petit ? "size-[9px] border-[1.5px]" : "size-[11px] border-[1.6px]",
        GLYPHE[ton],
        className,
      )}
    />
  );
}

/* Point de couleur : local (vert), distant (ambre), inactif. */
const POINT = {
  local: "bg-st-g",
  distant: "bg-st-a",
  aucun: "bg-tx-7",
} as const;

export function Point({ ou, className }: { ou: keyof typeof POINT; className?: string }) {
  return <span className={cn("inline-block size-[6px] flex-none rounded-full", POINT[ou], className)} />;
}

const AVATAR: Record<TeinteAvatar, string> = { av1: "bg-av1", av2: "bg-av2", av3: "bg-av3" };

const AFFICHAGE_AVATAR = {
  toujours: "inline-grid",
  "des-sm": "hidden sm:inline-grid",
  "sous-sm": "inline-grid sm:hidden",
} as const;

export function Avatar({
  initiales,
  teinte,
  petit = false,
  affichage = "toujours",
}: {
  initiales: string;
  teinte: TeinteAvatar;
  petit?: boolean;
  affichage?: keyof typeof AFFICHAGE_AVATAR;
}) {
  return (
    <span
      className={cn(
        "flex-none place-items-center rounded-full font-semibold text-av-tx",
        petit ? "size-[14px] text-[6px]" : "size-[18px] text-[7.5px]",
        AFFICHAGE_AVATAR[affichage],
        AVATAR[teinte],
      )}
    >
      {initiales}
    </span>
  );
}

const TEINTE: Record<Teinte | "a", string> = {
  ac: "bg-tint-ac-bg text-tint-ac-tx",
  g: "bg-tint-g-bg text-tint-g-tx",
  c: "bg-tint-c-bg text-tint-c-tx",
  a: "bg-chip text-st-a",
  neutre: "bg-chip text-tx-4",
};

/** Pastille (`Tag`) : contrat, échéance, score, état d'une exigence. */
export function Pastille({
  teinte = "neutre",
  mono = false,
  petit = false,
  children,
}: {
  teinte?: Teinte | "a";
  mono?: boolean;
  petit?: boolean;
  children: ReactNode;
}) {
  return (
    <span
      className={cn(
        "inline-flex flex-none items-center whitespace-nowrap rounded-r4 px-[5px] py-[3px] leading-none",
        petit ? "text-[9.5px]" : "text-[10.5px]",
        mono && "font-mono",
        TEINTE[teinte],
      )}
    >
      {children}
    </span>
  );
}

/** Touche imprimée (`Kbd`). */
export function Kbd({ children, accent = false, surAccent = false }: { children: ReactNode; accent?: boolean; surAccent?: boolean }) {
  return (
    <span
      className={cn(
        "inline-flex items-center rounded-r4 px-1 py-[3px] font-mono text-[9.5px] leading-none",
        accent ? "bg-ac text-on-accent" : surAccent ? "bg-on-accent/20 text-on-accent" : "bg-chip text-tx-4",
      )}
    >
      {children}
    </span>
  );
}

/** Bouton dessiné (non interactif) : primaire, secondaire, ou discret. */
const BOUTON = {
  primaire: "bg-ac text-on-accent",
  secondaire: "bg-elev text-tx-2",
  discret: "text-tx-4",
} as const;

const TAILLE_BOUTON = {
  petit: "h-5 px-2 text-[10.5px]",
  normal: "h-[24px] px-[9px] text-[11.5px]",
  dialogue: "h-7 px-3 text-[11.5px]",
} as const;

const AFFICHAGE_BOUTON: Record<Seuil | "toujours", string> = {
  toujours: "inline-flex",
  sm: "hidden sm:inline-flex",
  md: "hidden md:inline-flex",
  lg: "hidden lg:inline-flex",
};

export function FauxBouton({
  variante = "secondaire",
  taille = "normal",
  visible = "toujours",
  touche,
  children,
  className,
}: {
  variante?: keyof typeof BOUTON;
  taille?: keyof typeof TAILLE_BOUTON;
  /** Largeur à partir de laquelle le bouton apparaît. */
  visible?: Seuil | "toujours";
  touche?: string;
  children: ReactNode;
  /** Positionnement et contour seulement (`self-start`, anneau de focus dessiné). */
  className?: string;
}) {
  return (
    <span
      className={cn(
        "flex-none items-center gap-[6px] whitespace-nowrap rounded-r6 font-medium",
        AFFICHAGE_BOUTON[visible],
        TAILLE_BOUTON[taille],
        BOUTON[variante],
        className,
      )}
    >
      {children}
      {touche ? <Kbd surAccent={variante === "primaire"}>{touche}</Kbd> : null}
    </span>
  );
}

/** Interrupteur dessiné (`Switch`). */
export function FauxSwitch({ actif }: { actif: boolean }) {
  return (
    <span className={cn("relative h-[13px] w-[22px] flex-none rounded-full", actif ? "bg-ac" : "bg-chip")}>
      <span
        className={cn(
          "absolute top-[2px] size-[9px] rounded-full",
          actif ? "right-[2px] bg-on-accent" : "left-[2px] bg-tx-6",
        )}
      />
    </span>
  );
}

/** Contrôle segmenté dessiné (`SegmentedControl`). */
export function FauxSegment({ options, choisi }: { options: readonly string[]; choisi: string }) {
  return (
    <span className="flex rounded-r6 bg-elev p-[2px] text-[11px]">
      {options.map((option) => (
        <span
          key={option}
          className={cn(
            "flex-1 rounded-r4 px-1 py-[3px] text-center",
            option === choisi ? "bg-panel font-medium text-tx" : "text-tx-4",
          )}
        >
          {option}
        </span>
      ))}
    </span>
  );
}

/** Miniature de feuille : lignes grisées d'un document. */
export function FeuilleMiniature({ className }: { className?: string }) {
  return (
    <span
      className={cn(
        "flex flex-none flex-col gap-[2px] rounded-r3 border border-bd-menu bg-paper px-[4px] py-[5px]",
        className,
      )}
    >
      <span className="h-[1.5px] bg-tx-7" />
      <span className="h-[1.5px] w-[70%] bg-tx-7" />
      <span className="h-[1.5px] bg-tx-7" />
    </span>
  );
}

export function Filet({ className }: { className?: string }) {
  return <span className={cn("block h-px flex-none bg-bd-soft", className)} />;
}
