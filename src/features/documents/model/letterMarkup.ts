/**
 * Balisage restreint du corps d'une lettre, jumeau du module Rust `letter_markup`.
 *
 * Les deux existent parce que les deux runtimes doivent lire le même contenu : l'éditeur
 * pour l'afficher, le Rust pour l'assainir et l'imprimer. La grammaire est volontairement
 * minuscule — paragraphe, gras, souligné, taille, alignement — pour rester tenable des deux
 * côtés et n'accepter que ce que le PDF sait rendre.
 */

export type LetterAlign = "left" | "center" | "right";
export type LetterSize = "small" | "normal" | "large";

export interface LetterRun {
  text: string;
  bold: boolean;
  underline: boolean;
}

export interface LetterParagraph {
  runs: LetterRun[];
  align: LetterAlign;
  size: LetterSize;
}

/** Lit un corps de lettre, balisé ou écrit avant l'éditeur (texte brut). */
export function parseLetter(content: string): LetterParagraph[] {
  if (!content.includes("<p")) {
    return content
      .split("\n\n")
      .map((bloc) => bloc.trim())
      .filter((bloc) => bloc !== "")
      .map((bloc) => paragraphe([{ text: bloc, bold: false, underline: false }]));
  }
  const document_ = new DOMParser().parseFromString(content, "text/html");
  return blocs(document_.body);
}

/** Réécrit des paragraphes dans le balisage canonique : c'est cette forme qui est stockée. */
export function toMarkup(paragraphs: LetterParagraph[]): string {
  return paragraphs
    .map((paragraph) => {
      const align = paragraph.align === "left" ? "" : ` align="${paragraph.align}"`;
      const size = paragraph.size === "normal" ? "" : ` size="${paragraph.size}"`;
      const corps = paragraph.runs
        .filter((run) => run.text !== "")
        .map((run) => {
          let fragment = escaper(run.text);
          if (run.underline) fragment = `<u>${fragment}</u>`;
          if (run.bold) fragment = `<b>${fragment}</b>`;
          return fragment;
        })
        .join("");
      return `<p${align}${size}>${corps}</p>`;
    })
    .join("");
}

/**
 * Rend les paragraphes pour une zone éditable.
 *
 * Le balisage stocké ne suffit pas à l'affichage : `size` n'est pas un attribut que le
 * navigateur sait styler, il devient donc un `data-size` que la feuille de style reprend.
 * La chaîne produite ici vient de notre propre sérialiseur, jamais du contenu brut.
 */
export function toEditableHtml(paragraphs: LetterParagraph[]): string {
  return paragraphs
    .map((paragraph) => {
      const align = paragraph.align === "left" ? "" : ` style="text-align:${paragraph.align}"`;
      const size = paragraph.size === "normal" ? "" : ` data-size="${paragraph.size}"`;
      const corps = paragraph.runs
        .filter((run) => run.text !== "")
        .map((run) => {
          let fragment = escaper(run.text);
          if (run.underline) fragment = `<u>${fragment}</u>`;
          if (run.bold) fragment = `<b>${fragment}</b>`;
          return fragment;
        })
        .join("");
      return `<p${align}${size}>${corps}</p>`;
    })
    .join("");
}

/** Texte nu, pour compter les signes ou comparer deux versions. */
export function toPlainText(content: string): string {
  return parseLetter(content)
    .map((paragraph) => paragraph.runs.map((run) => run.text).join(""))
    .join("\n\n");
}

/**
 * Relit le contenu d'une zone éditable et le ramène au balisage canonique.
 *
 * C'est ici que le collage d'un traitement de texte est filtré : seules les intentions que
 * l'export sait honorer survivent, le reste du HTML tombe sans emporter les mots.
 */
export function markupFromDom(root: HTMLElement): string {
  const paragraphs = blocs(root);
  return toMarkup(paragraphs.length > 0 ? paragraphs : [paragraphe([])]);
}

function paragraphe(
  runs: LetterRun[],
  align: LetterAlign = "left",
  size: LetterSize = "normal",
): LetterParagraph {
  return { runs, align, size };
}

function blocs(root: HTMLElement | HTMLBodyElement): LetterParagraph[] {
  const paragraphs: LetterParagraph[] = [];
  const enfants = [...root.childNodes];
  // Un nœud texte à la racine survient dès la première frappe dans une zone vide : sans
  // ce rattrapage, la lettre perdrait ce que l'utilisateur vient d'écrire.
  const orphelins: LetterRun[] = [];
  for (const enfant of enfants) {
    if (enfant.nodeType === Node.TEXT_NODE) {
      const texte = enfant.textContent ?? "";
      if (texte.trim() !== "") orphelins.push({ text: texte, bold: false, underline: false });
      continue;
    }
    if (!(enfant instanceof HTMLElement)) continue;
    if (enfant.tagName === "BR") {
      paragraphs.push(paragraphe([]));
      continue;
    }
    for (const morceau of decouperSurLesRetours(enfant)) {
      paragraphs.push(paragraphe(fusionner(morceau), alignement(enfant), taille(enfant)));
    }
  }
  if (orphelins.length > 0) {
    paragraphs.unshift(paragraphe(fusionner(orphelins)));
  }
  return paragraphs;
}

/** Un `<br>` vaut une rupture de paragraphe : le PDF n'a pas d'autre saut de ligne. */
function decouperSurLesRetours(element: HTMLElement): LetterRun[][] {
  const morceaux: LetterRun[][] = [[]];
  const parcourir = (noeud: Node, bold: boolean, underline: boolean) => {
    for (const enfant of noeud.childNodes) {
      if (enfant.nodeType === Node.TEXT_NODE) {
        const texte = enfant.textContent ?? "";
        if (texte !== "") {
          morceaux[morceaux.length - 1]?.push({ text: texte, bold, underline });
        }
        continue;
      }
      if (!(enfant instanceof HTMLElement)) continue;
      if (enfant.tagName === "BR") {
        morceaux.push([]);
        continue;
      }
      parcourir(enfant, bold || estGras(enfant), underline || estSouligne(enfant));
    }
  };
  parcourir(element, estGras(element), estSouligne(element));
  return morceaux;
}

function fusionner(runs: LetterRun[]): LetterRun[] {
  const sortie: LetterRun[] = [];
  for (const run of runs) {
    const dernier = sortie[sortie.length - 1];
    if (dernier && dernier.bold === run.bold && dernier.underline === run.underline) {
      dernier.text += run.text;
      continue;
    }
    sortie.push({ ...run });
  }
  return sortie;
}

function estGras(element: HTMLElement): boolean {
  if (element.tagName === "B" || element.tagName === "STRONG") return true;
  const poids = element.style.fontWeight;
  return poids === "bold" || Number(poids) >= 600;
}

function estSouligne(element: HTMLElement): boolean {
  if (element.tagName === "U" || element.tagName === "INS") return true;
  return element.style.textDecorationLine.includes("underline")
    || element.style.textDecoration.includes("underline");
}

function alignement(element: HTMLElement): LetterAlign {
  const valeur = element.style.textAlign || element.getAttribute("align") || "";
  return valeur === "center" || valeur === "right" ? valeur : "left";
}

function taille(element: HTMLElement): LetterSize {
  const valeur = element.dataset["size"] ?? element.getAttribute("size") ?? "";
  return valeur === "small" || valeur === "large" ? valeur : "normal";
}

function escaper(value: string): string {
  return value.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}
