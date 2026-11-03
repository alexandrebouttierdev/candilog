import { describe, expect, it } from "vitest";
import { FACTS } from "../providerFacts";
import { PROVIDERS } from "../providers";

describe("faits affichés par fournisseur", () => {
  it("décrit chaque fournisseur proposé, avec trois jauges", () => {
    for (const provider of PROVIDERS) {
      const facts = FACTS[provider.id](provider.label);
      expect(facts.description.length, provider.id).toBeGreaterThan(0);
      expect(facts.meters.map((meter) => meter.label)).toEqual(["Confidentialité", "Coût", "Hors connexion"]);
    }
  });

  it("dit que rien ne sort avec l'IA locale", () => {
    expect(FACTS.candilog_local("IA locale").meters[0]?.value).toBe("Rien ne sort");
  });

  it("nomme le destinataire d'un fournisseur distant", () => {
    const facts = FACTS.claude("Anthropic");
    expect(facts.description).toContain("Anthropic");
    expect(facts.meters[0]).toMatchObject({ value: "Envoyé à Anthropic", tone: "bg-st-a" });
  });
});
