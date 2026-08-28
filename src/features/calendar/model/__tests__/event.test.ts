import { describe, expect, it } from "vitest";
import { fromInterview, fromFollowUp, groupByDay } from "../event";
import type { Interview } from "@/features/interviews/services/interviewService";
import type { FollowUp } from "@/features/followups/services/followUpService";

function interview(id: string, timestamp: string): Interview {
  return {
    id,
    application_id: "c1",
    application_job_title: "Développeur Frontend",
    company_name: "Nova Digital",
    contact_id: null,
    contact_name: null,
    interview_date: timestamp,
    type: "Visio",
    location: null,
    notes: null,
    minutes: null,
    analysis_ai: null,
    created_at: "2026-08-20T00:00:00Z",
    updated_at: "2026-08-20T00:00:00Z",
  };
}

function follow_up(id: string, date: string): FollowUp {
  return {
    id,
    application_id: "c2",
    application_job_title: "Product Designer",
    company_name: "Atlas Studio",
    follow_up_date: date,
    type: "Email",
    notes: null,
    created_at: "2026-08-20T00:00:00Z",
  };
}

describe("conversion en événement", () => {
  it("donne à l'entretien la tonalité de l'avancement et son heure", () => {
    const event = fromInterview(interview("e1", "2026-08-25T14:00:00+02:00"));

    expect(event).toMatchObject({
      kind: "interview",
      day: "2026-08-25",
      time: "14:00",
      tone: "success",
      label: "Développeur Frontend",
      detail: "Nova Digital",
    });
  });

  it("donne à la relance la tonalité de ce qui est à traiter, sans heure", () => {
    // Une relance se programme au jour : afficher « 00:00 » suggérerait un créneau.
    const event = fromFollowUp(follow_up("r1", "2026-08-27"));

    expect(event).toMatchObject({ kind: "follow_up", day: "2026-08-27", tone: "warning" });
    expect(event.time).toBeNull();
  });

  it("retombe sur un libellé générique quand la candidature n'est pas résolue", () => {
    const orphelin = { ...interview("e1", "2026-08-25T14:00:00+02:00"), application_job_title: null };
    expect(fromInterview(orphelin).label).toBe("Interview");
  });
});

describe("regroupement par jour", () => {
  it("range chaque événement dans sa journée", () => {
    const parDay = groupByDay([
      fromInterview(interview("e1", "2026-08-25T14:00:00+02:00")),
      fromFollowUp(follow_up("r1", "2026-08-27")),
      fromFollowUp(follow_up("r2", "2026-08-25")),
    ]);

    expect([...parDay.keys()].sort()).toEqual(["2026-08-25", "2026-08-27"]);
    expect(parDay.get("2026-08-25")).toHaveLength(2);
  });

  it("place les relances avant les entretiens d'une même journée", () => {
    // Une relance se traite quand on veut ; un entretien a un créneau, qui vient après.
    const parDay = groupByDay([
      fromInterview(interview("e1", "2026-08-25T09:00:00+02:00")),
      fromFollowUp(follow_up("r1", "2026-08-25")),
    ]);

    expect(parDay.get("2026-08-25")?.map((e) => e.kind)).toEqual(["follow_up", "interview"]);
  });

  it("trie les entretiens d'une journée par heure croissante", () => {
    const parDay = groupByDay([
      fromInterview(interview("e2", "2026-08-25T16:00:00+02:00")),
      fromInterview(interview("e1", "2026-08-25T09:00:00+02:00")),
    ]);

    expect(parDay.get("2026-08-25")?.map((e) => e.time)).toEqual(["09:00", "16:00"]);
  });

  it("ne crée aucune journée pour une liste vide", () => {
    expect(groupByDay([]).size).toBe(0);
  });
});
