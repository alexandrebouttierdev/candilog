import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";
import type { Dashboard, UpcomingItem } from "@/shared/types/generated/analytics";
import {
  buildTodos,
  formatEventTime,
  formatWhenShort,
  isTodayEmpty,
  splitUpcoming,
  TodoRows,
} from "../TodayUi";

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

describe("buildTodos", () => {
  const now = new Date(2026, 7, 29);

  it("place les relances en retard en tête, puis les actions du jour", () => {
    const todos = buildTodos(2, [item("relance", "r1"), item("entretien", "e1")], now);
    expect(todos.map((todo) => todo.key)).toEqual(["overdue", "prep-e1", "relance-r1"]);
    expect(todos[0]?.kind).toBe("En retard");
    expect(todos[1]?.kind).toBe("Préparer");
    expect(todos[2]?.kind).toBe("Email");
  });

  it("ignore les échéances qui ne sont pas aujourd'hui", () => {
    const demain: UpcomingItem = { ...item("entretien", "e2"), date: "2026-08-30T09:00:00+02:00" };
    expect(buildTodos(0, [demain], now)).toEqual([]);
  });
});

describe("TodoRows", () => {
  const now = new Date(2026, 7, 29);

  it("affiche une file d'actions avec le compte et sans case à cocher", () => {
    render(
      <TodoRows
        overdue={1}
        items={[item("entretien", "e1")]}
        now={now}
        onOpenApplications={() => {}}
        onOpenCalendar={() => {}}
      />,
    );
    expect(screen.getByRole("region", { name: "À faire" })).toBeInTheDocument();
    expect(screen.getByLabelText("2 à faire")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Relancer les candidatures en retard/ })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Nova Digital/ })).toBeInTheDocument();
    expect(screen.queryByRole("checkbox")).not.toBeInTheDocument();
  });

  it("reste visible quand il n'y a rien à traiter", () => {
    render(
      <TodoRows
        overdue={0}
        items={[]}
        now={now}
        onOpenApplications={() => {}}
        onOpenCalendar={() => {}}
      />,
    );
    expect(screen.getByText("Rien à traiter aujourd'hui.")).toBeInTheDocument();
  });
});
