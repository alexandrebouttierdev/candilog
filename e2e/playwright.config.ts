import { defineConfig } from "@playwright/test";
import { fileURLToPath, URL } from "node:url";

const racine = fileURLToPath(new URL("..", import.meta.url));

/**
 * Contrôle visuel des feuilles A4 de Candilog.
 *
 * Le banc de rendu (`e2e/harness`) est servi par le serveur de développement Vite, le même
 * que celui de l'application : les feuilles sont donc mesurées avec les polices, les jetons
 * de style et la logique de densité réels, jamais avec une copie du gabarit.
 *
 * Une seule ouvrière : les mesures géométriques dépendent de la taille de la fenêtre et de
 * la disponibilité des polices, deux ressources que des exécutions parallèles se disputent.
 */
export default defineConfig({
  testDir: "./specs",
  outputDir: fileURLToPath(new URL("../test-output/.playwright", import.meta.url)),
  fullyParallel: false,
  workers: 1,
  reporter: [["list"], ["json", { outputFile: "../test-output/playwright-report.json" }]],
  timeout: 60_000,
  use: {
    baseURL: "http://localhost:1420",
    // 210 mm à 96 ppp font 794 px : la fenêtre doit porter la feuille entière, sinon la
    // mesure de débordement lit une contrainte de la fenêtre et non du gabarit.
    viewport: { width: 1100, height: 1400 },
    deviceScaleFactor: 2,
  },
  webServer: {
    command: "npm run dev",
    cwd: racine,
    url: "http://localhost:1420",
    reuseExistingServer: true,
    timeout: 120_000,
  },
});
