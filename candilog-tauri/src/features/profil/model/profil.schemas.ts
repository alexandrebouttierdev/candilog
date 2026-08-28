import { z } from "zod";

const optionnel = z
  .string()
  .trim()
  .transform((valeur) => (valeur === "" ? null : valeur))
  .nullable();

const urlOptionnelle = (message: string) =>
  optionnel.refine(
    (valeur) =>
      valeur === null ||
      (z.url().safeParse(valeur).success && /^(https?):\/\//i.test(valeur)),
    message,
  );

export const identiteSchema = z.object({
  prenom: z.string().trim(),
  nom: z.string().trim(),
  email: z
    .string()
    .trim()
    .refine((valeur) => valeur === "" || z.email().safeParse(valeur).success, {
      message: "Adresse e-mail invalide",
    }),
  telephone: optionnel,
  ville: optionnel,
  titre: optionnel,
  resume: optionnel,
  linkedin: urlOptionnelle("Le profil LinkedIn doit commencer par http:// ou https://"),
  github: urlOptionnelle("Le profil GitHub doit commencer par http:// ou https://"),
  siteWeb: urlOptionnelle("Le site web doit commencer par http:// ou https://"),
});

export const experienceSchema = z
  .object({
    intitule: z.string().trim().min(1, "L'intitulé est obligatoire"),
    entreprise: z.string().trim().min(1, "L'entreprise est obligatoire"),
    lieu: optionnel,
    dateDebut: z.string().trim().min(1, "La date de début est obligatoire"),
    dateFin: optionnel,
    posteActuel: z.boolean(),
    description: optionnel,
  })
  .superRefine((valeur, contexte) => {
    if (valeur.posteActuel && valeur.dateFin !== null) {
      contexte.addIssue({
        code: "custom",
        path: ["dateFin"],
        message: "Un poste actuel ne peut pas avoir de date de fin",
      });
    }
  });

export const formationSchema = z.object({
  diplome: z.string().trim().min(1, "Le diplôme est obligatoire"),
  etablissement: z.string().trim().min(1, "L'établissement est obligatoire"),
  lieu: optionnel,
  dateDebut: optionnel,
  dateFin: optionnel,
  description: optionnel,
});

export const langueSchema = z.object({
  nom: z.string().trim().min(1, "La langue est obligatoire"),
  niveau: z.string().trim().min(1, "Le niveau est obligatoire"),
});

export const projetSchema = z.object({
  nom: z.string().trim().min(1, "Le nom est obligatoire"),
  description: optionnel,
  url: urlOptionnelle("Le lien doit commencer par http:// ou https://"),
  technologies: optionnel,
});

export const certificationSchema = z.object({
  nom: z.string().trim().min(1, "Le nom est obligatoire"),
  organisme: optionnel,
  date: optionnel,
  url: urlOptionnelle("Le lien doit commencer par http:// ou https://"),
});

export const competencesSchema = z.array(
  z.object({ nom: z.string().trim().min(1, "Le nom est obligatoire") }),
);
export const experiencesSchema = z.array(experienceSchema);
export const formationsSchema = z.array(formationSchema);
export const languesSchema = z.array(langueSchema);
export const projetsSchema = z.array(projetSchema);
export const certificationsSchema = z.array(certificationSchema);
