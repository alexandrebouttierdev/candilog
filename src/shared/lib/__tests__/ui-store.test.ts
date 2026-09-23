import { beforeEach, describe, expect, it } from "vitest";
import { applyTheme, useUiStore } from "../ui-store";

beforeEach(() => {
  document.documentElement.removeAttribute("data-theme");
  useUiStore.setState({ theme: "system", toasts: [] });
});

describe("préférence de thème", () => {
  it("force le thème clair ou sombre par un attribut", () => {
    applyTheme("dark");
    expect(document.documentElement.getAttribute("data-theme")).toBe("dark");

    applyTheme("light");
    expect(document.documentElement.getAttribute("data-theme")).toBe("light");
  });

  it("retire l'attribut en mode système", () => {
    // C'est l'absence d'attribut qui laisse jouer `prefers-color-scheme` : y écrire
    // « system » figerait le thème sur la valeur par défaut des feuilles de style.
    applyTheme("dark");
    applyTheme("system");
    expect(document.documentElement.hasAttribute("data-theme")).toBe(false);
  });
});

describe("file de notifications", () => {
  it("n'affiche qu'une notification à la fois : la nouvelle remplace la précédente", () => {
    // Décision E15 du design : deux actions rapides ne doivent jamais empiler deux bandeaux.
    const { notify } = useUiStore.getState();
    notify({ tone: "success", title: "Candidature enregistrée" });
    const premiere = useUiStore.getState().toasts[0]!;
    notify({ tone: "error", title: "Enregistrement impossible" });

    const { toasts } = useUiStore.getState();
    expect(toasts.map((toast) => toast.title)).toEqual(["Enregistrement impossible"]);
    expect(toasts[0]!.id).not.toBe(premiere.id);
  });

  it("ne retire que la notification visée", () => {
    const { notify } = useUiStore.getState();
    notify({ tone: "info", title: "Première" });
    const premiere = useUiStore.getState().toasts[0]!;
    notify({ tone: "info", title: "Seconde" });

    // Le minuteur de la première, remplacée, ne doit pas faire disparaître la seconde.
    useUiStore.getState().dismissToast(premiere.id);

    expect(useUiStore.getState().toasts.map((toast) => toast.title)).toEqual(["Seconde"]);
  });
});
