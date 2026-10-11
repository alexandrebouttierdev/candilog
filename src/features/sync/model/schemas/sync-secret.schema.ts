import { z } from "zod";

/** Longueur minimale de la phrase secrète, identique à `crypto::MIN_PASSPHRASE_CHARS`. */
export const MIN_PASSPHRASE_CHARS = 12;

/**
 * Phrase secrète d'un nouvel espace : la confirmation évite qu'une faute de frappe rende
 * l'espace illisible depuis les autres ordinateurs.
 */
export const createSpaceSchema = z
  .object({
    passphrase: z
      .string()
      .min(MIN_PASSPHRASE_CHARS, `Au moins ${MIN_PASSPHRASE_CHARS} caractères.`)
      .max(1024, "Phrase secrète trop longue."),
    confirmation: z.string(),
  })
  .refine((values) => values.passphrase === values.confirmation, {
    path: ["confirmation"],
    message: "Les deux saisies ne correspondent pas.",
  });

export type CreateSpaceValues = z.output<typeof createSpaceSchema>;

/** Secret pour ouvrir un espace existant : la phrase secrète, ou la clé de récupération. */
export const unlockSchema = z
  .object({
    method: z.enum(["passphrase", "recovery"]),
    secret: z.string().trim().min(1, "Saisissez le secret de l'espace.").max(1024, "Trop long."),
  })
  .refine(
    (values) =>
      values.method === "passphrase" ||
      values.secret.replace(/[\s-]/g, "").length >= 52,
    { path: ["secret"], message: "Une clé de récupération compte 52 caractères." },
  );

export type UnlockValues = z.output<typeof unlockSchema>;

/** Rejoindre un espace existant : le scénario est toujours choisi explicitement. */
export const joinSpaceSchema = z
  .object({
    choice: z.enum(["join_merge", "join_replace_local"], {
      message: "Choisissez ce que deviennent les données de cet ordinateur.",
    }),
    method: z.enum(["passphrase", "recovery"]),
    secret: z.string().trim().min(1, "Saisissez le secret de l'espace.").max(1024, "Trop long."),
  })
  .refine(
    (values) =>
      values.method === "passphrase" ||
      values.secret.replace(/[\s-]/g, "").length >= 52,
    { path: ["secret"], message: "Une clé de récupération compte 52 caractères." },
  );

export type JoinSpaceInput = z.input<typeof joinSpaceSchema>;
export type JoinSpaceValues = z.output<typeof joinSpaceSchema>;
