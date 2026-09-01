/**
 * Contrôle du rendu HTML des CV et lettres produits par le scénario de bout en bout.
 *
 * Les artefacts viennent de `src-tauri/tests/e2e_documents.rs` : ce fichier ne génère rien,
 * il mesure. Chaque cas produit une capture et une fiche JSON reprises par le rapport.
 */
import { expect, test } from "@playwright/test";
import { fileURLToPath, URL } from "node:url";
import { existsSync, mkdirSync, readdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import {
  collisions,
  debordements,
  journalDuNavigateur,
  placeholders,
  polices,
  type Debordement,
} from "../lib/checks";
import { attenteDuProfil } from "../lib/documents";

const SORTIE = fileURLToPath(new URL("../../test-output", import.meta.url));
const RACINE = fileURLToPath(new URL("../..", import.meta.url));

const cas = existsSync(SORTIE)
  ? readdirSync(SORTIE)
      .filter((nom) => nom.startsWith("profile-"))
      .sort()
  : [];

const DOCUMENTS = [
  { genre: "resume", feuille: ".resume-paper", artefact: "workspace.json", capture: "cv.png", fiche: "cv-layout.json" },
  { genre: "letter", feuille: ".letter-paper", artefact: "letter.json", capture: "cover-letter.png", fiche: "cover-letter-layout.json" },
] as const;

test.describe("rendu des feuilles A4", () => {
  for (const profil of cas) {
    for (const cible of DOCUMENTS) {
      test(`${profil} — ${cible.genre}`, async ({ page }, info) => {
        const dossier = join(SORTIE, profil);
        test.skip(!existsSync(join(dossier, cible.artefact)), `${cible.artefact} absent`);

        const erreurs = journalDuNavigateur(page);
        await page.goto(`/e2e/harness/index.html?kind=${cible.genre}&dir=/test-output/${profil}`);
        await page.waitForFunction(() => document.body.dataset["etat"] !== "chargement", null, { timeout: 30_000 });
        const etat = await page.evaluate(() => ({
          etat: document.body.dataset["etat"],
          detail: document.body.dataset["detail"],
        }));
        expect(etat.etat, `banc en erreur : ${etat.detail ?? ""}`).toBe("pret");

        const feuille = page.locator(cible.feuille);
        await expect(feuille).toBeVisible();

        // Le rendu contrôlé doit être celui du média d'impression : c'est lui qui décrit
        // la page exportée. Les deux médias sont mesurés pour repérer tout écart.
        const mesures: Record<string, unknown> = {};
        for (const media of ["screen", "print"] as const) {
          await page.emulateMedia({ media });
          await page.evaluate(() => new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r))));
          mesures[media] = {
            debordements: await debordements(page, cible.feuille),
            collisions: await collisions(page, cible.feuille),
          };
        }
        await page.emulateMedia({ media: "screen" });

        const fiche = {
          profil,
          genre: cible.genre,
          ...mesures,
          placeholders: await placeholders(page, cible.feuille),
          polices: await polices(page, cible.feuille),
          avertissementDebordement: await page.locator(".resume-overflow-warning, .letter-overflow-warning").count(),
          boite: await feuille.boundingBox(),
          erreursNavigateur: erreurs,
        };
        mkdirSync(dossier, { recursive: true });
        writeFileSync(join(dossier, cible.fiche), JSON.stringify(fiche, null, 2), "utf8");
        writeFileSync(join(dossier, cible.genre === "resume" ? "cv.html" : "cover-letter.html"), await page.content(), "utf8");
        await feuille.screenshot({ path: join(dossier, cible.capture) });
        await info.attach(`${profil}-${cible.genre}`, { path: join(dossier, cible.capture), contentType: "image/png" });

        const ecran = mesures["screen"] as { debordements: Debordement[]; collisions: unknown[] };
        const impression = mesures["print"] as { debordements: Debordement[]; collisions: unknown[] };
        expect(fiche.polices.manquantes, "polices du gabarit absentes").toEqual([]);
        expect(fiche.erreursNavigateur, "erreurs du navigateur").toEqual([]);
        expect(fiche.placeholders, "valeurs parasites sur la feuille").toEqual([]);
        expect(ecran.collisions, "collisions à l'écran").toEqual([]);
        expect(impression.collisions, "collisions au média print").toEqual([]);

        // Un profil dont le contenu dépasse réellement la page doit le dire — c'est ce que
        // l'export refuse aussi. La feuille reste tenue de garder chaque bloc dans sa
        // colonne : seul le dépassement en bas de page est alors attendu.
        const trop = cible.genre === "resume" && attenteDuProfil(RACINE, profil) === "refus_longueur";
        if (trop) {
          expect(fiche.avertissementDebordement, "le papier doit signaler le débordement").toBe(1);
          const horsColonne = (axe: Debordement[]) => axe.filter((x) => x.axe === "hors-colonne");
          expect(horsColonne(ecran.debordements), "blocs sortis de leur colonne").toEqual([]);
          expect(horsColonne(impression.debordements), "blocs sortis de leur colonne (print)").toEqual([]);
          return;
        }
        expect(ecran.debordements, "débordements à l'écran").toEqual([]);
        expect(impression.debordements, "débordements au média print").toEqual([]);
        expect(fiche.avertissementDebordement, "le papier signale un débordement").toBe(0);
      });
    }
  }
});
