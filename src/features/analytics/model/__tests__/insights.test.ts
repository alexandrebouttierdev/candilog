import { describe, expect, it } from "vitest";
import type { Analytics } from "@/shared/types/generated/analytics";
import { insightsOf } from "../insights";

function data(overrides: Partial<Analytics> = {}): Analytics {
  return {
    metrics: {
      applications: 10,
      interviews: 2,
      responses: 4,
      rejected: 2,
      pending: 5,
      followed_up: 1,
      response_rate: 40,
      interview_rate: 20,
    },
    performance: { average_response_days: 9, applications_per_week: 2, upcoming_interviews: 0, overdue_follow_ups: 0 },
    activity: [],
    funnel: [],
    to_follow_up: [],
    channels: [],
    ...overrides,
  };
}

describe("constats des analyses", () => {
  it("compare les canaux dès que deux ont assez d'envois", () => {
    const constats = insightsOf(
      data({
        channels: [
          { channel: "OFFER", applications: 6, responses: 1 },
          { channel: "NETWORK", applications: 3, responses: 2 },
          { channel: "SPONTANEOUS", applications: 1, responses: 1 },
        ],
      }),
    );
    expect(constats[0]?.text).toBe(
      "« Réseau » répond le mieux : 67 % de réponses (2/3), contre 17 % pour « Offre publiée ».",
    );
  });

  it("ne compare pas un canal isolé", () => {
    const constats = insightsOf(data({ channels: [{ channel: "OFFER", applications: 10, responses: 4 }] }));
    expect(constats.map((constat) => constat.text)).toEqual(["2 réponses sur 4 ont mené à un entretien."]);
  });

  it("signale les candidatures à relancer", () => {
    const constats = insightsOf(
      data({
        to_follow_up: [
          { id: "a", job_title: "Dev", company_name: "Nova", sent_date: "2026-09-01", days: 20, reference_number: 1 },
        ],
      }),
    );
    expect(constats.some((constat) => constat.text.startsWith("1 candidature attend une réponse"))).toBe(true);
  });

  it("ne dit rien d'une période vide", () => {
    expect(insightsOf(data({ metrics: { ...data().metrics, applications: 0 } }))).toEqual([]);
  });
});
