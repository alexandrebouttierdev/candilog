/**
 * Lecture des artefacts du scénario : ce que le document est censé contenir.
 *
 * Sert deux contrôles : les caractères que le PDF doit savoir imprimer, et les faits qui
 * doivent — ou ne doivent pas — se retrouver dans le document rendu.
 */
import { existsSync, readFileSync } from "node:fs";

export type DocumentCv = {
  identity: {
    full_name: string;
    title: string;
    headline: string | null;
    city: string | null;
    phone: string | null;
    email: string;
    website: string | null;
    linkedin: string | null;
    github: string | null;
    extra: string[];
  };
  profile: string;
  experiences: { title: string; company: string; location: string | null; period: string; bullets: string[] }[];
  projects: { name: string; meta: string | null; url: string | null; bullets: string[] }[];
  skill_groups: { name: string; items: string[] }[];
  education: { degree: string; school: string; location: string | null; period: string; description: string | null }[];
  certifications: { name: string; issuer: string | null; date: string | null }[];
  languages: { name: string; level: string }[];
};

export type Lettre = {
  identity: { first_name: string; name: string; email: string; title: string | null; city: string | null; phone: string | null; address: string | null };
  company: string | null;
  job_title: string | null;
  content: string;
};

export type ProfilSource = {
  identity: { first_name: string; name: string; email: string; title: string | null; city: string | null; resume: string | null };
  experiences: { title: string; company: string; description: string | null }[];
  skills: { name: string }[];
  education: { degree: string; school: string }[];
  languages: { name: string; level: string }[];
  projects: { name: string }[];
  certifications: { name: string; issuer: string | null }[];
};

/** Tout le texte que le CV imprime, dans l'ordre du gabarit. */
export function texteDuCv(document: DocumentCv): string {
  const morceaux = [
    document.identity.full_name,
    document.identity.title,
    document.identity.headline ?? "",
    document.identity.city ?? "",
    document.identity.phone ?? "",
    document.identity.email,
    document.identity.website ?? "",
    document.identity.linkedin ?? "",
    document.identity.github ?? "",
    ...document.identity.extra,
    document.profile,
    ...document.experiences.flatMap((experience) => [
      experience.title,
      experience.company,
      experience.location ?? "",
      experience.period,
      ...experience.bullets,
    ]),
    ...document.projects.flatMap((projet) => [projet.name, projet.meta ?? "", projet.url ?? "", ...projet.bullets]),
    ...document.skill_groups.flatMap((groupe) => [groupe.name, ...groupe.items]),
    ...document.education.flatMap((formation) => [
      formation.degree,
      formation.school,
      formation.location ?? "",
      formation.period,
      formation.description ?? "",
    ]),
    ...document.certifications.flatMap((certification) => [
      certification.name,
      certification.issuer ?? "",
      certification.date ?? "",
    ]),
    ...document.languages.flatMap((langue) => [langue.name, langue.level]),
  ];
  return morceaux.join(" ");
}

/** Tout le texte que la lettre imprime, corps et en-tête d'identité compris. */
export function texteDeLaLettre(lettre: Lettre): string {
  return [
    lettre.identity.first_name,
    lettre.identity.name,
    lettre.identity.title ?? "",
    lettre.identity.address ?? "",
    lettre.identity.city ?? "",
    lettre.identity.phone ?? "",
    lettre.identity.email,
    lettre.company ?? "",
    lettre.job_title ?? "",
    lettre.content.replace(/<[^>]*>/g, " "),
  ].join(" ");
}

/** Normalise pour comparer des textes : minuscules, accents retirés, espaces réduits. */
export function cle(valeur: string): string {
  return valeur
    .normalize("NFD")
    .replace(/[̀-ͯ]/g, "")
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, " ")
    .trim();
}

/**
 * Comportement attendu de l'export pour un profil, quand ce n'est pas la réussite.
 *
 * Déclaré une seule fois, à côté du profil source, et lu ici comme par le scénario Rust :
 * une attente qui vivrait dans deux fichiers finirait par dire deux choses différentes.
 */
export function attenteDuProfil(racine: string, profil: string): string | null {
  const chemin = `${racine}/src-tauri/tests/fixtures/profiles/${profil}.expected.json`;
  if (!existsSync(chemin)) return null;
  return (JSON.parse(readFileSync(chemin, "utf8")) as { cv_pdf?: string }).cv_pdf ?? null;
}
