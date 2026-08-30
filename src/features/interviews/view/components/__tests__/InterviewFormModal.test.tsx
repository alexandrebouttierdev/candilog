import type { ReactNode } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { contactService } from "@/features/contacts/services/contactService";
import { InterviewFormModal } from "../InterviewFormModal";

function wrapper({ children }: { children: ReactNode }) {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return <QueryClientProvider client={client}>{children}</QueryClientProvider>;
}

describe("InterviewFormModal", () => {
  it("alimente le sélecteur de contact avec une page backend", async () => {
    const list = vi.spyOn(contactService, "list").mockResolvedValue([]);
    const listPage = vi.spyOn(contactService, "listPage").mockResolvedValue({
      items: [],
      total: 0,
      page: 1,
      page_size: 4,
      total_pages: 0,
    });

    render(
      <InterviewFormModal
        open
        interview={null}
        busy={false}
        onClose={vi.fn()}
        onSubmit={vi.fn()}
      />,
      { wrapper },
    );
    await userEvent.click(screen.getByPlaceholderText("Rechercher un contact…"));

    await waitFor(() =>
      expect(listPage).toHaveBeenCalledWith({
        page: 1,
        page_size: 4,
        search: "",
        tracking_role: null,
      }),
    );
    expect(list).not.toHaveBeenCalled();
  });
});
