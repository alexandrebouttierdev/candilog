import { z } from "zod";
import { idFacultatif, textFacultatif } from "@/shared/lib/zod-helpers";
import { FORMAT_DATE, versDateIso } from "@/shared/lib/dates";

/** Borne haute du volume horaire, alignée sur `MAX_WEEKLY_HOURS` côté Rust. */
export const MAX_WEEKLY_HOURS = 168;

/**
 * Volume horaire hebdomadaire facultatif.
 *
 * Le champ est un `<input>` : sa valeur est une chaîne, y compris quand elle est vide. La
 * virgule décimale est acceptée — c'est la façon française d'écrire 17,5 h, et la refuser
 * ferait échouer une saisie correcte.
 */
const weeklyHoursFacultatif = z
  .string()
  .trim()
  .default("")
  .superRefine((value, ctx) => {
    if (value === "") return;
    const hours = Number(value.replace(",", "."));
    if (!Number.isFinite(hours)) {
      ctx.addIssue({ code: "custom", message: "Indiquez un nombre d'heures." });
      return;
    }
    if (hours <= 0 || hours > MAX_WEEKLY_HOURS) {
      ctx.addIssue({
        code: "custom",
        message: `Le nombre d'heures doit être compris entre 0 et ${MAX_WEEKLY_HOURS}.`,
      });
    }
  })
  .transform((value) => (value === "" ? null : Number(value.replace(",", "."))));

/**
 * Formulaire candidature, création et modification.
 *
 * Reprend les règles de `ApplicationService::normalize` côté Rust : poste requis, contrat
 * requis, date valide, volume horaire plausible, lien limité à HTTP(S). L'entreprise est
 * requise ici **et** en base par une clé étrangère `NOT NULL` : une candidature sans
 * entreprise n'a pas de sens métier.
 *
 * Ville, adresse et type d'entreprise sont des **surcharges** : laissées vides, elles
 * valent `null` et la candidature hérite de son entreprise. Le formulaire ne préremplit
 * donc jamais la valeur héritée, il l'affiche en indication.
 */
export const applicationFormSchema = z
  .object({
    job_title: z.string().trim().min(1, "Le poste est obligatoire"),
    company_id: z.string().min(1, "L'entreprise est obligatoire"),
    contact_id: idFacultatif,
    application_type: z.enum(["OFFRE", "SPONTANEE"]),
    contract_type_code: z.string().min(1, "Le type de contrat est obligatoire"),
    weekly_work_schedule: z.enum(["FULL_TIME", "PART_TIME", "UNSPECIFIED"]),
    weekly_hours: weeklyHoursFacultatif,
    professional_domain_id: idFacultatif,
    city: textFacultatif,
    address: textFacultatif,
    company_type_id: idFacultatif,
    status: z.enum(["EN_ATTENTE", "RELANCEE", "ENTRETIEN", "REFUS"]),
    sent_date: z
      .string()
      .trim()
      .min(1, "La date d'envoi est obligatoire")
      .refine((value) => versDateIso(value) !== null, {
        message: `Date invalide — format attendu ${FORMAT_DATE}.`,
      })
      // Transformée dès la validation : le ViewModel et le backend ne manipulent que de l'ISO.
      .transform((value) => versDateIso(value) as string),
    job_url: z.string().trim().default(""),
    notes: textFacultatif,
  })
  .superRefine((values, ctx) => {
    // Le lien est la trace de l'offre à laquelle on a répondu : sans lui, relire la fiche
    // six mois plus tard ne dit plus à quoi la candidature correspondait.
    if (values.application_type !== "OFFRE") return;
    if (values.job_url === "") {
      ctx.addIssue({
        code: "custom",
        path: ["job_url"],
        message: "Le lien de l'offre est obligatoire pour une candidature à une offre.",
      });
      return;
    }
    if (!estUrlHttp(values.job_url)) {
      ctx.addIssue({
        code: "custom",
        path: ["job_url"],
        message: "Le lien de l'offre doit commencer par http:// ou https://",
      });
    }
  })
  .transform((values) => ({
    ...values,
    // Une candidature spontanée n'a pas d'offre : conserver le lien d'un ancien état
    // « offre » ferait pointer la fiche vers une annonce sans rapport.
    job_url: values.application_type === "OFFRE" ? values.job_url : null,
  }));

/** Vrai si la chaîne est une URL HTTP(S) — la règle appliquée par le backend. */
function estUrlHttp(value: string): boolean {
  try {
    return ["http:", "https:"].includes(new URL(value).protocol);
  } catch {
    return false;
  }
}

/** Valeurs validées, telles qu'envoyées au backend. */
export type ApplicationFormValues = z.output<typeof applicationFormSchema>;

/** Valeurs saisies, avant transformation — ce que manipule React Hook Form. */
export type ApplicationFormInput = z.input<typeof applicationFormSchema>;
