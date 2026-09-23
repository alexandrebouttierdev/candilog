import { describe, expect, it } from "vitest";
import type { Application } from "@/shared/types/generated/applications";
import { dueOf, formatReference } from "../presentation";

function iso(offsetDays: number): string {
  const date = new Date();
  date.setDate(date.getDate() + offsetDays);
  const pad = (value: number) => String(value).padStart(2, "0");
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}`;
}

function candidature(patch: Partial<Application>): Application {
  return {
    status: "EN_ATTENTE",
    sent_date: iso(-2),
    next_follow_up_date: null,
    next_interview_at: null,
    ...patch,
  } as Application;
}

describe("référence lisible", () => {
  it("complète le numéro sur trois chiffres", () => {
    expect(formatReference(7)).toBe("CAN-007");
    expect(formatReference(142)).toBe("CAN-142");
    expect(formatReference(1234)).toBe("CAN-1234");
  });
});

describe("échéance d'une ligne", () => {
  it("met l'entretien à venir avant la relance", () => {
    const due = dueOf(
      candidature({ next_interview_at: `${iso(0)}T14:30:00`, next_follow_up_date: iso(3) }),
    );
    expect(due).toMatchObject({ kind: "interview", label: "◷ auj. 14:30", tone: "success" });
  });

  it("affiche la prochaine relance quand aucun entretien n'est prévu", () => {
    const date = iso(5);
    expect(dueOf(candidature({ next_follow_up_date: date }))?.label).toBe(
      `↻ ${date.slice(8, 10)}-${date.slice(5, 7)}`,
    );
  });

  it("signale une candidature sans réponse depuis plus de 14 jours", () => {
    expect(dueOf(candidature({ sent_date: iso(-56) }))).toMatchObject({
      kind: "silence",
      label: "56 j",
      tone: "danger",
    });
    // Un entretien ou un refus est une réponse : aucun silence à signaler.
    expect(dueOf(candidature({ sent_date: iso(-56), status: "REFUS" }))).toBeNull();
    expect(dueOf(candidature({ sent_date: iso(-10) }))).toBeNull();
  });
});
