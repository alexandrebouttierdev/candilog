import { describe, expect, it } from "vitest";
import {
  markupFromDom,
  parseLetter,
  toMarkup,
  toPlainText,
} from "../letterMarkup";

function zone(html: string): HTMLElement {
  const element = document.createElement("div");
  element.innerHTML = html;
  return element;
}

describe("balisage de lettre", () => {
  it("lit une lettre écrite avant l'éditeur comme du texte brut", () => {
    const paragraphs = parseLetter("Madame, Monsieur,\n\nJe vous écris.");

    expect(paragraphs).toHaveLength(2);
    expect(paragraphs[0]?.runs[0]?.text).toBe("Madame, Monsieur,");
    expect(paragraphs[0]?.align).toBe("left");
    expect(paragraphs[0]?.size).toBe("normal");
  });

  it("relit le gras, le souligné, l'alignement et la taille", () => {
    const paragraphs = parseLetter(
      '<p align="center" size="large">Bonjour <b>Nova</b> et <u>Atlas</u></p>',
    );

    expect(paragraphs[0]?.align).toBe("center");
    expect(paragraphs[0]?.size).toBe("large");
    expect(paragraphs[0]?.runs).toEqual([
      { text: "Bonjour ", bold: false, underline: false },
      { text: "Nova", bold: true, underline: false },
      { text: " et ", bold: false, underline: false },
      { text: "Atlas", bold: false, underline: true },
    ]);
  });

  it("écrit un balisage stable par aller-retour", () => {
    const source = '<p align="right" size="small">Je suis <b>disponible</b></p><p>Cordialement,</p>';

    expect(toMarkup(parseLetter(source))).toBe(source);
  });

  it("ramène le style d'un collage extérieur à ce que l'export sait rendre", () => {
    // Un collage de traitement de texte arrive en <span style>, pas en <b> : l'intention
    // doit être conservée, le reste du HTML écarté.
    const markup = markupFromDom(
      zone('<p style="text-align:center"><span style="font-weight:700">Nova</span><span style="color:red"> vif</span></p>'),
    );

    expect(markup).toBe('<p align="center"><b>Nova</b> vif</p>');
  });

  it("garde le texte tapé à la racine d'une zone encore vide", () => {
    expect(markupFromDom(zone("Premiers mots"))).toBe("<p>Premiers mots</p>");
  });

  it("coupe un retour à la ligne forcé en deux paragraphes, comme le PDF", () => {
    expect(markupFromDom(zone("<div>Alex Exemple<br>Rennes</div>"))).toBe(
      "<p>Alex Exemple</p><p>Rennes</p>",
    );
  });

  it("échappe les chevrons du texte au lieu de les prendre pour des balises", () => {
    const markup = markupFromDom(zone("<p>5 &lt; 7</p>"));

    expect(markup).toBe("<p>5 &lt; 7</p>");
    expect(toPlainText(markup)).toBe("5 < 7");
  });
});
