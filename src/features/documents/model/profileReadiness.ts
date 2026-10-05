import type { Profile } from "@/shared/types/generated/profile";

/**
 * Ce qui manque au profil pour qu'une génération ait de la matière, nommé comme
 * l'utilisateur le lit à l'écran. Vide quand il n'y a rien à dire.
 */
export type ProfileGaps = readonly string[];

/**
 * Un générateur n'invente pas un parcours : il reformule le profil. Sans nom, le document
 * sort sans en-tête — c'est ce qui faisait échouer le banc A4 des lettres. Sans aucune
 * expérience, formation ni compétence, le modèle n'a pas un fait à citer et comble les
 * trous tout seul.
 *
 * Les deux cas se disent **avant** la génération plutôt que de se découvrir sur la feuille.
 * Ce n'est pas une erreur et ça ne bloque rien : l'utilisateur peut vouloir essayer.
 *
 * Seules les sections dont les générateurs se servent vraiment comptent (`profileSections`) ;
 * les langues, projets, certifications et centres d'intérêt enrichissent un CV mais ne
 * suffisent pas à en écrire un.
 */
export function profileGaps(profile: Profile): ProfileGaps {
  const gaps: string[] = [];
  const { first_name, name } = profile.identity;
  if (!first_name.trim() || !name.trim()) gaps.push("votre nom");
  if (profile.experiences.length + profile.education.length + profile.skills.length === 0) {
    gaps.push("au moins une expérience, une formation ou une compétence");
  }
  return gaps;
}

/** Énumération française : « a », « a et b », « a, b et c ». */
export function enumerate(items: ProfileGaps): string {
  if (items.length <= 1) return items[0] ?? "";
  return `${items.slice(0, -1).join(", ")} et ${items[items.length - 1]!}`;
}
