import { z } from "zod";
import { texteFacultatif } from "@/shared/lib/zod-helpers";
import { FORMAT_DATE, heureValide, versDateIso, versHorodatage } from "@/shared/lib/dates";

/**
 * Formulaire entretien, création et modification.
 *
 * Les maquettes séparent « Date » et « Heure » en deux champs, là où la base ne porte qu'un
 * horodatage : la composition est faite ici, à la validation, pour que le ViewModel et le
 * backend ne manipulent qu'un `RFC 3339`.
 */
export const entretienFormSchema = z
  .object({
    candidatureId: z.string().min(1, "La candidature concernée est obligatoire"),
    contactId: z
      .string()
      .transform((valeur) => (valeur === "" ? null : valeur))
      .nullable()
      .default(null),
    date: z
      .string()
      .trim()
      .min(1, "La date est obligatoire")
      .refine((valeur) => versDateIso(valeur) !== null, {
        message: `Date invalide — format attendu ${FORMAT_DATE}.`,
      }),
    heure: z
      .string()
      .trim()
      .min(1, "L'heure est obligatoire")
      .refine(heureValide, { message: "Heure invalide — format attendu HH:MM." }),
    type: z.enum(["Présentiel", "Visio", "Téléphonique", "Technique", "RH", "Autre"]),
    lieu: texteFacultatif,
    notes: texteFacultatif,
    compteRendu: texteFacultatif,
  })
  .transform((valeurs) => ({
    candidatureId: valeurs.candidatureId,
    contactId: valeurs.contactId,
    // `versHorodatage` a déjà été éprouvé par les deux `refine` ci-dessus : la date et
    // l'heure sont valides, la composition ne peut plus échouer.
    dateEntretien: versHorodatage(valeurs.date, valeurs.heure) as string,
    type: valeurs.type,
    lieu: valeurs.lieu,
    notes: valeurs.notes,
    compteRendu: valeurs.compteRendu,
  }));

/** Valeurs validées, telles qu'envoyées au backend. */
export type EntretienFormValues = z.output<typeof entretienFormSchema>;

/** Valeurs saisies, avant transformation — ce que manipule React Hook Form. */
export type EntretienFormInput = z.input<typeof entretienFormSchema>;
