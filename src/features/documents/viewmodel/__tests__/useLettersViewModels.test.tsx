import type { ReactNode } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { aiService, useAiOperationStore } from "@/features/ai";
import { profileService } from "@/features/profile";
import { documentsService } from "../../services/documentsService";
import { useUiStore } from "@/shared/lib/ui-store";
import { AppError } from "@/shared/types/app-error";
import { useLetterWriterViewModel } from "../useLetterWriterViewModel";
import { useLettersLibraryViewModel } from "../useLettersLibraryViewModel";

vi.mock("@/features/ai/viewmodel/useAiProgress", () => ({
  useAiProgress: () => null,
}));

function wrapper({ children }: { children: ReactNode }) {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
  return <QueryClientProvider client={client}>{children}</QueryClientProvider>;
}

function profilePayload() {
  return {
    profile: {
      photo: null,
      identity: {
        first_name: "Alex", name: "Exemple", email: "alex@exemple.fr",
        phone: null, address: null, city: "Rennes", title: null,
        resume: null, birth_date: null, age: null, availability: null, desired_contracts: null, linkedin: null, github: null, website: null,
      },
      experiences: [], skills: [], education: [], languages: [], projects: [], certifications: [],
    interests: [],
    },
    completion: 40, incomplete_sections: [], updated_at: "2026-08-31T00:00:00Z",
  };
}

beforeEach(() => {
  vi.restoreAllMocks();
  useAiOperationStore.setState({ active: null });
  useUiStore.setState({ toasts: [] });
  vi.spyOn(profileService, "load").mockResolvedValue(profilePayload());
});

describe("ViewModel de la bibliothèque de lettres", () => {
  it("liste les lettres et expose l'identité du profil", async () => {
    vi.spyOn(documentsService, "listCoverLettersPage").mockResolvedValue({
      items: [{
        id: "l-1", name: "Lettre Astek", company: "Astek", job_title: "Dev",
        recipient: null, recipient_address: null, job_reference: null,
        tone: "formal", length: "medium", content: "Madame, Monsieur,",
        created_at: "2026-08-30T00:00:00Z",
      }],
      total: 1, page: 1, page_size: 8, total_pages: 1,
    });
    const { result } = renderHook(() => useLettersLibraryViewModel(), { wrapper });
    await waitFor(() => expect(result.current.coverLetters).toHaveLength(1));
    await waitFor(() => expect(result.current.identity?.first_name).toBe("Alex"));
  });
});

describe("ViewModel du rédacteur de lettre", () => {
  it("signale un refus d'enregistrement", async () => {
    vi.spyOn(aiService, "generateCoverLetter").mockResolvedValue({
      output: "Madame, Monsieur,", elapsed_ms: 1000, tokens_used: 10,
    });
    vi.spyOn(documentsService, "saveCoverLetter").mockRejectedValue(
      new AppError({ code: "VALIDATION_ERROR", message: "Contenu invalide." }),
    );
    const { result } = renderHook(() => useLetterWriterViewModel(null), { wrapper });
    act(() => result.current.setCompany("Astek"));
    await act(async () => { await result.current.run(null); });
    await waitFor(() => expect(result.current.output).toContain("Madame"));
    act(() => { result.current.save(); });
    await waitFor(() =>
      expect(useUiStore.getState().toasts.at(-1)?.title).toBe("Enregistrement impossible"),
    );
  });
});
