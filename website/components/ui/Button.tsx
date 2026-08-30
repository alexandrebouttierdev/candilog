import Link from "next/link";
import type { ComponentProps, ReactNode } from "react";

import { cn } from "@/lib/cn";

/* Les trois hauteurs de contrôle du §5 : 30px compact (en-tête), 38px principal
   (hero, bloc de téléchargement). Les paddings asymétriques viennent du design. */
const TAILLES = {
  compact: "h-[30px] text-[12.5px] gap-[7px]",
  principal: "h-[38px] text-[13.5px] gap-2",
} as const;

const VARIANTES = {
  accent:
    "border border-accent-strong bg-accent text-on-accent hover:bg-accent-strong",
  secondaire:
    "border border-control bg-surface text-ink hover:border-control-strong hover:bg-surface-alt",
} as const;

/* Le design ne met pas le même padding horizontal sur les deux variantes :
   l'accent compact est à 13px, le secondaire compact à 12px. */
const PADDINGS = {
  "accent-compact": "px-[13px]",
  "accent-principal": "px-[15px]",
  "secondaire-compact": "px-[12px]",
  "secondaire-principal": "px-[15px]",
} as const;

export type Variante = keyof typeof VARIANTES;
export type Taille = keyof typeof TAILLES;

function classes(variante: Variante, taille: Taille, className?: string) {
  return cn(
    "inline-flex shrink-0 items-center justify-center whitespace-nowrap rounded-control font-semibold transition-colors duration-[120ms]",
    TAILLES[taille],
    VARIANTES[variante],
    PADDINGS[`${variante}-${taille}`],
    className,
  );
}

type BaseProps = {
  children: ReactNode;
  variante?: Variante;
  taille?: Taille;
  className?: string;
};

/** Lien stylé en bouton — le cas courant sur cette landing (ancres et liens sortants). */
export function ButtonLink({
  children,
  variante = "accent",
  taille = "principal",
  className,
  href,
  ...rest
}: BaseProps & Omit<ComponentProps<typeof Link>, "className" | "children">) {
  return (
    <Link href={href} className={classes(variante, taille, className)} {...rest}>
      {children}
    </Link>
  );
}

/** Vrai bouton — pour les contrôles qui agissent sur la page (menus, onglets). */
export function Button({
  children,
  variante = "accent",
  taille = "principal",
  className,
  type = "button",
  ...rest
}: BaseProps & Omit<ComponentProps<"button">, "className" | "children">) {
  return (
    <button type={type} className={classes(variante, taille, className)} {...rest}>
      {children}
    </button>
  );
}
