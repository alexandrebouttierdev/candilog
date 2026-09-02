/**
 * Canaux de relance proposés par l'interface.
 *
 * Le champ est un texte libre en base, sans contrainte `CHECK` : les lignes héritées
 * peuvent porter d'autres valeurs, que l'interface affiche telles quelles.
 */

import type { IconName } from "@/shared/ui/icon-names";

export const CANAUX_FOLLOW_UP = ["Email", "Téléphone", "LinkedIn", "Autre"] as const;

/** Icône associée à un canal de relance. */
export function followUpIcon(channel: string): IconName {
  switch (channel) {
    case "Email":
      return "mail";
    case "Téléphone":
      return "call";
    case "LinkedIn":
      return "link";
    default:
      return "send";
  }
}
