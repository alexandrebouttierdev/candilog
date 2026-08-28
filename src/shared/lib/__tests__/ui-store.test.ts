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
  it("empile les notifications avec un identifiant propre", () => {
    const { notify } = useUiStore.getState();
    notify({ tone: "success", title: "Candidature enregistrée" });
    notify({ tone: "error", title: "Enregistrement impossible" });

    const { toasts } = useUiStore.getState();
    expect(toasts).toHaveLength(2);
    expect(new Set(toasts.map((toast) => toast.id)).size).toBe(2);
  });

  it("ne retire que la notification visée", () => {
    const { notify } = useUiStore.getState();
    notify({ tone: "info", title: "Première" });
    notify({ tone: "info", title: "Seconde" });

    const [premiere] = useUiStore.getState().toasts;
    useUiStore.getState().dismissToast(premiere!.id);

    expect(useUiStore.getState().toasts.map((toast) => toast.title)).toEqual(["Seconde"]);
  });
});
