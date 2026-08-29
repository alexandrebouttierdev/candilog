import { describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen } from "@testing-library/react";
import { KanbanBoard } from "../KanbanBoard";
import type { Application } from "../../../services/applicationService";

function dataTransfer(): DataTransfer {
  const store: Record<string, string> = {};
  return {
    setData(type: string, value: string) {
      store[type] = value;
    },
    getData(type: string) {
      return store[type] ?? "";
    },
    effectAllowed: "all",
    dropEffect: "none",
    setDragImage: vi.fn(),
  } as unknown as DataTransfer;
}

function cand(job_title: string, status: Application["status"]): Application {
  return {
    id: job_title,
    job_title,
    company_id: "e1",
    company_name: "Nova Digital",
    company_city: "Rennes",
    contact_id: null,
    contract_type: "CDI",
    status,
    sent_date: "2026-08-20",
    job_url: null,
    notes: null,
    created_at: "2026-08-20T00:00:00Z",
    updated_at: "2026-08-20T00:00:00Z",
  };
}

describe("KanbanBoard", () => {
  it("dépose la carte même si le navigateur envoie dragend avant drop", () => {
    // WebKit (Tauri Linux) vide l'état React au dragend, avant le drop : le statut
    // doit voyager dans dataTransfer, pas dans un useState.
    const onStatusChange = vi.fn();
    render(
      <KanbanBoard
        applications={[cand("Développeur", "EN_ATTENTE")]}
        breakdown={{ pending: 1, followed_up: 0, interview: 0, rejected: 0 }}
        selected_id={null}
        checkedIds={new Set()}
        onSelect={vi.fn()}
        onToggleSelect={vi.fn()}
        onStatusChange={onStatusChange}
        onCreate={vi.fn()}
      />,
    );

    const carte = screen.getByText("Développeur").closest("article");
    const entretien = screen.getByRole("heading", { name: "Entretien" }).closest("section");
    expect(carte).toBeTruthy();
    expect(entretien).toBeTruthy();

    const transfert = dataTransfer();
    fireEvent.dragStart(carte!, { dataTransfer: transfert });
    fireEvent.dragEnd(carte!);
    fireEvent.dragOver(entretien!, { dataTransfer: transfert });
    fireEvent.drop(entretien!, { dataTransfer: transfert });

    expect(onStatusChange).toHaveBeenCalledWith("Développeur", "ENTRETIEN");
  });

  it("impose une miniature plutôt que le cliché natif de la carte", () => {
    // WebKit photographie la couche entière (rail, verre, colonnes) et l'affiche
    // sous le curseur à taille réelle — la carte paraît alors énorme.
    render(
      <KanbanBoard
        applications={[cand("Développeur", "EN_ATTENTE")]}
        breakdown={{ pending: 1, followed_up: 0, interview: 0, rejected: 0 }}
        selected_id={null}
        checkedIds={new Set()}
        onSelect={vi.fn()}
        onToggleSelect={vi.fn()}
        onStatusChange={vi.fn()}
        onCreate={vi.fn()}
      />,
    );

    const carte = screen.getByText("Développeur").closest("article");
    const setDragImage = vi.fn();
    const transfert = dataTransfer();
    transfert.setDragImage = setDragImage;
    fireEvent.dragStart(carte!, { dataTransfer: transfert });

    expect(setDragImage).toHaveBeenCalled();
  });

  it("demande une création au statut de la colonne cliquée", () => {
    const onCreate = vi.fn();
    render(
      <KanbanBoard
        applications={[cand("Développeur", "EN_ATTENTE")]}
        breakdown={{ pending: 1, followed_up: 0, interview: 0, rejected: 0 }}
        selected_id={null}
        checkedIds={new Set()}
        onSelect={vi.fn()}
        onToggleSelect={vi.fn()}
        onStatusChange={vi.fn()}
        onCreate={onCreate}
      />,
    );

    fireEvent.click(
      screen.getByRole("button", { name: "Nouvelle candidature au statut Entretien" }),
    );

    expect(onCreate).toHaveBeenCalledWith("ENTRETIEN");
  });
});
