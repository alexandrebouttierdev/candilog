/**
 * Concatène des classes conditionnelles.
 *
 * Volontairement minimal, sans `tailwind-merge` : les composants de `shared/ui` exposent des
 * variantes typées plutôt qu'une classe libre, il n'y a donc pas de conflit d'utilitaires à
 * arbitrer. `className` sert au positionnement chez l'appelant (marges, largeur), pas à
 * repeindre le composant.
 */
export function cn(...parts: Array<string | false | null | undefined>): string {
  return parts.filter(Boolean).join(" ");
}
