import Link from "next/link";
import type { ComponentProps, ReactNode } from "react";

import { cn } from "@/lib/cn";

/* Hauteurs de contrôle : 32 px compact (en-tête), 40 px principal (hero, bloc de
   téléchargement), 48 px tactile (mobile, cible ≥ 44 px). */
const TAILLES = {
  compact: "h-[32px] px-3 text-[13px] gap-[7px]",
  principal: "h-[40px] px-4 text-[14px] gap-2",
  tactile: "h-[48px] px-[18px] text-[15px] gap-2",
} as const;

/* Primaire = l'accent de l'application ; secondaire = contrôle sur panneau. */
const VARIANTES = {
  accent: "border border-transparent bg-ac text-on-accent hover:bg-ac-h hover:text-on-accent",
  secondaire:
    "border border-bd-menu bg-panel text-tx hover:border-bd-strong hover:bg-hover hover:text-tx",
} as const;

/* La classe `display` vit ici : un `hidden` passé par l'appelant entrerait en conflit
   avec `inline-flex` (pas de `tailwind-merge`). */
const AFFICHAGE = {
  toujours: "inline-flex",
  sm: "hidden sm:inline-flex",
  md: "hidden md:inline-flex",
} as const;

export type Variante = keyof typeof VARIANTES;
export type Taille = keyof typeof TAILLES;

function classes(variante: Variante, taille: Taille, visible: keyof typeof AFFICHAGE, className?: string) {
  return cn(
    AFFICHAGE[visible],
    "shrink-0 items-center justify-center whitespace-nowrap rounded-r7 font-medium transition-colors duration-[120ms]",
    TAILLES[taille],
    VARIANTES[variante],
    className,
  );
}

type BaseProps = {
  children: ReactNode;
  variante?: Variante;
  taille?: Taille;
  /** Largeur à partir de laquelle le bouton apparaît. */
  visible?: keyof typeof AFFICHAGE;
  className?: string;
};

/** Lien stylé en bouton — le cas courant sur cette landing (ancres et liens sortants). */
export function ButtonLink({
  children,
  variante = "accent",
  taille = "principal",
  visible = "toujours",
  className,
  href,
  ...rest
}: BaseProps & Omit<ComponentProps<typeof Link>, "className" | "children">) {
  return (
    <Link href={href} className={classes(variante, taille, visible, className)} {...rest}>
      {children}
    </Link>
  );
}

/** Vrai bouton — pour les contrôles qui agissent sur la page (menus, onglets). */
export function Button({
  children,
  variante = "accent",
  taille = "principal",
  visible = "toujours",
  className,
  type = "button",
  ...rest
}: BaseProps & Omit<ComponentProps<"button">, "className" | "children">) {
  return (
    <button type={type} className={classes(variante, taille, visible, className)} {...rest}>
      {children}
    </button>
  );
}
