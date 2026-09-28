/**
 * Canaux de relance proposés par l'interface.
 *
 * Le champ est un texte libre en base, sans contrainte `CHECK` : les lignes héritées
 * peuvent porter d'autres valeurs, que l'interface affiche telles quelles.
 */

export const CANAUX_FOLLOW_UP = ["Email", "Téléphone", "LinkedIn", "Autre"] as const;
