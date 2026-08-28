import { z } from "zod";
import { textFacultatif } from "@/shared/lib/zod-helpers";
import { FORMAT_DATE, timeValide, versDateIso, versTimestamp } from "@/shared/lib/dates";

/**
 * Form entretien, création et modification.
 *
 * Les maquettes séparent « Date » et « Time » en deux champs, là où la base ne porte qu'un
 * horodatage : la composition est faite ici, à la validation, pour que le ViewModel et le
 * backend ne manipulent qu'un `RFC 3339`.
 */
export const interviewFormSchema = z
  .object({
    application_id: z.string().min(1, "La candidature concernée est obligatoire"),
    contact_id: z
      .string()
      .transform((value) => (value === "" ? null : value))
      .nullable()
      .default(null),
    date: z
      .string()
      .trim()
      .min(1, "La date est obligatoire")
      .refine((value) => versDateIso(value) !== null, {
        message: `Date invalide — format attendu ${FORMAT_DATE}.`,
      }),
    time: z
      .string()
      .trim()
      .min(1, "L'heure est obligatoire")
      .refine(timeValide, { message: "Heure invalide — format attendu HH:MM." }),
    type: z.enum(["Présentiel", "Visio", "Téléphonique", "Technique", "RH", "Autre"]),
    location: textFacultatif,
    notes: textFacultatif,
    minutes: textFacultatif,
  })
  .transform((values) => ({
    application_id: values.application_id,
    contact_id: values.contact_id,
    // `versHorodatage` a déjà été éprouvé par les deux `refine` ci-dessus : la date et
    // l'heure sont valides, la composition ne peut plus échouer.
    interview_date: versTimestamp(values.date, values.time) as string,
    type: values.type,
    location: values.location,
    notes: values.notes,
    minutes: values.minutes,
  }));

/** Values validées, telles qu'envoyées au backend. */
export type InterviewFormValues = z.output<typeof interviewFormSchema>;

/** Values saisies, avant transformation — ce que manipule React Hook Form. */
export type InterviewFormInput = z.input<typeof interviewFormSchema>;
