import { beforeEach, describe, expect, it } from "vitest";
import {
  animationsEnabled,
  applyDisplayPrefs,
  density,
  setAnimationsEnabled,
  setDensity,
} from "../display-prefs";

beforeEach(() => {
  window.localStorage.clear();
  document.documentElement.removeAttribute("data-density");
  document.documentElement.removeAttribute("data-motion");
});

describe("préférences d'affichage", () => {
  it("part de la densité confortable et des animations actives", () => {
    expect(density()).toBe("comfortable");
    expect(animationsEnabled()).toBe(true);
    applyDisplayPrefs();
    expect(document.documentElement.hasAttribute("data-density")).toBe(false);
    expect(document.documentElement.hasAttribute("data-motion")).toBe(false);
  });

  it("applique la densité compacte et la retire au retour en confortable", () => {
    setDensity("compact");
    expect(document.documentElement.getAttribute("data-density")).toBe("compact");
    setDensity("comfortable");
    expect(document.documentElement.hasAttribute("data-density")).toBe(false);
  });

  it("coupe les animations par un attribut que les jetons traduisent en durées nulles", () => {
    setAnimationsEnabled(false);
    expect(animationsEnabled()).toBe(false);
    expect(document.documentElement.getAttribute("data-motion")).toBe("off");
    setAnimationsEnabled(true);
    expect(document.documentElement.hasAttribute("data-motion")).toBe(false);
  });
});
