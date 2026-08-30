/**
 * Concatène des classes conditionnelles.
 *
 * Reprise de `shared/lib/cn.ts` de l'application desktop, avec le même parti pris :
 * volontairement minimal, sans `tailwind-merge`. Les composants exposent des variantes
 * typées (voir les objets STATUS, VARIANTS…) plutôt qu'une classe libre, il n'y a donc
 * pas de conflit d'utilitaires à arbitrer. `className` sert au positionnement chez
 * l'appelant (marges, largeur), pas à repeindre le composant.
 */
export function cn(...parts: Array<string | false | null | undefined>): string {
  return parts.filter(Boolean).join(" ");
}
