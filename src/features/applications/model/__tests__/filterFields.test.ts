import { describe, expect, it } from "vitest";
import { EMPTY_FILTER } from "../schemas/application-filter.schema";
import { chipsOf, daysAgo, removeField, toggleExcluded, toggleValue } from "../filterFields";

const label = (_key: string, value: string) => value;

describe("champs du menu de filtre", () => {
  it("décocher la dernière valeur retire aussi l'inversion du critère", () => {
    const filtre = { ...EMPTY_FILTER, status: ["REFUS" as const], excluded: ["status" as const] };
    const suivant = toggleValue(filtre, "status", "REFUS");
    expect(suivant.status).toEqual([]);
    expect(suivant.excluded).toEqual([]);
  });

  it("n'inverse pas une borne", () => {
    const filtre = { ...EMPTY_FILTER, min_weekly_hours: 24 };
    expect(toggleExcluded(filtre, "hours")).toBe(filtre);
  });

  it("retire une période en effaçant ses deux bornes", () => {
    const filtre = { ...EMPTY_FILTER, start_date: "2026-08-01", end_date: "2026-09-01" };
    expect(removeField(filtre, "sent")).toMatchObject({ start_date: null, end_date: null });
  });

  it("formule les puces selon le critère et son inversion", () => {
    const puces = chipsOf(
      {
        ...EMPTY_FILTER,
        contract_type_code: ["CDI", "CDD"],
        job_title: "stage",
        min_weekly_hours: 35,
        end_date: "2026-09-09",
        excluded: ["job_title"],
      },
      label,
    );
    expect(puces.map((puce) => `${puce.field} ${puce.op} ${puce.value}`)).toEqual([
      "Contrat est CDI, CDD",
      "Envoyée jusqu'au 09-09-2026",
      "Poste ne contient pas stage",
      "Heures ≥ 35 h",
    ]);
  });

  it("calcule une date passée en jours calendaires locaux", () => {
    expect(daysAgo(14, new Date(2026, 8, 23))).toBe("2026-09-09");
    expect(daysAgo(30, new Date(2026, 2, 15))).toBe("2026-02-13");
  });
});
