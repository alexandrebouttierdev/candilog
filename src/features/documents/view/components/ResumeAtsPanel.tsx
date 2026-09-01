import type { ResumeProposal, ResumeWorkspace } from "@/shared/types/generated/documents";
import { Button, EmptyState, StatusPill } from "@/shared/ui";
import { cn } from "@/shared/lib/cn";
import { ScoreBadge } from "./DocumentUi";

/**
 * Panneau de décisions ATS de l'éditeur de CV.
 *
 * Rend le score courant, le gain cumulé depuis la génération
 * (`workspace.score.total - workspace.initial_score`), puis une carte par proposition avec
 * son état (`pending`, `accepted`, `rejected` ou non applicable) et, pour une proposition en
 * attente et applicable, le score qu'elle apporterait si elle était acceptée.
 */
export function ResumeAtsPanel({
  workspace,
  onAccept,
  onReject,
  onUndo,
  busy = false,
}: {
  workspace: ResumeWorkspace;
  onAccept: (proposal_id: string) => void;
  onReject: (proposal_id: string) => void;
  onUndo: (proposal_id: string) => void;
  busy?: boolean;
}) {
  const cumulativeGain = workspace.score.total - workspace.initial_score;

  return (
    <div className="space-y-5 p-4">
      <div className="flex items-center gap-3">
        <ScoreBadge value={workspace.score.total} />
        {cumulativeGain !== 0 ? (
          <span
            className={cn(
              "text-label font-medium",
              cumulativeGain > 0 ? "text-success" : "text-danger",
            )}
          >
            {cumulativeGain > 0 ? `+${cumulativeGain} points` : `${cumulativeGain} points`}
          </span>
        ) : null}
      </div>
      {workspace.proposals.length === 0 ? (
        <EmptyState
          icon="query_stats"
          title="Aucune proposition"
          description="Le CV correspond déjà pleinement à l’offre analysée."
        />
      ) : (
        <ul className="space-y-3">
          {workspace.proposals.map((proposal) => (
            <ResumeProposalCard
              key={proposal.id}
              proposal={proposal}
              currentScore={workspace.score.total}
              busy={busy}
              onAccept={onAccept}
              onReject={onReject}
              onUndo={onUndo}
            />
          ))}
        </ul>
      )}
    </div>
  );
}

function statusPill(proposal: ResumeProposal) {
  if (!proposal.applicable) return <StatusPill tone="neutral" icon="block">Non applicable</StatusPill>;
  if (proposal.status === "accepted") return <StatusPill tone="success" icon="check">Ajoutée au CV</StatusPill>;
  if (proposal.status === "rejected") return <StatusPill tone="neutral" icon="close">Ignorée</StatusPill>;
  return <StatusPill tone="accent" icon="query_stats">À décider</StatusPill>;
}

function ResumeProposalCard({
  proposal,
  currentScore,
  busy,
  onAccept,
  onReject,
  onUndo,
}: {
  proposal: ResumeProposal;
  currentScore: number;
  busy: boolean;
  onAccept: (proposal_id: string) => void;
  onReject: (proposal_id: string) => void;
  onUndo: (proposal_id: string) => void;
}) {
  const projected = currentScore + proposal.gain;

  return (
    <li className="space-y-2.5 rounded-card border border-line p-3.5">
      <div className="flex items-start justify-between gap-2">
        <p className="min-w-0 flex-1 text-label font-semibold text-ink">{proposal.label}</p>
        {statusPill(proposal)}
      </div>
      <p className="text-body text-ink-muted">{proposal.proposed_text}</p>
      {!proposal.applicable ? (
        <p className="text-meta text-ink-faint">Ne s’applique plus au CV actuel.</p>
      ) : proposal.status === "pending" ? (
        <>
          <p className="text-meta text-ink-faint">
            Score projeté <strong className="text-ink">{projected}</strong>
          </p>
          <div className="flex gap-2">
            <Button
              variant="primary"
              disabled={busy}
              aria-label={`Accepter ${proposal.label}`}
              onClick={() => onAccept(proposal.id)}
            >
              Accepter
            </Button>
            <Button
              variant="ghost"
              disabled={busy}
              aria-label={`Refuser ${proposal.label}`}
              onClick={() => onReject(proposal.id)}
            >
              Refuser
            </Button>
          </div>
        </>
      ) : (
        <div className="flex gap-2">
          <Button
            variant="ghost"
            disabled={busy}
            aria-label={`Annuler ${proposal.label}`}
            onClick={() => onUndo(proposal.id)}
          >
            Annuler
          </Button>
        </div>
      )}
    </li>
  );
}
