import { act, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { aiService } from "@/features/ai/services/aiService";
import type { ImportProfilePreview } from "@/shared/types/generated/profile";
import { ProfileImportModal } from "../ProfileImportModal";

vi.mock("@/features/ai/services/aiService", () => ({
  aiService: { importProfile: vi.fn(), cancel: vi.fn() },
  generation_id: () => "gen-1",
}));

vi.mock("@/features/ai/viewmodel/useAiProgress", () => ({
  useCancelAiOnUnmount: () => undefined,
}));

const progress = vi.hoisted(() => ({
  current: { step: null as string | null, entries: [] as { at: string; message: string }[] },
}));

vi.mock("../../../viewmodel/useProfileImportProgress", () => ({
  useProfileImportProgress: () => progress.current,
}));

function conflictOnlyPreview(): ImportProfilePreview {
  return {
    identity: [
      {
        id: "title",
        label: "Titre professionnel",
        proposed: "Lead",
        existing: "Dev",
        has_conflict: true,
      },
    ],
    experiences: [],
    skills: [],
    education: [],
    languages: [],
    projects: [],
    certifications: [],
    counts: {
      identity: 1,
      experiences: 0,
      skills: 0,
      education: 0,
      languages: 0,
      projects: 0,
      certifications: 0,
    },
  };
}

describe("ProfileImportModal", () => {
  beforeEach(() => {
    progress.current = { step: null, entries: [] };
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("n'affiche pas Importer avant la revue", () => {
    render(
      <ProfileImportModal open busy={false} onClose={vi.fn()} onApply={vi.fn()} />,
    );

    expect(screen.getByRole("button", { name: "Annuler" })).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Importer les éléments sélectionnés" })).not.toBeInTheDocument();
    expect(screen.queryByText(/%/)).not.toBeInTheDocument();
  });

  it("n'annonce pas l'analyse tant que le fichier n'est pas choisi", async () => {
    vi.mocked(aiService.importProfile).mockReturnValue(new Promise(() => undefined));

    render(<ProfileImportModal open busy={false} onClose={vi.fn()} onApply={vi.fn()} />);

    await userEvent.click(screen.getByRole("button", { name: /Choisir et analyser un CV PDF/ }));

    expect(screen.queryByText("Analyse du CV en cours…")).not.toBeInTheDocument();
    expect(screen.queryByText(/Temps écoulé/)).not.toBeInTheDocument();
    expect(screen.getByText(/fenêtre de sélection/i)).toBeInTheDocument();
  });

  it("bascule sur l'analyse à la première progression reçue", async () => {
    vi.mocked(aiService.importProfile).mockReturnValue(new Promise(() => undefined));
    const ui = () => (
      <ProfileImportModal open busy={false} onClose={vi.fn()} onApply={vi.fn()} />
    );
    const { rerender } = render(ui());

    await userEvent.click(screen.getByRole("button", { name: /Choisir et analyser un CV PDF/ }));
    progress.current = {
      step: "Lecture du fichier…",
      entries: [{ at: "2026-08-30T10:00:00Z", message: "Lecture du fichier" }],
    };
    rerender(ui());

    expect(await screen.findByText("Analyse du CV en cours…")).toBeInTheDocument();
    expect(screen.getByText("Lecture du fichier…")).toBeInTheDocument();
  });

  it("mesure la durée d'analyse sans le temps passé dans le sélecteur", async () => {
    const now = vi.spyOn(Date, "now");
    now.mockReturnValue(1_000);
    let publish: (preview: ImportProfilePreview) => void = () => undefined;
    vi.mocked(aiService.importProfile).mockReturnValue(
      new Promise((resolve) => {
        publish = resolve;
      }),
    );
    const ui = () => (
      <ProfileImportModal open busy={false} onClose={vi.fn()} onApply={vi.fn()} />
    );
    const { rerender } = render(ui());

    await userEvent.click(screen.getByRole("button", { name: /Choisir et analyser un CV PDF/ }));
    // Neuf secondes passées dans le sélecteur natif avant le choix du fichier.
    now.mockReturnValue(10_000);
    progress.current = {
      step: "Lecture du fichier…",
      entries: [{ at: new Date(10_000).toISOString(), message: "Lecture du fichier" }],
    };
    rerender(ui());
    now.mockReturnValue(12_000);
    await act(async () => {
      publish(conflictOnlyPreview());
      await Promise.resolve();
    });

    expect(screen.getByText("Analyse terminée en 2 s")).toBeInTheDocument();
    now.mockRestore();
  });

  it("applique l'import même si tout conserve l'existant", async () => {
    vi.mocked(aiService.importProfile).mockResolvedValue(conflictOnlyPreview());
    const onApply = vi.fn().mockResolvedValue({ added: 0, replaced: 0, skipped: 1 });

    render(<ProfileImportModal open busy={false} onClose={vi.fn()} onApply={onApply} />);

    await userEvent.click(screen.getByRole("button", { name: /Choisir et analyser un CV PDF/ }));
    await userEvent.click(
      screen.getByRole("button", { name: "Importer les éléments sélectionnés" }),
    );

    expect(onApply).toHaveBeenCalledOnce();
    expect(aiService.importProfile).toHaveBeenCalledWith({ generation_id: "gen-1" });
  });
});
