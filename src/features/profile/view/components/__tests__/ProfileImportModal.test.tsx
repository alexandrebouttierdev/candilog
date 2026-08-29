import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { open } from "@tauri-apps/plugin-dialog";
import { aiService } from "@/features/ai/services/aiService";
import type { ImportProfilePreview } from "@/shared/types/generated/profile";
import { ProfileImportModal } from "../ProfileImportModal";

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: vi.fn(),
}));

vi.mock("@/features/ai/services/aiService", () => ({
  aiService: { importProfile: vi.fn(), cancel: vi.fn() },
  generation_id: () => "gen-1",
}));

vi.mock("@/features/ai/viewmodel/useAiProgress", () => ({
  useCancelAiOnUnmount: () => undefined,
}));

vi.mock("../../../viewmodel/useProfileImportProgress", () => ({
  useProfileImportProgress: () => ({ step: null, entries: [] }),
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
  it("n'affiche pas Importer avant la revue", () => {
    render(
      <ProfileImportModal open busy={false} onClose={vi.fn()} onApply={vi.fn()} />,
    );

    expect(screen.getByRole("button", { name: "Annuler" })).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Importer les éléments sélectionnés" })).not.toBeInTheDocument();
    expect(screen.queryByText(/%/)).not.toBeInTheDocument();
  });

  it("applique l'import même si tout conserve l'existant", async () => {
    vi.mocked(open).mockResolvedValue("/tmp/cv.pdf");
    vi.mocked(aiService.importProfile).mockResolvedValue(conflictOnlyPreview());
    const onApply = vi.fn().mockResolvedValue({ added: 0, replaced: 0, skipped: 1 });

    render(<ProfileImportModal open busy={false} onClose={vi.fn()} onApply={onApply} />);

    await userEvent.click(screen.getByRole("button", { name: /Choisir un CV PDF/ }));
    await userEvent.click(screen.getByRole("button", { name: "Analyser le CV" }));
    await userEvent.click(
      screen.getByRole("button", { name: "Importer les éléments sélectionnés" }),
    );

    expect(onApply).toHaveBeenCalledOnce();
  });
});
