import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { ResumeAtsPanel } from "../ResumeAtsPanel";
import { workspaceFixture } from "../../../model/resumeWorkspace";
import type { ResumeWorkspace } from "@/shared/types/generated/documents";
import type { ComponentProps } from "react";

function assistantWorkspace(): ResumeWorkspace {
  const base = workspaceFixture({ skill_groups: [] });
  return {
    ...base,
    job_offer: { ...base.job_offer, title: "Administrateur systèmes", skills: ["Docker", "Kubernetes"] },
    score: { ...base.score, missing: ["Docker", "Kubernetes"] },
    profile_library: [
      { id: "skill-docker", label: "Docker", detail: null, content: { type: "skill", name: "Docker" } },
      { id: "project-lab", label: "Homelab", detail: "Proxmox", content: { type: "project", value: { id: "project-lab", name: "Homelab", meta: "Proxmox", url: null, bullets: [] } } },
    ],
    content_recommendations: [
      {
        id: "recommend-skill-docker",
        label: "Docker",
        reason: "Directement demandé dans l’offre.",
        relevance: "very_relevant",
        action: { type: "add", item_id: "skill-docker" },
        layout_after: base.layout,
      },
    ],
  };
}

function renderPanel(workspace: ResumeWorkspace, overrides: Partial<ComponentProps<typeof ResumeAtsPanel>> = {}) {
  const props = {
    workspace,
    onAddProfileItem: vi.fn(),
    onApplyRecommendation: vi.fn(),
    onIgnoreRecommendation: vi.fn(),
    onAccept: vi.fn(),
    onReject: vi.fn(),
    onUndo: vi.fn(),
    ...overrides,
  };
  render(<ResumeAtsPanel {...props} />);
  return props;
}

describe("ResumeAtsPanel", () => {
  it("sépare la recommandation, la bibliothèque et une compétence absente du profil", () => {
    renderPanel(assistantWorkspace());
    expect(screen.getByRole("heading", { name: "Recommandé pour cette offre" })).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Disponible dans votre profil" })).toBeInTheDocument();
    expect(screen.getByText("Très pertinent")).toBeInTheDocument();
    expect(screen.getByText("Kubernetes")).toBeInTheDocument();
    expect(screen.getByText(/absentes de votre profil/i)).toBeInTheDocument();
  });

  it("laisse l'utilisateur ajouter manuellement un contenu réel du profil", async () => {
    const add = vi.fn();
    renderPanel(assistantWorkspace(), { onAddProfileItem: add });
    await userEvent.click(screen.getByRole("button", { name: "Ajouter Homelab" }));
    expect(add).toHaveBeenCalledWith("project-lab");
  });

  it("permet d'accepter ou d'ignorer une recommandation", async () => {
    const apply = vi.fn();
    const ignore = vi.fn();
    renderPanel(assistantWorkspace(), { onApplyRecommendation: apply, onIgnoreRecommendation: ignore });
    await userEvent.click(screen.getAllByRole("button", { name: "Ajouter" })[0]!);
    expect(apply).toHaveBeenCalledWith("recommend-skill-docker");
    await userEvent.click(screen.getByRole("button", { name: "Ignorer" }));
    expect(ignore).toHaveBeenCalledWith("recommend-skill-docker");
  });

  it("avertit sans supprimer les choix quand la page déborde", () => {
    const workspace = assistantWorkspace();
    workspace.layout = { status: "overflow", used_per_mille: 1120, remaining_points: -24, page_count: 2, overflow: true };
    workspace.content_recommendations = [];
    renderPanel(workspace);
    expect(screen.getByText("Dépassement")).toBeInTheDocument();
    expect(screen.getByText(/Vos choix sont conservés/i)).toBeInTheDocument();
    expect(screen.getByText("Aucun ajout conseillé")).toBeInTheDocument();
  });

  it("garde les suggestions locales utilisables quand l'IA est indisponible", () => {
    const workspace = assistantWorkspace();
    workspace.recommendation_error = "Connexion au fournisseur impossible";
    workspace.content_recommendations = [];
    const add = vi.fn();
    renderPanel(workspace, { onAddProfileItem: add });
    expect(screen.getByText("Recommandations IA indisponibles")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Ajouter Homelab" })).toBeEnabled();
  });

  it("explique pourquoi aucune recommandation n'est calculée sans offre", () => {
    const workspace = assistantWorkspace();
    workspace.job_offer = { title: "", skills: [], soft_skills: [], experience: null, keywords: [] };
    workspace.content_recommendations = [];
    renderPanel(workspace);
    expect(screen.getByText("Offre absente")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Ajouter Homelab" })).toBeEnabled();
  });

  it("affiche des états utiles quand la sélection et la bibliothèque sont vides", () => {
    const workspace = assistantWorkspace();
    workspace.content_recommendations = [];
    workspace.profile_library = [];
    workspace.score = { ...workspace.score, missing: [] };
    renderPanel(workspace);
    expect(screen.getByText("Aucune recommandation prioritaire")).toBeInTheDocument();
    expect(screen.getByText("Tout est à jour")).toBeInTheDocument();
  });

  it("annonce le recalcul sans masquer la bibliothèque", () => {
    renderPanel(assistantWorkspace(), { busy: true });
    expect(screen.getByText("Mise à jour en cours")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Ajouter Homelab" })).toBeDisabled();
  });
});
