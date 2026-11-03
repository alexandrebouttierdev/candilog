import { describe, expect, it } from "vitest";
import type { CompanyResearchResult } from "@/shared/types/generated/ai";
import type { CompanyFormInput } from "../company-form.schema";
import {
  companyResearchResultSchema,
  companyReviewSchema,
  companySearchResultSchema,
  initialReview,
  replacesInput,
  reviewToFormPatch,
} from "../company-research.schema";

const vide = { value: null, origin: null, source_url: null, note: null } as const;

function resultat(): CompanyResearchResult {
  return {
    fields: {
      name: { value: "Nova Conseil", origin: "registry", source_url: "https://annuaire-entreprises.data.gouv.fr/etablissement/38012986612345", note: null },
      sector_id: { value: "5ec70000-0000-4000-8000-000000000007", origin: "ai", source_url: null, note: "à vérifier" },
      company_type_id: { ...vide },
      company_size: { value: "PME", origin: "registry", source_url: null, note: null },
      website: { value: "https://www.nova-conseil.fr", origin: "wikidata", source_url: null, note: null },
      city: { value: "Rennes", origin: "registry", source_url: null, note: null },
      address: { value: "4 Rue de la Paix 35000 Rennes", origin: "registry", source_url: null, note: null },
    },
    sources: [{ label: "Annuaire", url: "https://annuaire-entreprises.data.gouv.fr/etablissement/38012986612345" }],
    warnings: [],
    ai_used: true,
  };
}

const formulaire = (patch: Partial<CompanyFormInput> = {}): CompanyFormInput => ({
  name: "Nova",
  sector_id: "",
  company_type_id: "",
  company_size: "UNKNOWN",
  website: "",
  city: "",
  address: "",
  notes: "",
  ...patch,
});

describe("charge utile de la recherche d'entreprise", () => {
  it("accepte une réponse conforme", () => {
    expect(companyResearchResultSchema.safeParse(resultat()).success).toBe(true);
    expect(
      companySearchResultSchema.safeParse({ candidates: [], confident: false, truncated: false }).success,
    ).toBe(true);
  });

  it("refuse un lien qui n'est pas du Web et un SIRET mal formé", () => {
    const forge = resultat();
    forge.sources[0] = { label: "x", url: "javascript:alert(1)" };
    expect(companyResearchResultSchema.safeParse(forge).success).toBe(false);
    expect(
      companySearchResultSchema.safeParse({
        candidates: [{ siret: "123", siren: "123456789" }],
        confident: false,
        truncated: false,
      }).success,
    ).toBe(false);
  });
});

describe("vérification avant remplissage", () => {
  it("préserve ce que l'utilisateur a déjà saisi", () => {
    const revue = initialReview(resultat(), formulaire({ city: "Saint-Malo" }));
    // Nom déjà saisi, ville déjà saisie : proposés mais non appliqués par défaut.
    expect(revue.name).toEqual({ apply: false, value: "Nova Conseil" });
    expect(revue.city).toEqual({ apply: false, value: "Rennes" });
    // Champs vides : appliqués par défaut.
    expect(revue.address.apply).toBe(true);
    expect(revue.company_size.apply).toBe(true);
    // Rien de trouvé : rien à appliquer.
    expect(revue.company_type_id).toEqual({ apply: false, value: "" });
    expect(replacesInput("city", formulaire({ city: "Saint-Malo" }))).toBe(true);
    expect(replacesInput("company_size", formulaire())).toBe(false);
  });

  it("ne verse dans le formulaire que les champs cochés", () => {
    const revue = companyReviewSchema.parse({
      ...initialReview(resultat(), formulaire()),
      website: { apply: false, value: "https://www.nova-conseil.fr" },
    });
    expect(reviewToFormPatch(revue)).toEqual({
      sector_id: "5ec70000-0000-4000-8000-000000000007",
      company_size: "PME",
      city: "Rennes",
      address: "4 Rue de la Paix 35000 Rennes",
    });
  });

  it("refuse un nom vidé ou un site modifié en lien invalide", () => {
    const base = initialReview(resultat(), formulaire());
    const nomVide = companyReviewSchema.safeParse({ ...base, name: { apply: true, value: "  " } });
    expect(nomVide.success).toBe(false);
    const site = companyReviewSchema.safeParse({ ...base, website: { apply: true, value: "ftp://nova" } });
    expect(site.success).toBe(false);
    const nonApplique = companyReviewSchema.safeParse({ ...base, website: { apply: false, value: "ftp://nova" } });
    expect(nonApplique.success).toBe(true);
  });
});
