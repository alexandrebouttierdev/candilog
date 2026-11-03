import type { ReactNode } from "react";
import { QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { followUpService, type FollowUp, type NewFollowUp } from "@/features/followups";
import { interviewService } from "@/features/interviews";
import { createTestQueryClient } from "@/shared/lib/test-utils";
import { useUiStore } from "@/shared/lib/ui-store";
import { AppError } from "@/shared/types/app-error";
import { gridBounds } from "../../model/month";
import { useCalendarViewModel } from "../useCalendarViewModel";

function relance(id: string, follow_up_date: string): FollowUp {
  return {
    id,
    application_id: "candidature-1",
    application_job_title: "Développeuse Rust",
    company_name: "Nova",
    follow_up_date,
    type: "Email",
    notes: null,
    done_at: null,
    created_at: "2026-09-01T08:00:00Z",
  };
}

const saisie: NewFollowUp = {
  application_id: "candidature-1",
  follow_up_date: "2026-10-15",
  type: "Email",
  notes: null,
};

function rendu() {
  const client = createTestQueryClient();
  const wrapper = ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={client}>{children}</QueryClientProvider>
  );
  return renderHook(() => useCalendarViewModel(), { wrapper });
}

beforeEach(() => {
  vi.restoreAllMocks();
  useUiStore.setState({ toasts: [] });
  vi.spyOn(interviewService, "listBetween").mockResolvedValue([]);
});

describe("calendrier", () => {
  it("charge toute la grille affichée, cases des mois voisins comprises", async () => {
    const relances = vi.spyOn(followUpService, "listBetween").mockResolvedValue([]);
    const { result } = rendu();

    act(() => result.current.allerA(2026, 9));

    const { from, to } = gridBounds(2026, 9);
    // Octobre 2026 commence un jeudi : la grille déborde sur la fin septembre.
    expect(from < "2026-10-01").toBe(true);
    await waitFor(() => expect(relances).toHaveBeenCalledWith(from, to));
    expect(interviewService.listBetween).toHaveBeenCalledWith(`${from}T00:00:00`, `${to}T23:59:59`);
  });

  it("ne compte que les événements du mois, mais affiche ceux des cases voisines", async () => {
    const { from } = gridBounds(2026, 9);
    vi.spyOn(followUpService, "listBetween").mockResolvedValue([
      relance("dans-le-mois", "2026-10-15"),
      relance("case-voisine", from),
    ]);
    const { result } = rendu();

    act(() => result.current.allerA(2026, 9));

    await waitFor(() => expect(result.current.isLoading).toBe(false));
    await waitFor(() => expect(result.current.parDay.get(from)?.length).toBe(1));
    expect(result.current.countFollowUps).toBe(1);
    expect(result.current.countInterviews).toBe(0);
    expect(result.current.followUpDe("case-voisine")?.follow_up_date).toBe(from);
  });

  it("change d'année en naviguant au-delà de décembre et avant janvier", () => {
    vi.spyOn(followUpService, "listBetween").mockResolvedValue([]);
    const { result } = rendu();

    act(() => result.current.allerA(2026, 11));
    act(() => result.current.naviguer(1));
    expect([result.current.year, result.current.month]).toEqual([2027, 0]);
    act(() => result.current.naviguer(-1));
    expect([result.current.year, result.current.month]).toEqual([2026, 11]);
  });

  it("crée une relance neuve et met à jour une relance existante", async () => {
    vi.spyOn(followUpService, "listBetween").mockResolvedValue([]);
    const create = vi.spyOn(followUpService, "create").mockResolvedValue(relance("r1", "2026-10-15"));
    const update = vi.spyOn(followUpService, "update").mockResolvedValue(relance("r2", "2026-10-15"));
    const { result } = rendu();

    await act(async () => {
      await result.current.saveFollowUp({ id: null, input: saisie });
    });
    await act(async () => {
      await result.current.saveFollowUp({ id: "r2", input: saisie });
    });

    expect(create).toHaveBeenCalledWith(saisie);
    expect(update).toHaveBeenCalledWith("r2", saisie);
    expect(useUiStore.getState().toasts[0]).toMatchObject({ tone: "success", title: "Relance enregistrée" });
  });

  it("signale l'échec d'une suppression avec le message du natif", async () => {
    vi.spyOn(followUpService, "listBetween").mockResolvedValue([]);
    vi.spyOn(followUpService, "delete").mockRejectedValue(
      new AppError({ code: "NOT_FOUND", message: "Introuvable : relance." }),
    );
    const { result } = rendu();

    await act(async () => {
      await result.current.deleteFollowUp("absente").catch(() => undefined);
    });

    expect(useUiStore.getState().toasts[0]).toMatchObject({
      tone: "error",
      title: "Suppression impossible",
      detail: "Introuvable : relance.",
    });
  });
});
