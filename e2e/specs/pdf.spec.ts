/**
 * Contrôle des PDF exportés par Candilog : structure, marges, chevauchements, glyphes.
 *
 * Le PDF officiel est celui du moteur Rust (`infrastructure/pdf`). Playwright n'en produit
 * aucun : il n'est ici qu'un lanceur de contrôles et un rapporteur.
 */
import { expect, test } from "@playwright/test";
import { fileURLToPath, URL } from "node:url";
import { existsSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { caracteresPerdus, controlerPdf } from "../lib/pdf";
import {
  attenteDuProfil,
  texteDeLaLettre,
  texteDuCv,
  type DocumentCv,
  type Lettre,
} from "../lib/documents";

const SORTIE = fileURLToPath(new URL("../../test-output", import.meta.url));
const RACINE = fileURLToPath(new URL("../..", import.meta.url));

const cas = existsSync(SORTIE)
  ? readdirSync(SORTIE)
      .filter((nom) => nom.startsWith("profile-"))
      .sort()
  : [];

const DOCUMENTS = [
  { genre: "cv", fichier: "cv.pdf", source: "workspace.json", fiche: "cv-pdf.json", images: "cv-pdf-pages" },
  { genre: "lettre", fichier: "cover-letter.pdf", source: "letter.json", fiche: "cover-letter-pdf.json", images: "cover-letter-pdf-pages" },
] as const;

test.describe("PDF exportés", () => {
  for (const profil of cas) {
    for (const cible of DOCUMENTS) {
      // Aucune fixture n'est demandée : ces contrôles n'ouvrent pas de navigateur, ils
      // lisent le fichier exporté. `test.info()` donne les pièces jointes du rapport.
      test(`${profil} — ${cible.genre}`, async () => {
        const info = test.info();
        const dossier = join(SORTIE, profil);
        const chemin = join(dossier, cible.fichier);
        const images = join(dossier, cible.images);
        mkdirSync(images, { recursive: true });

        // Un profil dont le contenu dépasse réellement la page A4 doit être refusé à
        // l'export, pas exporté tronqué : l'attente est déclarée à côté du profil source.
        if (cible.genre === "cv" && attenteDuProfil(RACINE, profil) === "refus_longueur") {
          expect(
            existsSync(chemin),
            "l'export aurait dû refuser ce CV : son contenu dépasse la page",
          ).toBe(false);
          return;
        }

        const rapport = controlerPdf(chemin, images);
        const source = existsSync(join(dossier, cible.source))
          ? (JSON.parse(readFileSync(join(dossier, cible.source), "utf8")) as
              | { document: DocumentCv }
              | Lettre)
          : null;
        const attendu = !source
          ? ""
          : cible.genre === "cv"
            ? texteDuCv((source as { document: DocumentCv }).document)
            : texteDeLaLettre(source as Lettre);
        const perdus = attendu ? caracteresPerdus(attendu, rapport.texte) : [];
        writeFileSync(
          join(dossier, cible.fiche),
          JSON.stringify({ ...rapport, texte: rapport.texte.slice(0, 4000), caracteresPerdus: perdus }, null, 2),
          "utf8",
        );
        for (const capture of rapport.captures) {
          await info.attach(`${profil}-${cible.genre}-${capture.split("/").pop()}`, {
            path: capture,
            contentType: "image/png",
          });
        }

        expect(rapport.erreurs, "erreurs d'ouverture ou de lecture du PDF").toEqual([]);
        expect(rapport.entete, "en-tête PDF").toBe(true);
        expect(rapport.octets, "taille du fichier").toBeGreaterThan(0);
        expect(rapport.pages, "nombre de pages").toBeGreaterThan(0);
        expect(Math.round(rapport.largeurPt), "largeur A4 en points").toBe(595);
        expect(Math.round(rapport.hauteurPt), "hauteur A4 en points").toBe(842);
        expect(rapport.mots, "mots extraits").toBeGreaterThan(20);
        expect(rapport.pagesVides, "pages sans aucun texte").toEqual([]);
        expect(rapport.captures.length, "pages rendues en image").toBe(rapport.pages);
        expect(rapport.horsMarges, "mots hors des marges").toEqual([]);
        expect(rapport.chevauchements, "mots imprimés l'un sur l'autre").toEqual([]);
        expect(perdus, "caractères du document absents du PDF").toEqual([]);
      });
    }
  }
});
