import { describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen } from "@testing-library/react";
import { KanbanBoard } from "../KanbanBoard";
import type { Application } from "../../../services/applicationService";
import type { ApplicationStatus } from "../../../services/applicationService";
import type { Page } from "@/shared/types/page";

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
    company_size: "PME",
    contact_id: null,
    application_type: "OFFRE",
    contract_type_code: "CDI",
    contract_type_name: "CDI",
    weekly_work_schedule: "FULL_TIME",
    weekly_hours: 35,
    professional_domain_id: "M18",
    professional_domain_name: "Informatique / Télécommunication",
    city: null,
    address: null,
    company_type_id: null,
    effective_city: "Rennes",
    effective_address: null,
    effective_company_type_id: "IT_SERVICES_COMPANY",
    effective_company_type_name: "ESN / Société de services numériques",
    status,
    sent_date: "2026-08-20",
    job_url: null,
    notes: null,
    created_at: "2026-08-20T00:00:00Z",
    updated_at: "2026-08-20T00:00:00Z",
  };
}

function columns(
  applications: readonly Application[],
  totals: Partial<Record<ApplicationStatus, number>> = {},
): Record<ApplicationStatus, Page<Application>> {
  const column = (status: ApplicationStatus): Page<Application> => {
    const items = applications.filter((application) => application.status === status);
    const total = totals[status] ?? items.length;
    return {
      items,
      total,
      page: 1,
      page_size: 8,
      total_pages: Math.max(1, Math.ceil(total / 8)),
    };
  };
  return {
    EN_ATTENTE: column("EN_ATTENTE"),
    RELANCEE: column("RELANCEE"),
    ENTRETIEN: column("ENTRETIEN"),
    REFUS: column("REFUS"),
  };
}

describe("KanbanBoard", () => {
  it("dépose la carte même si le navigateur envoie dragend avant drop", () => {
    // WebKit (Tauri Linux) vide l'état React au dragend, avant le drop : le statut
    // doit voyager dans dataTransfer, pas dans un useState.
    const onStatusChange = vi.fn();
    render(
      <KanbanBoard
        columns={columns([cand("Développeur", "EN_ATTENTE")])}
        selected_id={null}
        checkedIds={new Set()}
        onSelect={vi.fn()}
        onToggleSelect={vi.fn()}
        onStatusChange={onStatusChange}
        onCreate={vi.fn()}
        onPageChange={vi.fn()}
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
        columns={columns([cand("Développeur", "EN_ATTENTE")])}
        selected_id={null}
        checkedIds={new Set()}
        onSelect={vi.fn()}
        onToggleSelect={vi.fn()}
        onStatusChange={vi.fn()}
        onCreate={vi.fn()}
        onPageChange={vi.fn()}
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
        columns={columns([cand("Développeur", "EN_ATTENTE")])}
        selected_id={null}
        checkedIds={new Set()}
        onSelect={vi.fn()}
        onToggleSelect={vi.fn()}
        onStatusChange={vi.fn()}
        onCreate={onCreate}
        onPageChange={vi.fn()}
      />,
    );

    fireEvent.click(
      screen.getByRole("button", { name: "Nouvelle candidature au statut Entretien" }),
    );

    expect(onCreate).toHaveBeenCalledWith("ENTRETIEN");
  });

  it("affiche une pagination dans une colonne qui contient plus d'une page", () => {
    const onPageChange = vi.fn();
    render(
      <KanbanBoard
        columns={columns([cand("Développeur", "EN_ATTENTE")], { EN_ATTENTE: 9 })}
        selected_id={null}
        checkedIds={new Set()}
        onSelect={vi.fn()}
        onToggleSelect={vi.fn()}
        onStatusChange={vi.fn()}
        onCreate={vi.fn()}
        onPageChange={onPageChange}
      />,
    );

    const attente = screen.getByRole("heading", { name: "En attente" }).closest("section");
    expect(attente).toBeTruthy();
    expect(attente).toHaveTextContent("1–8 sur 9");
    expect(attente).toContainElement(
      screen.getByRole("button", { name: "Page suivante de En attente" }),
    );
    fireEvent.click(screen.getByRole("button", { name: "Page suivante de En attente" }));
    expect(onPageChange).toHaveBeenCalledWith("EN_ATTENTE", 2);
  });
});
