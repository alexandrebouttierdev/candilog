import type { ReactNode } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { useResumeEditor } from "../useResumeEditor";
import { workspaceFixture } from "../../model/resumeWorkspace";
import { documentsService } from "../../services/documentsService";
import { profileService } from "@/features/profile/services/profileService";
import type { ResumeProposal, ResumeWorkspace } from "@/shared/types/generated/documents";
import { AppError } from "@/shared/types/app-error";

function wrapper({ children }: { children: ReactNode }) {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false }, mutations: { retry: false } } });
  return <QueryClientProvider client={client}>{children}</QueryClientProvider>;
}

function missingSkillProposal(overrides: Partial<ResumeProposal> = {}): ResumeProposal {
  return {
    id: "skill-docker",
    kind: "missing_skill",
    target: { type: "skill_group", group_id: "group-1" },
    label: "Ajouter la compétence « Docker »",
    original_text: null,
    proposed_text: "Docker",
    gain: 5,
    status: "pending",
    applicable: true,
    ...overrides,
  };
}

function textReplacementProposal(overrides: Partial<ResumeProposal> = {}): ResumeProposal {
  return {
    id: "ats-0",
    kind: "text_replacement",
    target: { type: "profile" },
    label: "Reformuler le profil",
    original_text: "Profil synthétique.",
    proposed_text: "Profil avec React.",
    gain: 3,
    status: "pending",
    applicable: true,
    ...overrides,
  };
}

beforeEach(() => {
  vi.restoreAllMocks();
});

afterEach(() => {
  vi.useRealTimers();
});

describe("édition locale immédiate", () => {
  it("modifie le document localement sans attendre l'IPC", () => {
    const recalculate = vi.spyOn(documentsService, "recalculateResume");
    const { result } = renderHook(() => useResumeEditor(workspaceFixture()), { wrapper });

    act(() => {
      result.current.updateField({ type: "profile" }, "Nouveau profil");
    });

    expect(result.current.workspace.document.profile).toBe("Nouveau profil");
    expect(recalculate).not.toHaveBeenCalled();
  });
});

describe("recalcul différé", () => {
  it("ne recalcule qu'une seule fois après plusieurs frappes", async () => {
    vi.useFakeTimers();
    const recalculate = vi
      .spyOn(documentsService, "recalculateResume")
      .mockImplementation((workspace) => Promise.resolve(workspace));
    const { result } = renderHook(() => useResumeEditor(workspaceFixture()), { wrapper });

    act(() => { result.current.updateField({ type: "profile" }, "P"); });
    act(() => { result.current.updateField({ type: "profile" }, "Pr"); });
    act(() => { result.current.updateField({ type: "profile" }, "Pro"); });

    await act(async () => {
      await vi.advanceTimersByTimeAsync(300);
    });

    expect(recalculate).toHaveBeenCalledTimes(1);
    expect(result.current.workspace.document.profile).toBe("Pro");
  });

  it("ignore une réponse de recalcul plus ancienne que la dernière modification", async () => {
    vi.useFakeTimers();
    let resolveFirst: ((workspace: ResumeWorkspace) => void) | undefined;
    const recalculate = vi
      .spyOn(documentsService, "recalculateResume")
      .mockImplementationOnce(
        (workspace) =>
          new Promise((resolve) => {
            resolveFirst = () => resolve({ ...workspace, initial_score: 1 });
          }),
      )
      .mockImplementation((workspace) => Promise.resolve(workspace));
    const { result } = renderHook(() => useResumeEditor(workspaceFixture()), { wrapper });

    act(() => { result.current.updateField({ type: "profile" }, "Premier"); });
    await act(async () => { await vi.advanceTimersByTimeAsync(300); });
    act(() => { result.current.updateField({ type: "profile" }, "Second"); });
    await act(async () => { await vi.advanceTimersByTimeAsync(300); });

    act(() => { resolveFirst?.(workspaceFixture()); });
    await act(async () => { await Promise.resolve(); });

    expect(recalculate).toHaveBeenCalledTimes(2);
    expect(result.current.workspace.document.profile).toBe("Second");
    expect(result.current.workspace.initial_score).not.toBe(1);
  });
});

describe("décisions ATS via IPC", () => {
  it("accepte une proposition via l'IPC et met à jour le workspace", async () => {
    const base = workspaceFixture();
    base.proposals = [missingSkillProposal()];
    const updated: ResumeWorkspace = {
      ...base,
      proposals: [missingSkillProposal({ status: "accepted" })],
    };
    const apply = vi.spyOn(documentsService, "applyResumeProposal").mockResolvedValue(updated);
    const { result } = renderHook(() => useResumeEditor(base), { wrapper });

    await act(async () => {
      await result.current.applyProposal("skill-docker");
    });

    expect(apply).toHaveBeenCalledWith(base, "skill-docker");
    expect(result.current.workspace).toBe(updated);
  });

  it("refuse une proposition via l'IPC et met à jour le workspace", async () => {
    const base = workspaceFixture();
    base.proposals = [textReplacementProposal()];
    const updated: ResumeWorkspace = {
      ...base,
      proposals: [textReplacementProposal({ status: "rejected" })],
    };
    const reject = vi.spyOn(documentsService, "rejectResumeProposal").mockResolvedValue(updated);
    const { result } = renderHook(() => useResumeEditor(base), { wrapper });

    await act(async () => {
      await result.current.rejectProposal("ats-0");
    });

    expect(reject).toHaveBeenCalledWith(base, "ats-0");
    expect(result.current.workspace).toBe(updated);
  });

  it("signale en français l'échec d'une acceptation", async () => {
    const base = workspaceFixture();
    base.proposals = [missingSkillProposal()];
    vi.spyOn(documentsService, "applyResumeProposal").mockRejectedValue(
      new AppError({ code: "VALIDATION_ERROR", message: "Cette proposition ne correspond plus au CV actuel." }),
    );
    const { result } = renderHook(() => useResumeEditor(base), { wrapper });

    await act(async () => {
      await result.current.applyProposal("skill-docker");
    });

    expect(result.current.error).toBe("Cette proposition ne correspond plus au CV actuel.");
    expect(result.current.workspace).toBe(base);
  });
});

describe("annulation d'une décision ATS ciblée", () => {
  it("restaure une compétence acceptée en préservant une autre décision plus récente", async () => {
    const base = workspaceFixture();
    base.document.skill_groups = [{ id: "group-1", name: "Compétences", items: ["Rust", "Docker"] }];
    base.document.profile = "Profil avec React.";
    base.proposals = [
      missingSkillProposal({ status: "accepted" }),
      textReplacementProposal({ status: "accepted" }),
    ];
    const recalculate = vi
      .spyOn(documentsService, "recalculateResume")
      .mockImplementation((workspace) => Promise.resolve(workspace));
    const { result } = renderHook(() => useResumeEditor(base), { wrapper });

    await act(async () => {
      await result.current.undoProposal("skill-docker");
    });

    expect(recalculate).toHaveBeenCalledTimes(1);
    const sent = recalculate.mock.calls[0]![0];
    // La compétence est retirée du groupe ciblé...
    expect(sent.document.skill_groups[0]!.items).toEqual(["Rust"]);
    // ...et sa proposition seule repasse en attente...
    expect(sent.proposals.find((p) => p.id === "skill-docker")!.status).toBe("pending");
    // ...tandis que la reformulation acceptée plus tard n'est pas défaite : ni son statut...
    expect(sent.proposals.find((p) => p.id === "ats-0")!.status).toBe("accepted");
    // ...ni le texte qu'elle avait déjà remplacé.
    expect(sent.document.profile).toBe("Profil avec React.");
    expect(result.current.workspace.proposals.find((p) => p.id === "ats-0")!.status).toBe("accepted");
  });

  it("restaure le texte d'origine d'une reformulation acceptée", async () => {
    const base = workspaceFixture();
    base.document.profile = "Profil avec React.";
    base.proposals = [textReplacementProposal({ status: "accepted" })];
    const recalculate = vi
      .spyOn(documentsService, "recalculateResume")
      .mockImplementation((workspace) => Promise.resolve(workspace));
    const { result } = renderHook(() => useResumeEditor(base), { wrapper });

    await act(async () => {
      await result.current.undoProposal("ats-0");
    });

    const sent = recalculate.mock.calls[0]![0];
    expect(sent.document.profile).toBe("Profil synthétique.");
    expect(sent.proposals[0]!.status).toBe("pending");
  });

  it("remet une proposition refusée en attente sans toucher au document", async () => {
    const base = workspaceFixture();
    base.proposals = [textReplacementProposal({ status: "rejected" })];
    const recalculate = vi
      .spyOn(documentsService, "recalculateResume")
      .mockImplementation((workspace) => Promise.resolve(workspace));
    const { result } = renderHook(() => useResumeEditor(base), { wrapper });

    await act(async () => {
      await result.current.undoProposal("ats-0");
    });

    const sent = recalculate.mock.calls[0]![0];
    expect(sent.document.profile).toBe(base.document.profile);
    expect(sent.proposals[0]!.status).toBe("pending");
  });

  it("ne fait rien pour une proposition déjà en attente", async () => {
    const base = workspaceFixture();
    base.proposals = [missingSkillProposal({ status: "pending" })];
    const recalculate = vi.spyOn(documentsService, "recalculateResume");
    const { result } = renderHook(() => useResumeEditor(base), { wrapper });

    await act(async () => {
      await result.current.undoProposal("skill-docker");
    });

    expect(recalculate).not.toHaveBeenCalled();
    expect(result.current.workspace).toBe(base);
  });
});

describe("annulation et rétablissement locaux", () => {
  it("annule une édition locale sans appeler l'IPC", () => {
    const recalculate = vi.spyOn(documentsService, "recalculateResume");
    const initial = workspaceFixture();
    const { result } = renderHook(() => useResumeEditor(initial), { wrapper });

    act(() => { result.current.updateField({ type: "profile" }, "Modifié"); });
    expect(result.current.canUndo).toBe(true);

    act(() => { result.current.undo(); });

    expect(result.current.workspace).toBe(initial);
    expect(result.current.canUndo).toBe(false);
    expect(result.current.canRedo).toBe(true);
    expect(recalculate).not.toHaveBeenCalled();
  });

  it("rétablit une édition annulée", () => {
    const initial = workspaceFixture();
    const { result } = renderHook(() => useResumeEditor(initial), { wrapper });

    act(() => { result.current.updateField({ type: "profile" }, "Modifié"); });
    act(() => { result.current.undo(); });
    act(() => { result.current.redo(); });

    expect(result.current.workspace.document.profile).toBe("Modifié");
    expect(result.current.canRedo).toBe(false);
  });

  it("couvre aussi l'acceptation d'une proposition ATS", async () => {
    const base = workspaceFixture();
    base.proposals = [missingSkillProposal()];
    const updated: ResumeWorkspace = { ...base, proposals: [missingSkillProposal({ status: "accepted" })] };
    vi.spyOn(documentsService, "applyResumeProposal").mockResolvedValue(updated);
    const { result } = renderHook(() => useResumeEditor(base), { wrapper });

    await act(async () => {
      await result.current.applyProposal("skill-docker");
    });
    expect(result.current.canUndo).toBe(true);

    act(() => { result.current.undo(); });
    expect(result.current.workspace).toBe(base);
  });
});

describe("confirmation d'ajout au profil", () => {
  it("ouvre la demande d'ajout au profil uniquement pour une compétence manquante acceptée", async () => {
    const base = workspaceFixture();
    base.proposals = [missingSkillProposal()];
    const updated: ResumeWorkspace = { ...base, proposals: [missingSkillProposal({ status: "accepted" })] };
    vi.spyOn(documentsService, "applyResumeProposal").mockResolvedValue(updated);
    const { result } = renderHook(() => useResumeEditor(base), { wrapper });

    await act(async () => {
      await result.current.applyProposal("skill-docker");
    });

    expect(result.current.pendingProfileSkill).toEqual({
      proposal_id: "skill-docker",
      skill: "Docker",
      label: "Ajouter la compétence « Docker »",
    });
  });

  it("ne demande rien pour une suggestion textuelle acceptée", async () => {
    const base = workspaceFixture();
    base.proposals = [textReplacementProposal()];
    const updated: ResumeWorkspace = { ...base, proposals: [textReplacementProposal({ status: "accepted" })] };
    vi.spyOn(documentsService, "applyResumeProposal").mockResolvedValue(updated);
    const { result } = renderHook(() => useResumeEditor(base), { wrapper });

    await act(async () => {
      await result.current.applyProposal("ats-0");
    });

    expect(result.current.pendingProfileSkill).toBeNull();
  });

  it("garde la compétence au CV seul sans appeler le profil", async () => {
    const base = workspaceFixture();
    base.proposals = [missingSkillProposal()];
    const updated: ResumeWorkspace = { ...base, proposals: [missingSkillProposal({ status: "accepted" })] };
    vi.spyOn(documentsService, "applyResumeProposal").mockResolvedValue(updated);
    const addSkill = vi.spyOn(profileService, "addSkill");
    const { result } = renderHook(() => useResumeEditor(base), { wrapper });

    await act(async () => {
      await result.current.applyProposal("skill-docker");
    });
    act(() => { result.current.keepSkillInResumeOnly(); });

    expect(result.current.pendingProfileSkill).toBeNull();
    expect(addSkill).not.toHaveBeenCalled();
  });

  it("ajoute la compétence au profil puis referme la demande", async () => {
    const base = workspaceFixture();
    base.proposals = [missingSkillProposal()];
    const updated: ResumeWorkspace = { ...base, proposals: [missingSkillProposal({ status: "accepted" })] };
    vi.spyOn(documentsService, "applyResumeProposal").mockResolvedValue(updated);
    const addSkill = vi.spyOn(profileService, "addSkill").mockResolvedValue({
      profile: {
        identity: {
          first_name: "Alex",
          name: "Exemple",
          email: "alex@exemple.fr",
          phone: null,
          address: null,
          city: null,
          title: null,
          resume: null,
          linkedin: null,
          github: null,
          website: null,
        },
        experiences: [],
        projects: [],
        education: [],
        certifications: [],
        languages: [],
        skills: [{ name: "Docker" }],
      },
      completion: 40,
      incomplete_sections: [],
      updated_at: null,
    });
    const { result } = renderHook(() => useResumeEditor(base), { wrapper });

    await act(async () => {
      await result.current.applyProposal("skill-docker");
    });
    await act(async () => {
      await result.current.addPendingSkillToProfile();
    });

    expect(addSkill).toHaveBeenCalledWith("Docker");
    expect(result.current.pendingProfileSkill).toBeNull();
    expect(result.current.error).toBeNull();
  });
});
