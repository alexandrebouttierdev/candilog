import { describe, expect, it } from "vitest";
import { isTypingTarget, matches } from "../useShortcut";

function input(type: string): HTMLInputElement {
  const element = document.createElement("input");
  element.type = type;
  return element;
}

describe("isTypingTarget", () => {
  it("considère les champs de saisie de texte comme une frappe", () => {
    for (const type of ["text", "search", "email", "number", "date", "password"]) {
      expect(isTypingTarget(input(type))).toBe(true);
    }
    expect(isTypingTarget(document.createElement("textarea"))).toBe(true);
    expect(isTypingTarget(document.createElement("select"))).toBe(true);
  });

  it("laisse passer les raccourcis depuis une case à cocher ou un bouton radio", () => {
    // Cocher une ligne puis taper `S` doit changer le statut des lignes cochées : le focus
    // resté sur la case ne reçoit pas de texte.
    for (const type of ["checkbox", "radio", "range", "button"]) {
      expect(isTypingTarget(input(type))).toBe(false);
    }
    expect(isTypingTarget(document.createElement("button"))).toBe(false);
  });
});

describe("matches", () => {
  it("reconnaît une lettre seule, sans modificateur", () => {
    expect(matches("s", new KeyboardEvent("keydown", { key: "s" }))).toBe(true);
    expect(matches("s", new KeyboardEvent("keydown", { key: "s", metaKey: true }))).toBe(false);
  });

  it("reconnaît Échap", () => {
    expect(matches("escape", new KeyboardEvent("keydown", { key: "Escape" }))).toBe(true);
  });
});
