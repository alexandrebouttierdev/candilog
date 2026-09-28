import type { HistoryEntry } from "@/shared/types/generated/relations";
import { historyLines, localToday } from "../history";

function entry(partial: Partial<HistoryEntry> & Pick<HistoryEntry, "kind" | "at">): HistoryEntry {
  return { application_id: null, job_title: null, detail: null, note_id: null, ...partial };
}

describe("historyLines", () => {
  const now = new Date(2026, 8, 10, 12, 0);

  it("met chaque fait en mots, candidature concernée à part", () => {
    const lines = historyLines(
      [
        entry({ kind: "interview", at: new Date(2026, 8, 12, 14, 30).toISOString(), detail: "Visio", job_title: "Technicien N2" }),
        entry({ kind: "note", at: "2026-09-02", detail: "Réponse positive de Claire", note_id: "n1" }),
        entry({ kind: "status_changed", at: new Date(2026, 8, 1, 9, 0).toISOString(), detail: "REFUS", job_title: "Technicien N2" }),
        entry({ kind: "follow_up_done", at: new Date(2026, 7, 30, 9, 0).toISOString(), detail: "Email", job_title: "Technicien N2" }),
        entry({ kind: "application_sent", at: "2026-08-29", job_title: "Technicien N2" }),
        entry({ kind: "added", at: new Date(2026, 7, 1, 9, 0).toISOString() }),
      ],
      "company",
      now,
    );

    expect(lines.map((line) => [line.day, line.text, line.job])).toEqual([
      ["12-09", "Entretien prévu à 14:30 · visioconférence", "Technicien N2"],
      ["02-09", "Réponse positive de Claire", null],
      ["01-09", "Candidature refusée", "Technicien N2"],
      ["30-08", "Relance faite par e-mail", "Technicien N2"],
      ["29-08", "Candidature envoyée", "Technicien N2"],
      ["01-08", "Entreprise ajoutée au suivi", null],
    ]);
    expect(lines[1]?.noteId).toBe("n1");
    expect(lines[0]?.noteId).toBeNull();
  });

  it("dit un entretien passé sans « prévu » et un contact « ajouté »", () => {
    const [interview, added] = historyLines(
      [
        entry({ kind: "interview", at: new Date(2026, 8, 3, 10, 0).toISOString(), detail: "Autre" }),
        entry({ kind: "added", at: "2026-08-01" }),
      ],
      "contact",
      now,
    );
    expect(interview?.text).toBe("Entretien à 10:00");
    expect(added?.text).toBe("Contact ajouté");
    expect(added?.fullDate).toBe("01-08-2026");
  });

  it("donne la date du jour dans le fuseau de la machine", () => {
    expect(localToday(new Date(2026, 0, 5, 23, 59))).toBe("2026-01-05");
  });
});
