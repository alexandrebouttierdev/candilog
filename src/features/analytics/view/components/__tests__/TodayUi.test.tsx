import { describe, expect, it } from "vitest";
import type { Dashboard, UpcomingItem } from "@/shared/types/generated/analytics";
import { formatEventTime, formatWhenShort, isTodayEmpty, splitUpcoming } from "../TodayUi";

function item(kind: string, id: string): UpcomingItem {
  return {
    id,
    kind,
    date: kind === "entretien" ? "2026-08-29T14:30:00+02:00" : "2026-08-29",
    job_title: "Poste",
    company_name: kind === "entretien" ? "Nova Digital" : "Atlas Studio",
    detail: kind === "entretien" ? "Visio" : "Email",
  };
}

function emptyDashboard(overrides: Partial<Dashboard> = {}): Dashboard {
  return {
    metrics: {
      applications: 0,
      interviews: 0,
      responses: 0,
      rejected: 0,
      pending: 0,
      followed_up: 0,
      response_rate: 0,
      interview_rate: 0,
    },
    performance: {
      average_response_days: null,
      applications_per_week: 0,
      upcoming_interviews: 0,
      overdue_follow_ups: 0,
    },
    upcoming_items: [],
    pipeline: [],
    activity: [{ start: "2026-08-24", count: 0 }],
    recent: [],
    ...overrides,
  };
}

describe("formatWhenShort", () => {
  it("extrait l'heure au format HH:MM", () => {
    expect(formatEventTime("2026-08-29T14:30:00+02:00")).toBe("14:30");
    expect(formatWhenShort("2026-08-29T14:30:00+02:00")).toBe("14:30");
  });

  it("abrège aujourd'hui, demain et les autres jours", () => {
    const now = new Date(2026, 7, 29);
    expect(formatWhenShort("2026-08-29", now)).toBe("Auj.");
    expect(formatWhenShort("2026-08-30", now)).toBe("Dem.");
    expect(formatWhenShort("2026-09-02", now)).toBe("02/09");
  });
});

describe("isTodayEmpty", () => {
  it("détecte un bureau sans donnée", () => {
    expect(isTodayEmpty(emptyDashboard())).toBe(true);
  });

  it("reste occupé dès qu'une relance ou une candidature existe", () => {
    expect(isTodayEmpty(emptyDashboard({ upcoming_items: [item("relance", "r1")] }))).toBe(false);
  });
});

describe("splitUpcoming", () => {
  it("met l'entretien en briefing même s'il n'est pas le premier", () => {
    const relance = item("relance", "r1");
    const entretien = item("entretien", "e1");
    const { next, rest } = splitUpcoming([relance, entretien]);
    expect(next).toBe(entretien);
    expect(rest).toEqual([relance]);
  });

  it("prend la première échéance s'il n'y a pas d'entretien", () => {
    const relance = item("relance", "r1");
    const { next, rest } = splitUpcoming([relance]);
    expect(next).toBe(relance);
    expect(rest).toEqual([]);
  });

  it("reste vide sans échéance", () => {
    expect(splitUpcoming([])).toEqual({ next: null, rest: [] });
  });
});
