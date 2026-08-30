import type { ReactNode } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter } from "react-router-dom";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { documentsService } from "../../../services/documentsService";
import { useUiStore } from "@/shared/lib/ui-store";
import { ResumeLibraryPage } from "../ResumeLibraryPage";

function wrapper({ children }: { children: ReactNode }) {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return <QueryClientProvider client={client}><MemoryRouter>{children}</MemoryRouter></QueryClientProvider>;
}

beforeEach(() => {
  vi.restoreAllMocks();
  useUiStore.setState({ toasts: [] });
});

describe("bibliothèque de CV paginée", () => {
  it("recherche et change de page côté backend sans charger la liste exhaustive", async () => {
    const exhaustive = vi.spyOn(documentsService, "listResume").mockResolvedValue([]);
    const paged = vi.spyOn(documentsService, "listResumePage").mockImplementation(({ page, page_size, search }) => Promise.resolve({
      items: [{ id: `${search || "cv"}-${page}`, name: `${search || "CV"} page ${page}`, created_at: "2026-08-30T00:00:00Z" }],
      total: search ? 1 : 9,
      page,
      page_size,
      total_pages: search ? 1 : 2,
    }));
    vi.spyOn(documentsService, "getResume").mockImplementation((id) => Promise.resolve({ id, name: id, content: null, created_at: "2026-08-30T00:00:00Z" }));

    render(<ResumeLibraryPage />, { wrapper });
    await waitFor(() => expect(paged).toHaveBeenCalledWith({ page: 1, page_size: 8, search: "" }));

    await userEvent.click(screen.getByRole("button", { name: "Page suivante" }));
    await waitFor(() => expect(paged).toHaveBeenCalledWith({ page: 2, page_size: 8, search: "" }));

    const search = screen.getByPlaceholderText("Rechercher une version…");
    await userEvent.type(search, "cible");
    await waitFor(() => expect(paged).toHaveBeenCalledWith({ page: 1, page_size: 8, search: "cible" }));
    expect(exhaustive).not.toHaveBeenCalled();
  });
});
