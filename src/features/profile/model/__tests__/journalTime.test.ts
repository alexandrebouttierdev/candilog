import { describe, expect, it } from "vitest";
import { formatJournalTime } from "../journalTime";

describe("formatJournalTime", () => {
  it("formate un ISO en heure locale", () => {
    expect(formatJournalTime("2026-08-29T14:32:01.000Z")).toMatch(/^\d{2}:\d{2}:\d{2}$/);
  });
});
