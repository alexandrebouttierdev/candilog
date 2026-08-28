/** Rôles proposés par les maquettes ; le champ reste libre en base. */
export const ROLES = ["Recruteur", "Manager", "Référent", "Ancien collègue", "Autre"] as const;

/** Icône et teinte d'un rôle, dans la liste maître comme sur la fiche. */
export interface RoleMeta {
  readonly icon: string;
  readonly tone: "neutral" | "accent" | "success";
}

/**
 * Habillage d'un rôle de suivi.
 *
 * Les maquettes ne teintent que les deux rôles qui pèsent sur une candidature — le
 * recruteur en accent, le manager en succès ; tout le reste, y compris les rôles saisis
 * hors de la liste proposée, reste neutre.
 */
export function roleMeta(role: string | null | undefined): RoleMeta {
  if (role === "Recruteur") return { icon: "badge", tone: "accent" };
  if (role === "Manager") return { icon: "star", tone: "success" };
  return { icon: "person", tone: "neutral" };
}
