/**
 * Contrôles génériques d'une feuille A4 rendue dans le navigateur.
 *
 * Tout est mesuré dans la page, sur la géométrie réelle : aucune règle n'est déduite du
 * code du gabarit. Les fonctions sont volontairement indépendantes du document contrôlé —
 * une correction ne doit jamais viser un profil, seulement une cause commune.
 */
import type { Page } from "@playwright/test";

export type Debordement = {
  selecteur: string;
  axe: "horizontal" | "vertical" | "hors-colonne" | "hors-page";
  depassement: number;
  extrait: string;
};

export type Collision = {
  premier: string;
  second: string;
  recouvrement: { x: number; y: number };
};

export type Placeholder = { motif: string; extrait: string };

/** Tolérance en pixels : l'arrondi sous-pixel du moteur de rendu n'est pas un défaut. */
const TOLERANCE = 1.5;

/** Valeurs qui trahissent une variable de gabarit non résolue ou une donnée technique. */
const MOTIFS_PARASITES: [string, RegExp][] = [
  ["undefined", /\bundefined\b/],
  ["null", /\bnull\b/],
  ["NaN", /\bNaN\b/],
  ["[object Object]", /\[object [A-Z]\w*\]/],
  ["Lorem ipsum", /lorem ipsum/i],
  ["TODO", /\bTODO\b/],
  ["accolades de gabarit", /\{\{|\}\}/],
  ["interpolation", /\$\{/],
  ["balise HTML brute", /<\/?(p|b|u|div|span|br)\b[^>]*>/i],
  ["caractère de remplacement", /[�□]/],
];

/**
 * Débordements réels de la feuille.
 *
 * Trois familles : un contenu plus large que son conteneur, un contenu plus haut que son
 * conteneur, et un bloc sorti de la surface imprimable de la feuille. Un conteneur
 * volontairement défilant (`overflow: auto|scroll`) est ignoré : son défilement est un
 * choix, pas un accident.
 */
export async function debordements(page: Page, feuille: string): Promise<Debordement[]> {
  return page.evaluate(
    ({ feuille, tolerance }) => {
      const racine = document.querySelector(feuille);
      if (!racine) return [];

      const chemin = (element: Element): string => {
        const parts: string[] = [];
        let courant: Element | null = element;
        while (courant && parts.length < 5) {
          const classes = courant.className?.toString().trim().split(/\s+/).slice(0, 2).join(".");
          parts.unshift(classes ? `${courant.tagName.toLowerCase()}.${classes}` : courant.tagName.toLowerCase());
          if (courant === racine) break;
          courant = courant.parentElement;
        }
        return parts.join(" > ");
      };
      const extrait = (element: Element) =>
        (element.textContent ?? "").trim().replace(/\s+/g, " ").slice(0, 70);

      const trouves: {
        selecteur: string;
        axe: "horizontal" | "vertical" | "hors-colonne" | "hors-page";
        depassement: number;
        extrait: string;
      }[] = [];

      // Surface imprimable : la boîte de contenu de la feuille, marges déduites.
      const styleFeuille = getComputedStyle(racine);
      const cadre = racine.getBoundingClientRect();
      const surface = {
        left: cadre.left + parseFloat(styleFeuille.paddingLeft) + parseFloat(styleFeuille.borderLeftWidth),
        right: cadre.right - parseFloat(styleFeuille.paddingRight) - parseFloat(styleFeuille.borderRightWidth),
        top: cadre.top + parseFloat(styleFeuille.paddingTop) + parseFloat(styleFeuille.borderTopWidth),
        bottom: cadre.bottom - parseFloat(styleFeuille.paddingBottom) - parseFloat(styleFeuille.borderBottomWidth),
      };

      for (const element of [racine, ...racine.querySelectorAll("*")]) {
        const style = getComputedStyle(element);
        if (style.display === "none" || style.visibility === "hidden") continue;
        // Un débordement n'est un défaut que s'il cache quelque chose. Sur un bloc en
        // `overflow: visible`, le contenu reste lisible : ce qui compte alors est qu'il ne
        // sorte pas de la page et ne recouvre pas un voisin — les deux autres contrôles.
        // Sans cette distinction, chaque titre à interlignage serré (`line-height: 1.02`)
        // était signalé parce que la boîte de ses glyphes dépasse sa boîte de ligne.
        const rogne = /hidden|clip/.test(`${style.overflowX} ${style.overflowY}`);
        const defilant = /auto|scroll/.test(`${style.overflowX} ${style.overflowY}`);
        if (rogne && !defilant) {
          const surLargeur = element.scrollWidth - element.clientWidth;
          const surHauteur = element.scrollHeight - element.clientHeight;
          if (element.clientWidth > 0 && surLargeur > tolerance) {
            trouves.push({ selecteur: chemin(element), axe: "horizontal", depassement: surLargeur, extrait: extrait(element) });
          }
          if (element.clientHeight > 0 && surHauteur > tolerance) {
            trouves.push({ selecteur: chemin(element), axe: "vertical", depassement: surHauteur, extrait: extrait(element) });
          }
        }
        if (element === racine) continue;
        const boite = element.getBoundingClientRect();
        if (boite.width === 0 && boite.height === 0) continue;

        // Un bloc plus large que sa colonne en sort, même sans rognage : c'est ainsi que
        // l'étiquette de section et le patronyme de la lettre débordaient sur le voisin.
        const parent = element.parentElement;
        if (parent && parent !== racine.parentElement && getComputedStyle(element).position === "static") {
          const styleParent = getComputedStyle(parent);
          const cadreParent = parent.getBoundingClientRect();
          const gauche = cadreParent.left + parseFloat(styleParent.paddingLeft) + parseFloat(styleParent.borderLeftWidth);
          const droite = cadreParent.right - parseFloat(styleParent.paddingRight) - parseFloat(styleParent.borderRightWidth);
          const sortie = Math.max(gauche - boite.left, boite.right - droite);
          if (sortie > tolerance) {
            trouves.push({ selecteur: chemin(element), axe: "hors-colonne", depassement: sortie, extrait: extrait(element) });
          }
        }

        // Un bloc hors de la surface imprimable sort de la page à l'export.
        const sortie = Math.max(
          surface.left - boite.left,
          boite.right - surface.right,
          surface.top - boite.top,
          boite.bottom - surface.bottom,
        );
        if (sortie > tolerance) {
          trouves.push({ selecteur: chemin(element), axe: "hors-page", depassement: sortie, extrait: extrait(element) });
        }
      }
      return trouves;
    },
    { feuille, tolerance: TOLERANCE },
  );
}

/**
 * Superpositions anormales entre blocs.
 *
 * Deux enfants successifs d'un même conteneur, tous deux de niveau bloc et dans le flux,
 * ne doivent jamais se recouvrir sur les deux axes à la fois. Les éléments positionnés
 * (`absolute`, `fixed`) et les éléments en ligne sont écartés : leur superposition peut
 * être voulue par le dessin.
 */
export async function collisions(page: Page, feuille: string): Promise<Collision[]> {
  return page.evaluate(
    ({ feuille, tolerance }) => {
      const racine = document.querySelector(feuille);
      if (!racine) return [];
      const chemin = (element: Element) => {
        const classes = element.className?.toString().trim().split(/\s+/).slice(0, 2).join(".");
        const texte = (element.textContent ?? "").trim().replace(/\s+/g, " ").slice(0, 40);
        return `${element.tagName.toLowerCase()}${classes ? `.${classes}` : ""}${texte ? ` « ${texte} »` : ""}`;
      };
      const trouves: { premier: string; second: string; recouvrement: { x: number; y: number } }[] = [];

      for (const conteneur of [racine, ...racine.querySelectorAll("*")]) {
        const enfants = [...conteneur.children].filter((enfant) => {
          const style = getComputedStyle(enfant);
          if (style.display === "none" || style.visibility === "hidden") return false;
          if (style.position === "absolute" || style.position === "fixed") return false;
          if (style.display.startsWith("inline") && style.display !== "inline-block") return false;
          const boite = enfant.getBoundingClientRect();
          return boite.width > 0 && boite.height > 0;
        });
        for (let index = 0; index + 1 < enfants.length; index += 1) {
          const a = enfants[index]!.getBoundingClientRect();
          const b = enfants[index + 1]!.getBoundingClientRect();
          const x = Math.min(a.right, b.right) - Math.max(a.left, b.left);
          const y = Math.min(a.bottom, b.bottom) - Math.max(a.top, b.top);
          if (x > tolerance && y > tolerance) {
            trouves.push({
              premier: chemin(enfants[index]!),
              second: chemin(enfants[index + 1]!),
              recouvrement: { x, y },
            });
          }
        }
      }
      return trouves;
    },
    { feuille, tolerance: TOLERANCE },
  );
}

/** Valeurs parasites visibles sur la feuille. */
export async function placeholders(page: Page, feuille: string): Promise<Placeholder[]> {
  const texte = await page.locator(feuille).innerText();
  return MOTIFS_PARASITES.flatMap(([motif, regex]) => {
    const trouve = regex.exec(texte);
    if (!trouve) return [];
    const debut = Math.max(0, trouve.index - 40);
    return [{ motif, extrait: texte.slice(debut, trouve.index + trouve[0].length + 40).replace(/\s+/g, " ") }];
  });
}

/**
 * Familles de polices réellement appliquées, et caractères que la page ne sait pas rendre.
 *
 * Le gabarit annonce IBM Plex : si le navigateur retombe sur une police système, l'aperçu
 * cesse de correspondre au PDF, qui embarque ces fichiers.
 */
export async function polices(page: Page, feuille: string) {
  return page.evaluate((feuille) => {
    const racine = document.querySelector(feuille);
    if (!racine) return { chargees: [] as string[], manquantes: [] as string[], familles: [] as string[] };
    const familles = new Set<string>();
    for (const element of racine.querySelectorAll("*")) {
      if ((element.textContent ?? "").trim() === "") continue;
      familles.add(getComputedStyle(element).fontFamily.split(",")[0]!.replace(/["']/g, "").trim());
    }
    const attendues = ['1em "IBM Plex Sans"', '1em "IBM Plex Mono"', '600 1em "IBM Plex Sans"', '500 1em "IBM Plex Mono"'];
    return {
      familles: [...familles],
      chargees: attendues.filter((police) => document.fonts.check(police)),
      manquantes: attendues.filter((police) => !document.fonts.check(police)),
    };
  }, feuille);
}

/** Journaux du navigateur, branchés avant toute navigation. */
export function journalDuNavigateur(page: Page) {
  const erreurs: string[] = [];
  page.on("console", (message) => {
    if (message.type() === "error") erreurs.push(`console.error: ${message.text()}`);
  });
  page.on("pageerror", (erreur) => erreurs.push(`pageerror: ${erreur.message}`));
  page.on("requestfailed", (requete) => {
    const echec = requete.failure()?.errorText ?? "inconnu";
    // Une requête annulée par la navigation n'est pas une ressource manquante.
    if (echec.includes("ERR_ABORTED")) return;
    erreurs.push(`requestfailed: ${requete.url()} (${echec})`);
  });
  return erreurs;
}
