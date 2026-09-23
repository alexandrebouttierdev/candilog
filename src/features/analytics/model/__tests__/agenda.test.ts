import { describe, expect, it } from "vitest";
import type { AgendaItem } from "@/shared/types/generated/analytics";
import { daysBetween, horizonOf, longDay, shortWeekday, splitAgenda } from "../agenda";

function echeance(id: string, date: string, kind: AgendaItem["kind"] = "follow_up"): AgendaItem {
  return {
    kind,
    id,
    application_id: `candidature-${id}`,
    reference_number: 97,
    job_title: "Ingénieur systèmes",
    company_name: "Groupe Astréa",
    status: "RELANCEE",
    date,
    detail: "Email",
    location: null,
  };
}

describe("agenda — horizons d'Aujourd'hui", () => {
  const today = "2026-09-23";

  it("range une relance passée en retard, celle du jour aujourd'hui, la suivante dans la semaine", () => {
    expect(horizonOf(echeance("a", "2026-09-18"), today)).toBe("late");
    expect(horizonOf(echeance("b", "2026-09-23"), today)).toBe("today");
    expect(horizonOf(echeance("c", "2026-09-25"), today)).toBe("week");
  });

  it("lit la date d'un entretien horodaté sans tenir compte de l'heure", () => {
    expect(horizonOf(echeance("d", "2026-09-23T14:30:00", "interview"), today)).toBe("today");
  });

  it("répartit sans changer l'ordre chronologique reçu", () => {
    const groups = splitAgenda(
      [echeance("a", "2026-09-18"), echeance("b", "2026-09-24"), echeance("c", "2026-09-27")],
      today,
    );
    expect(groups.late.map((item) => item.id)).toEqual(["a"]);
    expect(groups.today).toEqual([]);
    expect(groups.week.map((item) => item.id)).toEqual(["b", "c"]);
  });

  it("compte les jours de retard en jours calendaires", () => {
    expect(daysBetween("2026-09-18", today)).toBe(5);
    // Passage à l'heure d'hiver : l'arrondi garde un jour entier.
    expect(daysBetween("2026-10-24", "2026-10-26")).toBe(2);
  });

  it("formate les dates en français", () => {
    expect(longDay(new Date(2026, 8, 12))).toBe("Samedi 12 septembre");
    expect(shortWeekday("2026-09-15")).toBe("mar. 15");
  });
});
