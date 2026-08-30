import { cn } from "@/lib/cn";

/**
 * Logo de marque monochrome (GitHub, Apple, Ubuntu, Fedora, Linux, fournisseurs IA).
 *
 * Le SVG sert de masque et la couleur vient de `currentColor` : le logo prend donc le
 * token de texte de son conteneur (`text-ink`, `text-ink-tertiary`, `text-on-accent`…)
 * et suit le thème tout seul. C'est ce que le prototype encodait en dur dans ses URL
 * `cdn.simpleicons.org/<marque>/<couleur>` — et ça évite le filtre d'inversion, donc
 * aucune variante conditionnelle n'est nécessaire.
 */
export function BrandIcon({
  name,
  size = 14,
  className,
  set = "brand",
}: {
  name: string;
  size?: number;
  className?: string;
  set?: "brand" | "providers";
}) {
  return (
    <span
      aria-hidden="true"
      className={cn("block shrink-0 bg-current", className)}
      style={{
        width: size,
        height: size,
        maskImage: `url(/${set}/${name}.svg)`,
        WebkitMaskImage: `url(/${set}/${name}.svg)`,
        maskRepeat: "no-repeat",
        WebkitMaskRepeat: "no-repeat",
        maskSize: "contain",
        WebkitMaskSize: "contain",
        maskPosition: "center",
        WebkitMaskPosition: "center",
      }}
    />
  );
}
