import { z } from "zod";

const optional = z
  .string()
  .trim()
  .transform((value) => (value === "" ? null : value))
  .nullable();

const urlOptional = (message: string) =>
  optional.refine(
    (value) =>
      value === null ||
      (z.url().safeParse(value).success && /^(https?):\/\//i.test(value)),
    message,
  );

export const identitySchema = z.object({
  first_name: z.string().trim(),
  name: z.string().trim(),
  email: z
    .string()
    .trim()
    .refine((value) => value === "" || z.email().safeParse(value).success, {
      message: "Adresse e-mail invalide",
    }),
  phone: optional,
  address: optional,
  city: optional,
  title: optional,
  resume: optional,
  linkedin: urlOptional("Le profil LinkedIn doit commencer par http:// ou https://"),
  github: urlOptional("Le profil GitHub doit commencer par http:// ou https://"),
  website: urlOptional("Le site web doit commencer par http:// ou https://"),
});

export const experienceSchema = z
  .object({
    title: z.string().trim().min(1, "L'intitulé est obligatoire"),
    company: z.string().trim().min(1, "L'entreprise est obligatoire"),
    location: optional,
    start_date: z.string().trim().min(1, "La date de début est obligatoire"),
    end_date: optional,
    current: z.boolean(),
    description: optional,
  })
  .superRefine((value, context) => {
    if (value.current && value.end_date !== null) {
      context.addIssue({
        code: "custom",
        path: ["end_date"],
        message: "Un poste actuel ne peut pas avoir de date de fin",
      });
    }
  });

export const educationItemSchema = z.object({
  degree: z.string().trim().min(1, "Le diplôme est obligatoire"),
  school: z.string().trim().min(1, "L'établissement est obligatoire"),
  location: optional,
  start_date: optional,
  end_date: optional,
  description: optional,
});

export const languageSchema = z.object({
  name: z.string().trim().min(1, "La langue est obligatoire"),
  level: z.string().trim().min(1, "Le niveau est obligatoire"),
});

export const projectSchema = z.object({
  name: z.string().trim().min(1, "Le nom est obligatoire"),
  description: optional,
  url: urlOptional("Le lien doit commencer par http:// ou https://"),
  technologies: optional,
});

export const certificationSchema = z.object({
  name: z.string().trim().min(1, "Le nom est obligatoire"),
  issuer: optional,
  date: optional,
  url: urlOptional("Le lien doit commencer par http:// ou https://"),
});

export const skillsSchema = z.array(
  z.object({ name: z.string().trim().min(1, "Le nom est obligatoire") }),
);
export const experiencesSchema = z.array(experienceSchema);
export const educationSchema = z.array(educationItemSchema);
export const languagesSchema = z.array(languageSchema);
export const projectsSchema = z.array(projectSchema);
export const certificationsSchema = z.array(certificationSchema);

// `useFieldArray` exige un objet racine stable. Les schémas de section conservent les
// schémas métier ci-dessus et ne déballent `items` qu'au moment de fusionner le profil.
export const experiencesFormSchema = z.object({ items: experiencesSchema });
export const skillsFormSchema = z.object({ items: skillsSchema });
export const educationFormSchema = z.object({ items: educationSchema });
export const languagesFormSchema = z.object({ items: languagesSchema });
export const projectsFormSchema = z.object({ items: projectsSchema });
export const certificationsFormSchema = z.object({ items: certificationsSchema });
