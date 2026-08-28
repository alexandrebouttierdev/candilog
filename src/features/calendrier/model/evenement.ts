import type { Entretien } from "@/features/entretiens/services/entretien.service";
import type { Relance } from "@/features/relances/services/relance.service";
import { iconeEntretien } from "@/features/entretiens/model/types";
import { iconeRelance } from "@/features/relances/model/types";
import { heureDepuisHorodatage, jourDe } from "@/shared/lib/dates";
import type { Tone } from "@/shared/ui";

/**
 * Événement du calendrier, entretien ou relance ramenés à une forme commune.
 *
 * La grille n'a pas à connaître deux entités : elle affiche des pastilles datées. Le type
 * d'origine reste accessible pour rouvrir la bonne modale au clic.
 */
export interface EvenementCalendrier {
  readonly id: string;
  readonly genre: "entretien" | "relance";
  /** Jour `AAAA-MM-JJ` de regroupement. */
  readonly jour: string;
  /** Heure `HH:MM`, absente pour une relance qui se programme au jour. */
  readonly heure: string | null;
  readonly libelle: string;
  readonly detail: string | null;
  readonly icone: string;
  readonly tone: Tone;
}

/** Convertit un entretien en événement. Tonalité verte : c'est un avancement. */
export function depuisEntretien(entretien: Entretien): EvenementCalendrier {
  return {
    id: entretien.id,
    genre: "entretien",
    jour: jourDe(entretien.dateEntretien),
    heure: heureDepuisHorodatage(entretien.dateEntretien),
    libelle: entretien.candidaturePoste ?? "Entretien",
    detail: entretien.entrepriseNom,
    icone: iconeEntretien(entretien.type),
    tone: "success",
  };
}

/** Convertit une relance en événement. Tonalité ambre : c'est une action à mener. */
export function depuisRelance(relance: Relance): EvenementCalendrier {
  return {
    id: relance.id,
    genre: "relance",
    jour: jourDe(relance.dateRelance),
    heure: null,
    libelle: relance.candidaturePoste ?? "Relance",
    detail: relance.entrepriseNom,
    icone: iconeRelance(relance.type),
    tone: "warning",
  };
}

/**
 * Regroupe les événements par jour, chaque journée triée par heure.
 *
 * Les relances, sans heure, passent en tête de journée : elles se traitent quand on veut,
 * là où un entretien a un créneau.
 */
export function grouperParJour(
  evenements: readonly EvenementCalendrier[],
): Map<string, EvenementCalendrier[]> {
  const parJour = new Map<string, EvenementCalendrier[]>();
  for (const evenement of evenements) {
    const journee = parJour.get(evenement.jour);
    if (journee) {
      journee.push(evenement);
    } else {
      parJour.set(evenement.jour, [evenement]);
    }
  }
  for (const journee of parJour.values()) {
    journee.sort((a, b) => (a.heure ?? "").localeCompare(b.heure ?? ""));
  }
  return parJour;
}
