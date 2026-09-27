import { describe, expect, it } from "vitest";
import type { ApplicationFilter } from "@/shared/types/generated/applications";
import { EMPTY_FILTER } from "../schemas/application-filter.schema";
import { groupFilter, groupingLabel, nextGrouping } from "../grouping";

const base: ApplicationFilter = { ...EMPTY_FILTER, search: "", sort: "date", descending: true, ids: [] };

describe("regroupement de la liste", () => {
  it("parcourt statut, entreprise, contrat puis revient au statut", () => {
    expect(nextGrouping("status")).toBe("company");
    expect(nextGrouping("company")).toBe("contract");
    expect(nextGrouping("contract")).toBe("status");
    expect(groupingLabel("company")).toBe("entreprise");
  });

  it("restreint le filtre au groupe et lève l'inversion du même champ", () => {
    const filtre = { ...base, contract_type_code: ["CDI"], excluded: ["contract_type" as const, "status" as const] };

    const groupe = groupFilter(filtre, "contract", "CDD");

    expect(groupe.contract_type_code).toEqual(["CDD"]);
    expect(groupe.excluded).toEqual(["status"]);
    expect(groupFilter(base, "company", "e1").company_id).toBe("e1");
  });
});
