import type { ReactNode } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { companyService } from "@/features/companies/services/companyService";
import { ContactFormModal } from "../ContactFormModal";

function wrapper({ children }: { children: ReactNode }) {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return <QueryClientProvider client={client}>{children}</QueryClientProvider>;
}

describe("ContactFormModal", () => {
  it("alimente le sélecteur d'entreprise avec une page backend", async () => {
    const list = vi.spyOn(companyService, "list").mockResolvedValue([]);
    const listPage = vi.spyOn(companyService, "listPage").mockResolvedValue({
      items: [],
      total: 0,
      page: 1,
      page_size: 4,
      total_pages: 0,
    });

    render(
      <ContactFormModal
        open
        contact={null}
        busy={false}
        onClose={vi.fn()}
        onSubmit={vi.fn()}
      />,
      { wrapper },
    );
    await userEvent.click(screen.getByPlaceholderText("Rechercher une entreprise…"));

    await waitFor(() =>
      expect(listPage).toHaveBeenCalledWith({
        page: 1,
        page_size: 4,
        search: "",
        company_type: null,
      }),
    );
    expect(list).not.toHaveBeenCalled();
  });
});
