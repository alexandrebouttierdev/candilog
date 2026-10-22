import { describe, expect, it } from "vitest";
import type { SyncStatus } from "@/shared/types/generated/sync";
import { formatMoment, needsAttention, pendingLabel, phaseTone } from "../syncStatus";

const base: SyncStatus = {
  provider: "folder",
  phase: "up_to_date",
  auto_sync: true,
  location: "/Users/camille/Dropbox",
  pending_changes: 0,
  conflicts: 0,
  waiting_changes: 0,
  missing_files: 0,
  computers: 2,
  last_success_at: null,
  last_attempt_at: null,
  next_attempt_at: null,
  message: null,
};

describe("état de synchronisation", () => {
  it("donne une forme de glyphe à chaque état, sans dépendre de la couleur", () => {
    expect(phaseTone("up_to_date")).toBe("g");
    expect(phaseTone("offline")).toBe("a");
    expect(phaseTone("auth_required")).toBe("c");
    expect(phaseTone("syncing")).toBe("n");
  });

  it("signale les états qui attendent une action de l'utilisateur", () => {
    expect(needsAttention({ ...base, phase: "key_required" })).toBe(true);
    expect(needsAttention({ ...base, phase: "conflict" })).toBe(true);
    expect(needsAttention({ ...base, phase: "offline" })).toBe(false);
  });

  it("compte les modifications en attente en français", () => {
    expect(pendingLabel(0)).toBe("Aucune");
    expect(pendingLabel(1)).toBe("1 modification");
    expect(pendingLabel(3)).toBe("3 modifications");
  });

  it("affiche « Jamais » sans synchronisation confirmée", () => {
    expect(formatMoment(null)).toBe("Jamais");
    expect(formatMoment("2026-10-10T12:30:00Z")).toMatch(/octobre à \d{2}:\d{2}/);
  });
});
