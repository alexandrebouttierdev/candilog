import { cn } from "@/shared/lib/cn";

/**
 * Tonalité d'un glyphe : neutre, attention, positif, critique (`st-n/a/g/c` du design).
 *
 * Les quatre statuts de candidature s'y projettent (En attente, Relancée, Entretien,
 * Refusée), mais le glyphe sert aussi aux recommandations, aux étapes et aux dialogues.
 */
export type GlyphTone = "n" | "a" | "g" | "c";

const COLOR: Record<GlyphTone, string> = {
  n: "var(--st-n)",
  a: "var(--st-a)",
  g: "var(--st-g)",
  c: "var(--st-c)",
};

/**
 * Remplissage par tonalité : cercle vide, demi-disque, trois quarts, disque plein
 * (`DECISIONS.md` B12). La forme porte l'information autant que la couleur : un statut
 * reste lisible en niveaux de gris ou pour une deutéranopie.
 */
function fill(tone: GlyphTone): string {
  const color = COLOR[tone];
  switch (tone) {
    case "n":
      return "transparent";
    case "a":
      return `linear-gradient(90deg, ${color} 50%, transparent 50%)`;
    case "g":
      return `conic-gradient(${color} 55%, transparent 55%)`;
    case "c":
      return color;
  }
}

/**
 * Glyphe de statut, signature du produit : cercle de 11 px, contour 1,6 px (9 px et 1,5 px
 * en petit). Ne jamais le remplacer par une icône.
 *
 * Décoratif par défaut : le libellé voisin porte l'information. Passer `label` quand le
 * glyphe est seul (bouton de changement de statut, par exemple).
 */
export function StatusGlyph({
  tone,
  small = false,
  label,
  className,
}: {
  tone: GlyphTone;
  small?: boolean;
  label?: string;
  className?: string;
}) {
  return (
    <span
      {...(label ? { role: "img", "aria-label": label } : { "aria-hidden": true })}
      data-tone={tone}
      className={cn("inline-block flex-none rounded-full", small ? "size-[9px]" : "size-[11px]", className)}
      style={{
        border: `${small ? 1.5 : 1.6}px solid ${COLOR[tone]}`,
        background: fill(tone),
      }}
    />
  );
}
