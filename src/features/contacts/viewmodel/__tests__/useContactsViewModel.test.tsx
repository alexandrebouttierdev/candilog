import { beforeEach, describe, expect, it, vi } from "vitest";
import { renderHook, waitFor, act } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import type { ReactNode } from "react";
import { useContactsViewModel } from "../useContactsViewModel";
import { contactService } from "../../services/contactService";
import type { Contact } from "../../services/contactService";
import { useUiStore } from "@/shared/lib/ui-store";

function ct(name: string): Contact {
  return {
    id: name,
    company_id: null,
    company_name: null,
    first_name: "Camille",
    name,
    job_title: null,
    tracking_role: "Recruteur",
    email: null,
    phone: null,
    linkedin: null,
    notes: null,
    created_at: "2026-01-01T00:00:00Z",
    updated_at: "2026-01-01T00:00:00Z",
  };
}

function page(items: Contact[], total = items.length) {
  return { items, total, page: 1, page_size: 8, total_pages: Math.max(1, Math.ceil(total / 8)) };
}

function wrapper({ children }: { children: ReactNode }) {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
  return <QueryClientProvider client={client}>{children}</QueryClientProvider>;
}

beforeEach(() => {
  vi.restoreAllMocks();
  useUiStore.setState({ toasts: [] });
});

describe("ViewModel du réseau", () => {
  it("transmet le rôle au backend au lieu de filtrer localement", async () => {
    const listPage = vi.spyOn(contactService, "listPage").mockResolvedValue(page([ct("Rivet")]));

    const { result } = renderHook(() => useContactsViewModel(), { wrapper });
    await waitFor(() => expect(listPage).toHaveBeenCalled());

    act(() => result.current.filtrerParRole("Manager"));

    await waitFor(() =>
      expect(listPage).toHaveBeenLastCalledWith(
        expect.objectContaining({ tracking_role: "Manager" }),
      ),
    );
  });

  it("compte le rôle actif sans la recherche libre", async () => {
    vi.spyOn(contactService, "listPage").mockResolvedValue(page([ct("Rivet")]));

    const { result } = renderHook(() => useContactsViewModel(), { wrapper });
    await waitFor(() => expect(result.current.items).toHaveLength(1));

    act(() => result.current.rechercher("camille"));
    expect(result.current.filtersActifs).toBe(0);

    act(() => result.current.filtrerParRole("Recruteur"));
    expect(result.current.filtersActifs).toBe(1);
  });

  it("revient à la première page et ôte le rôle au reset", async () => {
    const listPage = vi.spyOn(contactService, "listPage").mockResolvedValue(page([ct("Rivet")], 40));

    const { result } = renderHook(() => useContactsViewModel(), { wrapper });
    await waitFor(() => expect(listPage).toHaveBeenCalled());

    act(() => result.current.setPage(3));
    act(() => result.current.filtrerParRole("Manager"));
    await waitFor(() => expect(result.current.page).toBe(1));

    act(() => result.current.setPage(2));
    act(() => result.current.resetFilters());

    expect(result.current.tracking_role).toBeNull();
    expect(result.current.page).toBe(1);
  });
});
