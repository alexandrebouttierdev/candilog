import { beforeEach, describe, expect, it, vi } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { QueryWrapper } from "@/shared/lib/test-utils";
import { applicationService, EMPTY_FILTER } from "@/features/applications";
import type { ApplicationFilter } from "@/features/applications";
import { viewsService } from "../../services/viewsService";
import type { SavedView } from "../../services/viewsService";
import { useSavedViews } from "../useSavedViews";

function filtre(overrides: Partial<ApplicationFilter> = {}): ApplicationFilter {
  return { ...EMPTY_FILTER, search: "", sort: "date", descending: true, ids: [], ...overrides };
}

function vue(id: string, filter: ApplicationFilter): SavedView {
  return { id, name: id, filter, position: 1, created_at: "2026-09-01", updated_at: "2026-09-01" };
}

beforeEach(() => {
  vi.restoreAllMocks();
  vi.spyOn(applicationService, "breakdown").mockResolvedValue({ pending: 4, followed_up: 2, interview: 1, rejected: 3 });
});

describe("vues enregistrées — décompte", () => {
  it("ne compte que les statuts retenus par la vue", async () => {
    vi.spyOn(viewsService, "list").mockResolvedValue([
      vue("a-relancer", filtre({ status: ["EN_ATTENTE", "RELANCEE"] })),
      vue("toutes", filtre()),
      vue("sans-refus", filtre({ status: ["REFUS"], excluded: ["status"] })),
    ]);

    const { result } = renderHook(() => useSavedViews(), { wrapper: QueryWrapper });

    await waitFor(() => expect(result.current.countOf("a-relancer")).toBe(6));
    await waitFor(() => expect(result.current.countOf("toutes")).toBe(10));
    // Statut exclu : tout sauf les refus.
    await waitFor(() => expect(result.current.countOf("sans-refus")).toBe(7));
  });
});
