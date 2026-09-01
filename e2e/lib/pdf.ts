/**
 * Contrôles du PDF réellement produit par Candilog.
 *
 * Rien n'est réinterprété depuis le HTML : le fichier exporté est ouvert, sa structure lue,
 * son texte extrait avec les rectangles de chaque mot, et ses pages rendues en image. Un
 * gabarit peut être irréprochable à l'écran et fautif à l'export ; c'est ici qu'on le voit.
 *
 * L'outillage est celui de Poppler (`pdfinfo`, `pdftotext`, `pdftoppm`), déjà présent sur
 * les postes de développement Linux et sans dépendance à ajouter au projet.
 */
import { execFileSync } from "node:child_process";
import { existsSync, mkdtempSync, readFileSync, readdirSync, rmSync, statSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

export type Mot = { texte: string; xMin: number; yMin: number; xMax: number; yMax: number; page: number };

export type RapportPdf = {
  fichier: string;
  existe: boolean;
  octets: number;
  entete: boolean;
  pages: number;
  largeurPt: number;
  hauteurPt: number;
  texte: string;
  mots: number;
  pagesVides: number[];
  horsMarges: { mot: string; page: number; cote: string; depassement: number }[];
  chevauchements: { premier: string; second: string; page: number; recouvrement: number }[];
  captures: string[];
  erreurs: string[];
};

/** Marge minimale exigée sur chaque bord, en points (≈ 8 mm). */
const MARGE_MIN_PT = 22.7;
/** Tolérance de recouvrement entre deux mots voisins, en points. */
const TOLERANCE_PT = 1.2;

function outil(commande: string, args: string[]): string {
  return execFileSync(commande, args, { encoding: "utf8", maxBuffer: 64 * 1024 * 1024 });
}

/** Lit les rectangles de chaque mot dans le repère de la page, page par page. */
function mots(fichier: string): Mot[] {
  const xml = outil("pdftotext", ["-bbox-layout", fichier, "-"]);
  const resultat: Mot[] = [];
  let page = 0;
  for (const bloc of xml.split(/<page\b/).slice(1)) {
    page += 1;
    for (const trouve of bloc.matchAll(
      /<word xMin="([\d.]+)" yMin="([\d.]+)" xMax="([\d.]+)" yMax="([\d.]+)">([\s\S]*?)<\/word>/g,
    )) {
      const texte = trouve[5]!
        .replace(/&amp;/g, "&")
        .replace(/&lt;/g, "<")
        .replace(/&gt;/g, ">")
        .replace(/&quot;/g, '"')
        .replace(/&apos;/g, "'");
      resultat.push({
        texte,
        xMin: Number(trouve[1]),
        yMin: Number(trouve[2]),
        xMax: Number(trouve[3]),
        yMax: Number(trouve[4]),
        page,
      });
    }
  }
  return resultat;
}

/**
 * Ouvre, mesure et rend un PDF exporté.
 *
 * `captureDans` reçoit les pages en PNG : c'est le résultat imprimé qu'on regarde, pas une
 * reconstitution du HTML.
 */
export function controlerPdf(fichier: string, captureDans?: string): RapportPdf {
  const rapport: RapportPdf = {
    fichier,
    existe: existsSync(fichier),
    octets: 0,
    entete: false,
    pages: 0,
    largeurPt: 0,
    hauteurPt: 0,
    texte: "",
    mots: 0,
    pagesVides: [],
    horsMarges: [],
    chevauchements: [],
    captures: [],
    erreurs: [],
  };
  if (!rapport.existe) {
    rapport.erreurs.push("fichier absent");
    return rapport;
  }
  rapport.octets = statSync(fichier).size;
  if (rapport.octets === 0) rapport.erreurs.push("fichier vide");
  rapport.entete = readFileSync(fichier).subarray(0, 5).toString("latin1") === "%PDF-";
  if (!rapport.entete) rapport.erreurs.push("en-tête PDF absent");

  try {
    const info = outil("pdfinfo", [fichier]);
    rapport.pages = Number(/^Pages:\s+(\d+)$/m.exec(info)?.[1] ?? 0);
    const taille = /^Page size:\s+([\d.]+) x ([\d.]+) pts/m.exec(info);
    rapport.largeurPt = Number(taille?.[1] ?? 0);
    rapport.hauteurPt = Number(taille?.[2] ?? 0);
  } catch (erreur) {
    rapport.erreurs.push(`pdfinfo : ${erreur instanceof Error ? erreur.message : String(erreur)}`);
    return rapport;
  }
  if (rapport.pages === 0) rapport.erreurs.push("aucune page détectable");

  try {
    rapport.texte = outil("pdftotext", ["-layout", fichier, "-"]);
  } catch (erreur) {
    rapport.erreurs.push(`pdftotext : ${erreur instanceof Error ? erreur.message : String(erreur)}`);
  }
  if (rapport.texte.trim() === "") rapport.erreurs.push("aucun texte extractible");

  let boites: Mot[] = [];
  try {
    boites = mots(fichier);
  } catch (erreur) {
    rapport.erreurs.push(`pdftotext -bbox : ${erreur instanceof Error ? erreur.message : String(erreur)}`);
  }
  rapport.mots = boites.length;

  for (let page = 1; page <= rapport.pages; page += 1) {
    if (!boites.some((mot) => mot.page === page)) rapport.pagesVides.push(page);
  }

  for (const mot of boites) {
    const cotes: [string, number][] = [
      ["gauche", MARGE_MIN_PT - mot.xMin],
      ["droite", mot.xMax - (rapport.largeurPt - MARGE_MIN_PT)],
      ["haut", MARGE_MIN_PT - mot.yMin],
      ["bas", mot.yMax - (rapport.hauteurPt - MARGE_MIN_PT)],
    ];
    for (const [cote, depassement] of cotes) {
      if (depassement > TOLERANCE_PT) {
        rapport.horsMarges.push({ mot: mot.texte, page: mot.page, cote, depassement: Number(depassement.toFixed(2)) });
      }
    }
  }

  // Deux mots qui se recouvrent sur les deux axes sont imprimés l'un sur l'autre. Les mots
  // d'une même ligne se touchent par construction : seul un recouvrement franc compte.
  for (let index = 0; index < boites.length; index += 1) {
    for (let autre = index + 1; autre < boites.length; autre += 1) {
      const a = boites[index]!;
      const b = boites[autre]!;
      if (a.page !== b.page) continue;
      if (b.yMin > a.yMax) break;
      const x = Math.min(a.xMax, b.xMax) - Math.max(a.xMin, b.xMin);
      const y = Math.min(a.yMax, b.yMax) - Math.max(a.yMin, b.yMin);
      // Les rectangles rendus par Poppler sont ceux de la police — hampe et jambage
      // compris —, pas ceux de l'encre. Sous un interlignage serré (`line-height: 1.06`
      // du bloc d'identité) deux lignes successives s'y recouvrent de quelques points sans
      // qu'aucun trait ne se touche. Un vrai surimpressionnement recouvre l'essentiel de la
      // hauteur du mot, pas sa marge typographique.
      const hauteur = Math.min(a.yMax - a.yMin, b.yMax - b.yMin);
      if (x > TOLERANCE_PT && y > TOLERANCE_PT && y > hauteur * 0.4) {
        rapport.chevauchements.push({
          premier: a.texte,
          second: b.texte,
          page: a.page,
          recouvrement: Number(Math.min(x, y).toFixed(2)),
        });
      }
    }
  }

  if (captureDans) {
    const temporaire = mkdtempSync(join(tmpdir(), "candilog-pdf-"));
    try {
      outil("pdftoppm", ["-png", "-r", "110", fichier, join(temporaire, "page")]);
      for (const nom of readdirSync(temporaire).sort()) {
        const cible = join(captureDans, `${nom.replace("page-", "").replace(".png", "")}.png`);
        execFileSync("cp", [join(temporaire, nom), cible]);
        rapport.captures.push(cible);
      }
    } catch (erreur) {
      rapport.erreurs.push(`pdftoppm : ${erreur instanceof Error ? erreur.message : String(erreur)}`);
    } finally {
      rmSync(temporaire, { recursive: true, force: true });
    }
  }
  return rapport;
}

/**
 * Caractères du document source que le PDF n'a pas su imprimer.
 *
 * L'export remplace par une espace tout caractère absent des polices embarquées : un accent,
 * une ligature ou un tiret cadratin disparu se lit ici, et nulle part ailleurs.
 */
export function caracteresPerdus(attendu: string, extrait: string): string[] {
  const presents = new Set(extrait.normalize("NFC"));
  const perdus = new Set<string>();
  for (const caractere of attendu.normalize("NFC")) {
    if (/\s/.test(caractere)) continue;
    if (!presents.has(caractere)) perdus.add(caractere);
  }
  return [...perdus];
}
