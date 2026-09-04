import { beforeEach, describe, expect, it, vi } from "vitest";
import { ipc } from "@/shared/services/ipc";
import { playCompletionSound } from "@/shared/lib/completion-sound";
import { aiService } from "../aiService";

vi.mock("@/shared/services/ipc", () => ({ ipc: vi.fn() }));
vi.mock("@/shared/lib/completion-sound", () => ({ playCompletionSound: vi.fn() }));

describe("aiService", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("annonce la fin d'une génération par un son", async () => {
    vi.mocked(ipc).mockResolvedValue("Madame, Monsieur,");

    await aiService.generateCoverLetter({
      generation_id: "gen-1",
      company: null,
      job_title: null,
      tone: "formal",
      length: "medium",
      context: "Une offre",
      previous_cover_letter: null,
      instruction: null,
    });

    expect(playCompletionSound).toHaveBeenCalledOnce();
  });

  it("reste muet quand le sélecteur de fichier a été annulé", async () => {
    vi.mocked(ipc).mockResolvedValue(null);

    await aiService.importProfile({ generation_id: "gen-2" });

    expect(playCompletionSound).not.toHaveBeenCalled();
  });

  it("sélectionne un PDF sans lancer ni annoncer un traitement IA", async () => {
    vi.mocked(ipc).mockResolvedValue({ path: "/tmp/cv.pdf", name: "cv.pdf" });

    await expect(aiService.selectResumeFile()).resolves.toEqual({
      path: "/tmp/cv.pdf",
      name: "cv.pdf",
    });

    expect(ipc).toHaveBeenCalledWith("ai_select_resume_file");
    expect(playCompletionSound).not.toHaveBeenCalled();
  });

  it("reste muet quand la génération échoue", async () => {
    vi.mocked(ipc).mockRejectedValue(new Error("indisponible"));

    await expect(aiService.analyzeListing("offre")).rejects.toThrow();
    expect(playCompletionSound).not.toHaveBeenCalled();
  });
});
