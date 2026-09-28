import { beforeEach, describe, expect, it, vi } from "vitest";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClientProvider } from "@tanstack/react-query";
import { MemoryRouter } from "react-router-dom";
import type { ReactNode } from "react";
import { createTestQueryClient } from "@/shared/lib/test-utils";
import { applicationService, EMPTY_FILTER } from "@/features/applications";
import { viewsService } from "@/features/views";
import type { SavedView } from "@/features/views";
import { SavedViewsNav } from "../SavedViewsNav";

const VUE: SavedView = {
  id: "v-1",
  name: "À relancer",
  filter: { ...EMPTY_FILTER, status: ["RELANCEE"], search: "", sort: "date", descending: true, ids: [] },
  position: 1,
  created_at: "2026-09-01",
  updated_at: "2026-09-01",
};

function wrapper({ children }: { children: ReactNode }) {
  return (
    <QueryClientProvider client={createTestQueryClient()}>
      <MemoryRouter>{children}</MemoryRouter>
    </QueryClientProvider>
  );
}

beforeEach(() => {
  vi.restoreAllMocks();
  vi.spyOn(applicationService, "breakdown").mockResolvedValue({ pending: 1, followed_up: 3, interview: 0, rejected: 0 });
});

describe("navigation — vues enregistrées", () => {
  it("ouvre Candidatures sur la vue, avec son décompte", async () => {
    vi.spyOn(viewsService, "list").mockResolvedValue([VUE]);
    render(<SavedViewsNav />, { wrapper });

    const lien = await screen.findByRole("link", { name: /À relancer/ });
    expect(lien).toHaveAttribute("href", "/applications?view=v-1");
    await waitFor(() => expect(lien).toHaveTextContent("3"));
  });

  it("supprime une vue après confirmation", async () => {
    vi.spyOn(viewsService, "list").mockResolvedValue([VUE]);
    const supprimer = vi.spyOn(viewsService, "delete").mockResolvedValue(undefined);
    render(<SavedViewsNav />, { wrapper });

    await userEvent.click(await screen.findByRole("button", { name: "Actions sur la vue À relancer" }));
    await userEvent.click(await screen.findByRole("menuitem", { name: "Supprimer la vue…" }));
    await userEvent.click(within(screen.getByRole("alertdialog")).getByRole("button", { name: "Supprimer la vue" }));

    await waitFor(() => expect(supprimer).toHaveBeenCalledWith("v-1"));
  });

  it("explique comment créer une vue quand il n'y en a aucune", async () => {
    vi.spyOn(viewsService, "list").mockResolvedValue([]);
    render(<SavedViewsNav />, { wrapper });

    expect(await screen.findByText(/Enregistrer la vue/)).toBeInTheDocument();
  });
});
