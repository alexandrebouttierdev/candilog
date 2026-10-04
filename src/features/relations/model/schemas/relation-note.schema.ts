import { z } from "zod";
import { FORMAT_DATE, toIsoDate } from "@/shared/lib/dates";

/**
 * Borne haute du corps d'une note, alignée sur le service Rust et le `CHECK` de
 * `relation_notes` (`docs/DATA.md`).
 *
 * Reprise ici pour que la limite soit connue de l'interface : sans elle, une note trop
 * longue n'était refusée qu'après l'aller-retour IPC, au moment d'enregistrer.
 */
export const MAX_NOTE_BODY = 2000;

/**
 * Note datée d'une entreprise ou d'un contact.
 *
 * `noted_on` est la date du **fait** et non celle de la saisie : elle est saisie au format
 * d'affichage (`JJ-MM-AAAA`) et transformée en ISO pour le backend, comme la relance.
 */
export const relationNoteFormSchema = z.object({
  body: z
    .string()
    .trim()
    .min(1, "La note est obligatoire")
    .max(MAX_NOTE_BODY, `La note ne peut pas dépasser ${MAX_NOTE_BODY} caractères`),
  noted_on: z
    .string()
    .trim()
    .min(1, "La date est obligatoire")
    .refine((value) => toIsoDate(value) !== null, {
      message: `Date invalide — format attendu ${FORMAT_DATE}.`,
    })
    .transform((value) => toIsoDate(value) as string),
});

/** Valeurs validées, telles qu'envoyées au backend. */
export type RelationNoteFormValues = z.output<typeof relationNoteFormSchema>;

/** Valeurs saisies, avant transformation — ce que manipule React Hook Form. */
export type RelationNoteFormInput = z.input<typeof relationNoteFormSchema>;
