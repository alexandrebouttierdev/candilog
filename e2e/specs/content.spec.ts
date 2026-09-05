/**
 * Contrôle du contenu : fidélité au profil source, prise en compte de l'offre, diversité.
 *
 * Le CV et la lettre de Candilog sont recadrés sur des faits vérifiés (`ground_generated_resume`,
 * `render_grounded_letter`). Ces contrôles vérifient que la garantie tient vraiment sur des
 * profils très différents, et qu'aucun texte de gabarit ne fuit dans le document rendu.
 */
import { expect, test } from "@playwright/test";
import { fileURLToPath, URL } from "node:url";
import { existsSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { cle, texteDuCv, type DocumentCv, type Lettre, type ProfilSource } from "../lib/documents";

const SORTIE = fileURLToPath(new URL("../../test-output", import.meta.url));
const OFFRE = process.env["CANDILOG_E2E_OFFER"] ?? "";

const cas = existsSync(SORTIE)
  ? readdirSync(SORTIE)
      .filter((nom) => nom.startsWith("profile-") && existsSync(join(SORTIE, nom, "workspace.json")))
      .sort()
  : [];

function lire<T>(profil: string, fichier: string): T {
  return JSON.parse(readFileSync(join(SORTIE, profil, fichier), "utf8")) as T;
}

/** Défauts de rédaction française que le gabarit peut produire quel que soit le profil. */
function defautsDeRedaction(texte: string): string[] {
  const defauts: string[] = [];
  for (const trouve of texte.matchAll(/\b(?:de|que)\s+([aeiouâàéèêëîïôùûüh])/gi)) {
    // « de Astek », « au poste de Administrateur » : l'élision manque.
    defauts.push(`élision manquante : « ${texte.slice(Math.max(0, trouve.index - 25), trouve.index + 35).replace(/\s+/g, " ")} »`);
  }
  for (const trouve of texte.matchAll(/[.!?]{2,}/g)) {
    if (trouve[0] === "..." ) continue;
    defauts.push(`ponctuation doublée : « ${texte.slice(Math.max(0, trouve.index - 40), trouve.index + 10).replace(/\s+/g, " ")} »`);
  }
  for (const paragraphe of texte.split(/\n{2,}/)) {
    if (paragraphe.includes("\n") && !/^(Madame|Cordialement|Veuillez)/.test(paragraphe.trim())) {
      defauts.push(`retour à la ligne au milieu d'une phrase : « ${paragraphe.replace(/\n/g, " ⏎ ").slice(0, 110)} »`);
    }
  }
  return defauts;
}

test.describe("contenu des documents", () => {
  for (const profil of cas) {
    test(`${profil} — le CV ne contient que des faits du profil`, () => {
      const source = lire<ProfilSource>(profil, "profile.json");
      const document = lire<{ document: DocumentCv }>(profil, "workspace.json").document;

      const attendu = [source.identity.first_name, source.identity.name].filter(Boolean).join(" ");
      expect(document.identity.full_name, "nom du candidat").toBe(attendu);
      expect(document.identity.email, "adresse e-mail").toBe(source.identity.email);
      expect(cle(document.identity.title), "intitulé professionnel").toBe(cle(source.identity.title ?? ""));

      const experiencesSource = new Set(source.experiences.map((item) => `${cle(item.title)}|${cle(item.company)}`));
      const inventees = document.experiences
        .filter((item) => !experiencesSource.has(`${cle(item.title)}|${cle(item.company)}`))
        .map((item) => `${item.title} — ${item.company}`);
      expect(inventees, "expériences absentes du profil source").toEqual([]);

      const formationsSource = new Set(source.education.map((item) => `${cle(item.degree)}|${cle(item.school)}`));
      const diplomesInventes = document.education
        .filter((item) => !formationsSource.has(`${cle(item.degree)}|${cle(item.school)}`))
        .map((item) => `${item.degree} — ${item.school}`);
      expect(diplomesInventes, "formations absentes du profil source").toEqual([]);

      const competencesSource = new Set(source.skills.map((item) => cle(item.name)));
      const competencesInventees = document.skill_groups
        .flatMap((groupe) => groupe.items)
        .filter((item) => !competencesSource.has(cle(item)));
      expect(competencesInventees, "compétences absentes du profil source").toEqual([]);

      const certificationsSource = new Set(source.certifications.map((item) => cle(item.name)));
      expect(
        document.certifications.filter((item) => !certificationsSource.has(cle(item.name))).map((item) => item.name),
        "certifications absentes du profil source",
      ).toEqual([]);

      const languesSource = new Set(source.languages.map((item) => cle(item.name)));
      expect(
        document.languages.filter((item) => !languesSource.has(cle(item.name))).map((item) => item.name),
        "langues absentes du profil source",
      ).toEqual([]);

      // Chaque puce doit être une ligne de la description d'origine, jamais une phrase neuve.
      const lignesSource = new Set(
        source.experiences.flatMap((item) =>
          (item.description ?? "").split("\n").map((ligne) => cle(ligne)).filter(Boolean),
        ),
      );
      const puccesInventees = document.experiences
        .flatMap((item) => item.bullets)
        .filter((puce) => !lignesSource.has(cle(puce)));
      expect(puccesInventees, "puces d'expérience absentes du profil source").toEqual([]);
    });

    test(`${profil} — le socle est complet sans copier les contenus optionnels`, () => {
      const source = lire<ProfilSource>(profil, "profile.json");
      const workspace = lire<{ document: DocumentCv; profile_library: unknown[] }>(profil, "workspace.json");
      const document = workspace.document;

      expect(document.experiences.length, "expériences retenues").toBe(source.experiences.length);
      expect(document.education.length, "formations retenues").toBe(source.education.length);
      expect(document.skill_groups, "compétences copiées automatiquement").toEqual([]);
      expect(document.projects, "projets copiés automatiquement").toEqual([]);
      expect(document.certifications, "certifications copiées automatiquement").toEqual([]);
      expect(document.languages, "langues copiées automatiquement").toEqual([]);
      expect(workspace.profile_library.length, "bibliothèque optionnelle").toBe(
        source.skills.length + source.projects.length + source.certifications.length + source.languages.length,
      );

      const texte = texteDuCv(document);
      expect(texte, "valeur parasite dans le CV").not.toMatch(/\b(undefined|null|NaN)\b|\[object |\{\{|\$\{/);
      for (const experience of document.experiences) {
        expect(experience.period.trim(), `période de « ${experience.title} »`).not.toBe("");
      }
    });

    test(`${profil} — la lettre reste fidèle au profil et à l'offre`, () => {
      const source = lire<ProfilSource>(profil, "profile.json");
      const lettre = lire<Lettre>(profil, "letter.json");
      const corps = lettre.content;

      expect(corps, "poste visé").toContain(lettre.job_title ?? "");
      expect(corps, "entreprise visée").toContain(lettre.company ?? "");
      expect(corps, "valeur parasite dans la lettre").not.toMatch(/\b(undefined|null|NaN)\b|\[object |\{\{|\$\{/);

      // Chaque fait avancé doit venir du catalogue construit depuis le profil.
      const catalogue = [
        source.identity.resume ?? "",
        ...source.experiences.map((item) => `${item.title} chez ${item.company}${item.description ? ` : ${item.description}` : ""}`),
        ...source.skills.map((item) => item.name),
        ...source.education.map((item) => `${item.degree} à ${item.school}`),
        ...source.projects.map((item) => item.name),
        ...source.certifications.map((item) => item.name),
      ].map(cle);
      const amorces = [
        "Mon projet professionnel : ",
        "Mon expérience comprend notamment ",
        "Mon parcours comprend notamment ",
        "Je peux notamment mobiliser ",
        "Ma formation inclut ",
        "J'ai également mené le projet ",
        "Mon parcours comprend aussi la certification ",
      ];
      const faitsAvances = corps
        .split(/\n{2,}/)
        .map((bloc) => amorces.find((amorce) => bloc.startsWith(amorce)) ? bloc.slice(amorces.find((amorce) => bloc.startsWith(amorce))!.length) : null)
        .filter((fait): fait is string => fait !== null)
        .map((fait) => cle(fait.replace(/\.$/, "")));
      const horsCatalogue = faitsAvances.filter((fait) => !catalogue.some((entree) => entree.includes(fait) || fait.includes(entree)));
      expect(horsCatalogue, "faits de la lettre absents du profil").toEqual([]);

      expect(defautsDeRedaction(corps), "défauts de rédaction française").toEqual([]);
    });
  }

  test("les documents restent distincts d'un profil à l'autre", () => {
    test.skip(cas.length < 2, "au moins deux profils sont nécessaires");
    const empreintes = cas.map((profil) => ({
      profil,
      cv: new Set(cle(texteDuCv(lire<{ document: DocumentCv }>(profil, "workspace.json").document)).split(" ")),
      lettre: new Set(cle(lire<Lettre>(profil, "letter.json").content).split(" ")),
    }));
    const jaccard = (a: Set<string>, b: Set<string>) => {
      const commun = [...a].filter((mot) => b.has(mot)).length;
      return commun / (a.size + b.size - commun);
    };
    const trop = [];
    for (let index = 0; index < empreintes.length; index += 1) {
      for (let autre = index + 1; autre < empreintes.length; autre += 1) {
        const cv = jaccard(empreintes[index]!.cv, empreintes[autre]!.cv);
        const lettre = jaccard(empreintes[index]!.lettre, empreintes[autre]!.lettre);
        if (cv > 0.6) trop.push(`CV ${empreintes[index]!.profil} / ${empreintes[autre]!.profil} : ${cv.toFixed(2)}`);
        if (lettre > 0.75) trop.push(`Lettre ${empreintes[index]!.profil} / ${empreintes[autre]!.profil} : ${lettre.toFixed(2)}`);
      }
    }
    expect(trop, "documents trop semblables d'un profil à l'autre").toEqual([]);
  });

  test("l'offre de référence est réellement prise en compte", () => {
    test.skip(!OFFRE || !existsSync(OFFRE), "CANDILOG_E2E_OFFER absent");
    const offre = cle(readFileSync(OFFRE, "utf8"));
    const couverture = cas.map((profil) => {
      const workspace = lire<{ document: DocumentCv; job_offer: { skills: string[]; keywords: string[] } }>(
        profil,
        "workspace.json",
      );
      const texte = cle(texteDuCv(workspace.document));
      const attendus = [...workspace.job_offer.skills, ...workspace.job_offer.keywords].map(cle).filter(Boolean);
      const repris = attendus.filter((terme) => texte.includes(terme));
      return { profil, attendus: attendus.length, repris: repris.length, termes: repris };
    });
    writeFileSync(join(SORTIE, "offer-coverage.json"), JSON.stringify({ offreLue: offre.length, couverture }, null, 2), "utf8");
    // L'offre est structurée par le modèle puis recadrée sur son texte : la liste ne peut
    // pas être vide, sinon le CV et le score ATS ne sont ciblés sur rien.
    expect(couverture.filter((entree) => entree.attendus === 0).map((entree) => entree.profil), "offre non exploitée").toEqual([]);
  });
});
