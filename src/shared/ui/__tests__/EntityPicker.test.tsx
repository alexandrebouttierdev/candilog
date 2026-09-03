import type { ReactNode } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { EntityPicker } from "../EntityPicker";
import type { EntityOption } from "../EntityPicker";
import type { Page } from "@/shared/types/page";

function wrapper({ children }: { children: ReactNode }) {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  return <QueryClientProvider client={client}>{children}</QueryClientProvider>;
}

describe("EntityPicker", () => {
  it("recherche côté données, revient en page 1 et conserve le libellé choisi", async () => {
    const user = userEvent.setup();
    const fetchPage = vi.fn(
      ({ page, page_size, search }: { page: number; page_size: number; search: string }): Promise<Page<EntityOption>> =>
        Promise.resolve({
          items: [{ id: `${search || "tous"}-${page}`, label: `Résultat ${search || page}` }],
          total: search ? 1 : 6,
          page,
          page_size,
          total_pages: search ? 1 : 2,
        }),
    );
    const onChange = vi.fn();

    const { rerender } = render(
      <EntityPicker
        value={null}
        selectedLabel={null}
        placeholder="Rechercher…"
        queryKey={["test-picker"]}
        fetchPage={fetchPage}
        onChange={onChange}
      />,
      { wrapper },
    );

    const input = screen.getByRole("combobox");
    await user.click(input);
    await waitFor(() =>
      expect(fetchPage).toHaveBeenCalledWith({ page: 1, page_size: 4, search: "" }),
    );

    await user.click(screen.getByRole("button", { name: "Page suivante" }));
    await waitFor(() =>
      expect(fetchPage).toHaveBeenCalledWith({ page: 2, page_size: 4, search: "" }),
    );

    await user.type(input, "nova");
    await waitFor(() =>
      expect(fetchPage).toHaveBeenCalledWith({ page: 1, page_size: 4, search: "nova" }),
    );

    await user.click(await screen.findByRole("option", { name: "Résultat nova" }));
    expect(onChange).toHaveBeenCalledWith("nova-1");

    rerender(
      <EntityPicker
        value="nova-1"
        selectedLabel={null}
        placeholder="Rechercher…"
        queryKey={["test-picker"]}
        fetchPage={fetchPage}
        onChange={onChange}
      />,
    );
    expect(screen.getByRole("combobox")).toHaveValue("Résultat nova");
  });
});

describe("EntityPicker — création et erreurs", () => {
  const page = (items: EntityOption[]): Promise<Page<EntityOption>> =>
    Promise.resolve({ items, total: items.length, page: 1, page_size: 4, total_pages: 1 });

  it("propose de créer l'entité recherchée quand rien ne correspond", async () => {
    const user = userEvent.setup();
    const onCreate = vi.fn();

    render(
      <EntityPicker
        value={null}
        selectedLabel={null}
        placeholder="Rechercher…"
        queryKey={["creation-picker"]}
        fetchPage={() => page([])}
        onChange={vi.fn()}
        onCreate={onCreate}
        createLabel="Créer"
      />,
      { wrapper },
    );

    await user.click(screen.getByRole("combobox"));
    await user.type(screen.getByRole("combobox"), "Nova Digital");

    const bouton = await screen.findByRole("button", { name: "Créer « Nova Digital »" });
    await user.click(bouton);

    expect(onCreate).toHaveBeenCalledWith("Nova Digital");
  });

  it("distingue un échec de chargement d'une absence de résultat", async () => {
    const user = userEvent.setup();

    render(
      <EntityPicker
        value={null}
        selectedLabel={null}
        placeholder="Rechercher…"
        emptyHelp="Aucun résultat."
        queryKey={["erreur-picker"]}
        fetchPage={() => Promise.reject(new Error("IPC indisponible"))}
        onChange={vi.fn()}
      />,
      { wrapper },
    );

    await user.click(screen.getByRole("combobox"));

    expect(await screen.findByText("La recherche a échoué.")).toBeInTheDocument();
    expect(screen.queryByText("Aucun résultat.")).not.toBeInTheDocument();
  });
});
