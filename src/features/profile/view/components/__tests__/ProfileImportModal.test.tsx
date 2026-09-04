import { act, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { aiService } from "@/features/ai/services/aiService";
import type { AiExecution } from "@/features/ai/model/types";
import type { ImportProfilePreview } from "@/shared/types/generated/profile";
import { AppError } from "@/shared/types/app-error";
import { ProfileImportModal } from "../ProfileImportModal";
import { useAiOperationStore } from "@/features/ai/viewmodel/ai-operation-store";

vi.mock("@/features/ai/services/aiService", () => ({
  aiService: { importProfile: vi.fn(), cancel: vi.fn() },
  generation_id: () => "gen-1",
}));

const progress = vi.hoisted(() => ({
  current: { step: null as string | null, entries: [] as { at: string; message: string }[], tokens_used: null as number | null },
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

function execution(
  output = conflictOnlyPreview(),
  elapsed_ms = 2_000,
  tokens_used: number | null = 640,
): AiExecution<ImportProfilePreview> {
  return { output, elapsed_ms, tokens_used };
}

describe("ProfileImportModal", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    useAiOperationStore.setState({ active: null });
    vi.mocked(aiService.cancel).mockResolvedValue(undefined);
    progress.current = { step: null, entries: [], tokens_used: null };
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
      tokens_used: null,
    };
    rerender(ui());

    expect(await screen.findByText("Analyse du CV en cours…")).toBeInTheDocument();
    expect(screen.getByText("Lecture du fichier…")).toBeInTheDocument();
  });

  it("arrête l'analyse, revient au sélecteur et ignore le résultat tardif", async () => {
    let resolveImport: ((value: AiExecution<ImportProfilePreview>) => void) | undefined;
    vi.mocked(aiService.importProfile).mockReturnValue(
      new Promise((resolve) => { resolveImport = resolve; }),
    );
    let resolveCancel: (() => void) | undefined;
    vi.mocked(aiService.cancel).mockReturnValue(
      new Promise((resolve) => { resolveCancel = resolve; }),
    );
    const ui = () => (
      <ProfileImportModal open busy={false} onClose={vi.fn()} onApply={vi.fn()} />
    );
    const { rerender } = render(ui());
    await userEvent.click(screen.getByRole("button", { name: /Choisir et analyser un CV PDF/ }));
    progress.current = {
      step: "Lecture du fichier…",
      entries: [{ at: "2026-08-30T10:00:00Z", message: "Lecture du fichier" }],
      tokens_used: null,
    };
    rerender(ui());

    await userEvent.click(screen.getByRole("button", { name: "Arrêter" }));
    expect(aiService.cancel).toHaveBeenCalledWith("gen-1");
    expect(screen.getByRole("button", { name: "Arrêt…" })).toBeDisabled();

    await act(async () => {
      resolveCancel?.();
      await Promise.resolve();
    });
    expect(await screen.findByRole("button", { name: /Choisir et analyser un CV PDF/ })).toBeInTheDocument();

    await act(async () => {
      resolveImport?.(execution());
      await Promise.resolve();
    });
    expect(screen.queryByRole("button", { name: "Importer les éléments sélectionnés" })).not.toBeInTheDocument();
  });

  it("conserve l'analyse arrêtable quand l'annulation échoue", async () => {
    vi.mocked(aiService.importProfile).mockReturnValue(new Promise(() => undefined));
    vi.mocked(aiService.cancel).mockRejectedValue(
      new AppError({ code: "IO_ERROR", message: "L'arrêt a échoué." }),
    );
    const ui = () => (
      <ProfileImportModal open busy={false} onClose={vi.fn()} onApply={vi.fn()} />
    );
    const { rerender } = render(ui());
    await userEvent.click(screen.getByRole("button", { name: /Choisir et analyser un CV PDF/ }));
    progress.current = {
      step: "Lecture du fichier…",
      entries: [{ at: "2026-08-30T10:00:00Z", message: "Lecture du fichier" }],
      tokens_used: null,
    };
    rerender(ui());

    await userEvent.click(screen.getByRole("button", { name: "Arrêter" }));

    expect(await screen.findByText("L'arrêt a échoué.")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Arrêter" })).toBeEnabled();
    expect(screen.queryByRole("button", { name: /Choisir et analyser un CV PDF/ }))
      .not.toBeInTheDocument();
  });

  it("utilise le même arrêt quand la modale est fermée", async () => {
    vi.mocked(aiService.importProfile).mockReturnValue(new Promise(() => undefined));
    vi.mocked(aiService.cancel).mockResolvedValue(undefined);
    const onClose = vi.fn();
    render(<ProfileImportModal open busy={false} onClose={onClose} onApply={vi.fn()} />);
    await userEvent.click(screen.getByRole("button", { name: /Choisir et analyser un CV PDF/ }));

    await userEvent.click(screen.getByRole("button", { name: "Annuler" }));

    expect(aiService.cancel).toHaveBeenCalledWith("gen-1");
    expect(onClose).toHaveBeenCalledOnce();
  });

  it("mesure la durée d'analyse sans le temps passé dans le sélecteur", async () => {
    const now = vi.spyOn(Date, "now");
    now.mockReturnValue(1_000);
    let publish: (preview: AiExecution<ImportProfilePreview>) => void = () => undefined;
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
      tokens_used: null,
    };
    rerender(ui());
    now.mockReturnValue(12_000);
    await act(async () => {
      publish(execution());
      await Promise.resolve();
    });

    expect(screen.getByText("Analysé en 2 s · 640 tokens")).toBeInTheDocument();
    now.mockRestore();
  });

  it("applique l'import même si tout conserve l'existant", async () => {
    vi.mocked(aiService.importProfile).mockResolvedValue(execution());
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
