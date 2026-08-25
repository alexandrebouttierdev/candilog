import { z } from "zod";
import { texteFacultatif, urlFacultative } from "@/shared/lib/zod-helpers";

/** Format de date affiché et saisi, conformément aux maquettes. */
export const FORMAT_DATE_AFFICHE = "JJ-MM-AAAA";

/**
 * Convertit une date `JJ-MM-AAAA` en `AAAA-MM-JJ`, format attendu par le backend.
 *
 * Les filtres de période comparent des chaînes en base (`date_envoi >= ?`) : une date dans
 * un autre format s'y comparerait dans le désordre et disparaîtrait silencieusement des
 * résultats. La conversion est donc faite ici, une fois, plutôt qu'à chaque appel.
 */
export function versDateIso(saisie: string): string | null {
  const correspondance = /^(\d{2})-(\d{2})-(\d{4})$/.exec(saisie.trim());
  if (!correspondance) return null;
  const [, jour, mois, annee] = correspondance;
  const iso = `${annee}-${mois}-${jour}`;
  // `Date` accepte le 31 février en le décalant au 3 mars : comparer la valeur relue est le
  // seul moyen de refuser une date qui n'existe pas.
  const date = new Date(`${iso}T00:00:00Z`);
  return Number.isNaN(date.getTime()) || date.toISOString().slice(0, 10) !== iso ? null : iso;
}

/** Convertit une date `AAAA-MM-JJ` en `JJ-MM-AAAA` pour l'affichage. */
export function versDateAffichee(iso: string): string {
  const correspondance = /^(\d{4})-(\d{2})-(\d{2})/.exec(iso);
  if (!correspondance) return iso;
  const [, annee, mois, jour] = correspondance;
  return `${jour}-${mois}-${annee}`;
}

/**
 * Formulaire candidature, création et modification.
 *
 * Reprend les règles de `CandidatureService::valider` côté Rust : poste requis, date
 * valide, lien limité à HTTP(S). L'entreprise est requise ici **et** en base par une clé
 * étrangère `NOT NULL` : une candidature sans entreprise n'a pas de sens métier.
 */
export const candidatureFormSchema = z.object({
  poste: z.string().trim().min(1, "Le poste est obligatoire"),
  entrepriseId: z.string().min(1, "L'entreprise est obligatoire"),
  typeContrat: z.enum(["CDI", "CDD", "Freelance", "Stage", "Alternance", "Interim", "Autre"]),
  statut: z.enum(["EN_ATTENTE", "RELANCEE", "ENTRETIEN", "REFUS"]),
  dateEnvoi: z
    .string()
    .trim()
    .min(1, "La date d'envoi est obligatoire")
    .refine((valeur) => versDateIso(valeur) !== null, {
      message: `Date invalide — format attendu ${FORMAT_DATE_AFFICHE}.`,
    })
    // Transformée dès la validation : le ViewModel et le backend ne manipulent que de l'ISO.
    .transform((valeur) => versDateIso(valeur) as string),
  lienOffre: urlFacultative("Le lien de l'offre doit commencer par http:// ou https://"),
  notes: texteFacultatif,
});

/** Valeurs validées, telles qu'envoyées au backend. */
export type CandidatureFormValues = z.output<typeof candidatureFormSchema>;

/** Valeurs saisies, avant transformation — ce que manipule React Hook Form. */
export type CandidatureFormInput = z.input<typeof candidatureFormSchema>;
