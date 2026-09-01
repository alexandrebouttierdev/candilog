import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { ResumeAtsPanel } from "../ResumeAtsPanel";
import { workspaceFixture } from "../../../model/resumeWorkspace";
import type { ResumeWorkspace } from "@/shared/types/generated/documents";

/**
 * Workspace avec une unique proposition « missing_skill » en attente. Le score courant
 * (68) et le score initial sont écartés du même montant que le gain de la proposition, de
 * sorte que le gain cumulé affiché (`score.total - initial_score`) et le gain de la carte
 * pointent vers le même nombre dans ce scénario à une seule décision.
 */
function workspaceWithProposal({ gain }: { gain: number }): ResumeWorkspace {
  const base = workspaceFixture();
  return {
    ...base,
    score: { ...base.score, total: 68 },
    initial_score: 68 - gain,
    proposals: [
      {
        id: "missing-skill-docker",
        kind: "missing_skill",
        target: { type: "skill_group", group_id: "group-1" },
        label: "Docker",
        original_text: null,
        proposed_text: "Docker",
        gain,
        status: "pending",
        applicable: true,
      },
    ],
  };
}

describe("ResumeAtsPanel", () => {
  it("affiche le score projeté et applique une compétence", async () => {
    const accept = vi.fn();
    const reject = vi.fn();
    const undo = vi.fn();
    render(
      <ResumeAtsPanel
        workspace={workspaceWithProposal({ gain: 5 })}
        onAccept={accept}
        onReject={reject}
        onUndo={undo}
      />,
    );

    expect(screen.getByText("68")).toBeInTheDocument();
    expect(screen.getByText("73")).toBeInTheDocument();
    expect(screen.getByText("+5 points")).toBeInTheDocument();

    await userEvent.click(screen.getByRole("button", { name: "Accepter Docker" }));
    expect(accept).toHaveBeenCalledWith("missing-skill-docker");
  });

  it("propose d'annuler une décision déjà prise plutôt que de la refaire", async () => {
    const accept = vi.fn();
    const reject = vi.fn();
    const undo = vi.fn();
    const workspace = workspaceWithProposal({ gain: 5 });
    workspace.proposals = [{ ...workspace.proposals[0]!, status: "accepted" }];

    render(<ResumeAtsPanel workspace={workspace} onAccept={accept} onReject={reject} onUndo={undo} />);

    expect(screen.queryByRole("button", { name: "Accepter Docker" })).not.toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "Annuler Docker" }));
    expect(undo).toHaveBeenCalledWith("missing-skill-docker");
  });

  it("annule uniquement la proposition ciblée quand deux décisions sont prises", async () => {
    const undo = vi.fn();
    const workspace = workspaceWithProposal({ gain: 5 });
    workspace.proposals = [
      { ...workspace.proposals[0]!, status: "accepted" },
      {
        id: "text-reform",
        kind: "text_replacement",
        target: { type: "profile" },
        label: "Profil",
        original_text: "Avant",
        proposed_text: "Après",
        gain: 3,
        status: "accepted",
        applicable: true,
      },
    ];

    render(<ResumeAtsPanel workspace={workspace} onAccept={vi.fn()} onReject={vi.fn()} onUndo={undo} />);

    await userEvent.click(screen.getByRole("button", { name: "Annuler Docker" }));

    expect(undo).toHaveBeenCalledTimes(1);
    expect(undo).toHaveBeenCalledWith("missing-skill-docker");
    expect(screen.getByRole("button", { name: "Annuler Profil" })).toBeInTheDocument();
  });

  it("n'offre aucune action pour une proposition qui ne s'applique plus", () => {
    const workspace = workspaceWithProposal({ gain: 5 });
    workspace.proposals = [{ ...workspace.proposals[0]!, applicable: false }];

    render(<ResumeAtsPanel workspace={workspace} onAccept={vi.fn()} onReject={vi.fn()} onUndo={vi.fn()} />);

    expect(screen.queryByRole("button", { name: "Accepter Docker" })).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Refuser Docker" })).not.toBeInTheDocument();
  });
});
