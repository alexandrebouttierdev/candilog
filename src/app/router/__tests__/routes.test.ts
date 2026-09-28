import { describe, expect, it } from "vitest";
import { DESTINATIONS, LEGACY_REDIRECTS, activeTab, destinationForPath } from "../routes";
import { PATHS } from "@/shared/lib/paths";

describe("carte des destinations", () => {
  it("reprend les six destinations du design, sans les Réglages", () => {
    // Les Réglages sont une surcouche (`DECISIONS.md` B7), pas une destination.
    expect(DESTINATIONS.map((destination) => destination.key)).toEqual([
      "today",
      "applications",
      "relations",
      "documents",
      "ai",
      "profile",
    ]);
  });

  it("n'attribue jamais deux fois la même lettre au raccourci G", () => {
    const keys = DESTINATIONS.flatMap((destination) => (destination.goKey ? [destination.goKey] : []));
    expect(new Set(keys).size).toBe(keys.length);
    expect(keys).toEqual(["a", "c", "r", "d", "p"]);
  });

  it("rattache un chemin et ses sous-vues à sa destination", () => {
    expect(destinationForPath(PATHS.calendar).key).toBe("applications");
    expect(destinationForPath(PATHS.contacts).key).toBe("relations");
    expect(destinationForPath(PATHS.generateResume).key).toBe("documents");
    expect(destinationForPath(PATHS.ai).key).toBe("ai");
  });

  it("ne confond pas un préfixe avec un mot plus long", () => {
    // « /aide » ne doit pas activer « /ai ».
    expect(destinationForPath("/aide").key).toBe("today");
    expect(destinationForPath("/").key).toBe("today");
  });

  it("choisit l'onglet de vue exact, sinon le premier", () => {
    const candidatures = destinationForPath(PATHS.applications);
    expect(activeTab(candidatures, PATHS.applicationsKanban)?.label).toBe("Kanban");
    expect(activeTab(candidatures, "/applications/inconnue")?.label).toBe("Liste");
  });
});

describe("anciens chemins", () => {
  it("redirige chaque chemin v1 vers une destination v2", () => {
    for (const target of Object.values(LEGACY_REDIRECTS)) {
      expect(Object.values(PATHS)).toContain(target);
    }
    expect(LEGACY_REDIRECTS["/tracking/applications"]).toBe(PATHS.applications);
    expect(LEGACY_REDIRECTS["/relations/network"]).toBe(PATHS.contacts);
  });
});

describe("écrans routés", () => {
  it("donne une page à chaque destination et à chaque onglet de vue", async () => {
    const { ROUTES } = await import("../AppRouter");
    const paths = new Set(ROUTES.map((route) => (route.index ? "/" : `/${route.path ?? ""}`)));
    for (const destination of DESTINATIONS) {
      expect(paths).toContain(destination.path);
      for (const tab of destination.tabs ?? []) expect(paths).toContain(tab.path);
    }
  });
});
