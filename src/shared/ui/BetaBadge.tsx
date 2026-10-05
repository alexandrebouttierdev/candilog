import { cn } from "@/shared/lib/cn";
import { LineIcon } from "./LineIcon";

/**
 * Badge « bêta » (`docs/DESIGN.md` §7) : marque une fonctionnalité dont le résultat n'est
 * pas encore fiable — tout ce qui passe par l'IA.
 *
 * Ambre, la teinte du « à traiter » : ni une erreur, ni un état normal. L'éprouvette et le
 * mot vont ensemble, comme toute pastille du design — la couleur seule ne dirait rien à qui
 * ne la voit pas.
 *
 * Le `title` porte la conséquence pratique plutôt qu'une définition de « bêta » : ce que
 * l'utilisateur doit faire, c'est relire.
 */
export function BetaBadge({ size = "default", className }: { size?: "default" | "nav"; className?: string }) {
  return (
    <span
      title="Fonctionnalité en bêta : relisez toujours ce que l'IA produit."
      className={cn(
        "inline-flex flex-none items-center rounded-r5 bg-warning-tint text-st-a",
        size === "nav" ? "h-[17px] gap-[3px] px-1" : "h-5 gap-1 px-1.5",
        className,
      )}
    >
      <LineIcon name="beta" size={size === "nav" ? 10 : 11} />
      <span className={cn("font-medium", size === "nav" ? "text-mono-sm" : "text-caps")}>bêta</span>
    </span>
  );
}
